#![feature(impl_trait_in_fn_trait_return)]
use std::error::Error as StdError;
use std::future::IntoFuture;

use anyhow::{Context, Result};
use axum::extract::DefaultBodyLimit;
use axum::response::Html;
use axum::routing::get;
use buf_layer::BufLayer;
use dav::DavHandlerWrapper;
use bytes::{Buf, Bytes};
use dav_server::memls::MemLs;
use dav_server::DavHandler;
use dav_server_opendalfs::OpendalFs;
use mux_layer::MuxLayer;
use odrive::ODriveState;
use odrive_handler::onedrive_api_router;
use opendal::layers::LoggingLayer;
use opendal::services::{Memory, Onedrive};
use opendal::{Builder, Operator};
use tokio::fs::{read_to_string, File};
use tokio::io::AsyncWriteExt;
// use reqwest::{Certificate, Proxy};
use tower_http::trace::TraceLayer;
use uninit_svc::UninitSvc;
use utils::LogError;

mod dav;
mod buf_layer;
mod mux_layer;
mod odrive;
mod odrive_handler;
mod uninit_svc;
mod types;
mod utils;

/// remove the `is_fn` will cause error, maybe that's too much guessing of types
/// and rust internally has a search depth limit prevents from resolving
fn is_fn<F: (Fn(&str) -> bool) + 'static + Send + Sync + Unpin + Clone>(f: F) -> F { f }

fn dav_svc<B, D, E>(root: &str, client_id: &str, refresh_token: &str) -> Result<DavHandlerWrapper> where
    D: Buf + Send + 'static,
    E: StdError + Send + Sync + 'static,
    B: http_body::Body<Data=D, Error=E> + Send + 'static,
{
    // let cert = Certificate::from_pem(include_bytes!("../cert.pem"))?;
    // 1drive fs
    // let http_client = HttpClient::with(
    //     reqwest::ClientBuilder::new()
    //     // .proxy(Proxy::https("http://localhost:8080")?)
    //     // .add_root_certificate(cert)
    //     .build()?);
    let builder = Onedrive::default()
        .root(root)
        .client_id(client_id)
        .refresh_token(refresh_token);
    let mux_layer = MuxLayer::new(|| Memory::default().build().unwrap(), is_fn(|path| {
        // split into dir and file
        let mut parts = path.rsplitn(2, '/');
        let file = parts.next().unwrap_or(path);
        // let dir = parts.next().unwrap_or("/");
        let res = file.starts_with("._") || file.ends_with("DS_Store");
        log::debug!("route {} to {}", path, if res { "memory" } else { "onedrive" });
        res
    }));
    let op = Operator::new(builder)?
        .layer(BufLayer::default())
        .layer(mux_layer)
        .layer(LoggingLayer::default())
        .finish();
    // dav fs
    let webdavfs = OpendalFs::new(op);
    // http handler
    let dav_config = DavHandler::builder()
        .strip_prefix("/zotero")
        .filesystem(webdavfs)
        .locksystem(MemLs::new());
    let handler = dav_config
        .build_handler();
    // let svc = into_service(handler);
    let svc = DavHandlerWrapper::new(handler);
    Ok(svc)
}

async fn set_token(svc: UninitSvc<DavHandlerWrapper>, client_id: &str, od_root: &str, refresh_token: String) -> Result<(), anyhow::Error> {
        svc.init(dav_svc::<axum::body::Body, Bytes, axum::Error>(
            od_root, client_id, &refresh_token, 
        )?).await;
        anyhow::Ok(())
}

async fn save_token(state: ODriveState) -> Result<(), anyhow::Error> {
    // persist tokens
    let state_json = serde_json::to_string(&state).context("failed to serialize state")?;
    File::create("app_data.json").await?.write_all(state_json.as_bytes()).await?;
    Ok(())
}

async fn load_token() -> Result<Option<ODriveState>, anyhow::Error> {
    // test exists
    if std::path::Path::new("app_data.json").exists() {
        let data = read_to_string("app_data.json").await?;
        let data: ODriveState = serde_json::from_str(&data).context("failed to deserialize state")?;
        return Ok(Some(data));
    }
    Ok(None)
}

fn token_cb(svc: UninitSvc<DavHandlerWrapper>, client_id: String, od_root: String) -> impl Fn(ODriveState) + Clone + Send + 'static {
    move |state| {
        let svc = svc.clone();
        let client_id = client_id.clone();
        let od_root = od_root.clone();
        tokio::spawn(async move {
            save_token(state.clone()).await.log_err("failed to save refresh token");
            let refresh_token = state.refresh_token.expect("refresh token not found");
            set_token(svc.clone(), &client_id, &od_root, refresh_token).await.log_err("failed to set refresh token");
        });
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // console_subscriber::init();
    // get paraemters from env
    let onedrive_root = std::env::var("ONEDRIVE_ROOT").log_err("ONEDRIVE_ROOT not provided");
    let onedrive_client_id = std::env::var("ONEDRIVE_CLIENT_ID").log_err("ONEDRIVE_CLIENT_ID not provided");
    let onedrive_client_secret = std::env::var("ONEDRIVE_CLIENT_SECRET").log_err("ONEDRIVE_CLIENT_SECRET not provided");
    // let onedrive_access_token = std::env::var("ONEDRIVE_ACCESS_TOKEN").unwrap();
    let bind_addr = std::env::var("PAPERFS_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let exposed_url = std::env::var("PAPERFS_EXPOSED_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    // dav service
    let svc = UninitSvc::new();

    if let Some(state) = load_token().await.log_err("error loading saved refresh token") {
        log::info!("Loaded persisted refresh data");
        let refresh_token = state.refresh_token.expect("state not found");
        set_token(svc.clone(), &onedrive_client_id, &onedrive_root, refresh_token).await.log_err("failed to set refresh token");
    }

    // axum router
    let router = axum::Router::new()
        .route("/", get(Html(include_str!("../static/index.html"))))
        .route_service("/zotero", svc.clone())
        .route_service("/zotero/", svc.clone())
        .route_service("/zotero/{*ignore}", svc.clone())
        .nest("/api/v1/onedrive", onedrive_api_router(
            &onedrive_client_id,
            &exposed_url,
            None,  // let's pass None for now, effectively making refresh_token one-way passing only to dav_svc and saved file
            token_cb(svc.clone(), onedrive_client_id.clone(), onedrive_root.clone())
        ))
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(64 * 1024 * 1024));

    // start server
    log::info!("Server started at {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.log_err("failed to bind to address");

    axum::serve(listener, router).into_future().await.unwrap();
}
