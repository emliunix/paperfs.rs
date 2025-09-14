use std::error::Error as StdError;
use std::future::IntoFuture;
// std::future::IntoFuture; not used after switching to explicit server handling

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

// shutdown helper: listen for Ctrl+C and SIGTERM on unix
async fn shutdown_signal() {
    // Wait for Ctrl+C
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to listen for ctrl_c");
    };

    // On Unix also listen for SIGTERM
    #[cfg(unix)]
    let term = async {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM");
        sigterm.recv().await;
    };

    #[cfg(not(unix))]
    let term = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = term => {},
    }

    log::info!("shutdown signal received");
}
// use reqwest::{Certificate, Proxy};
use tower_http::trace::TraceLayer;
use types::OneDriveArgs;
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

fn dav_svc<B, D, E>(args: &OneDriveArgs) -> Result<DavHandlerWrapper> where
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
    let mut builder = Onedrive::default()
        .root(&args.onedrive_root)
        .client_id(&args.client_id)
        .refresh_token(args.refresh_token.as_ref().unwrap());
    if let Some(client_secret) = args.client_secret.as_ref() {
        builder = builder.client_secret(client_secret);
    }
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

async fn set_token(svc: UninitSvc<DavHandlerWrapper>, args: &OneDriveArgs) -> Result<(), anyhow::Error> {
        svc.init(dav_svc::<axum::body::Body, Bytes, axum::Error>(args)?).await;
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

fn token_cb(svc: UninitSvc<DavHandlerWrapper>, args: OneDriveArgs) -> impl Fn(ODriveState) + Clone + Send + 'static {
    move |state| {
        let svc = svc.clone();
        let args = args.clone();
        tokio::spawn(async move {
            save_token(state.clone()).await.log_err("failed to save refresh token");
            let refresh_token = state.refresh_token.expect("refresh token not found");
            set_token(svc.clone(), &OneDriveArgs { refresh_token: Some(refresh_token), ..args}).await.log_err("failed to set refresh token");
        });
    }
}

static GIT_REVISION: &str = env!("GIT_REVISION");

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    log::info!("paperfs version: {}", GIT_REVISION);
    
    // console_subscriber::init();
    // get paraemters from env
    let onedrive_root = std::env::var("ONEDRIVE_ROOT").log_err("ONEDRIVE_ROOT not provided");
    let onedrive_client_id = std::env::var("ONEDRIVE_CLIENT_ID").log_err("ONEDRIVE_CLIENT_ID not provided");
    let onedrive_client_secret = std::env::var("ONEDRIVE_CLIENT_SECRET").ok(); // optional
    // let onedrive_access_token = std::env::var("ONEDRIVE_ACCESS_TOKEN").unwrap();
    let bind_addr = std::env::var("PAPERFS_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let exposed_url = std::env::var("PAPERFS_EXPOSED_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let onedrive_args = OneDriveArgs {
        onedrive_root: onedrive_root,
        client_id: onedrive_client_id,
        client_secret: onedrive_client_secret,
        ..Default::default()
    };

    // dav service
    let svc = UninitSvc::new();

    if let Some(state) = load_token().await.log_err("error loading saved refresh token") {
        log::info!("Loaded persisted refresh data");
        let refresh_token = state.refresh_token.expect("state not found");
        set_token(svc.clone(), &OneDriveArgs { refresh_token: Some(refresh_token), ..onedrive_args.clone()}).await.log_err("failed to set refresh token");
    }

    // axum router
    let router = axum::Router::new()
        .route("/", get(Html(include_str!("../static/index.html"))))
        .route_service("/zotero", svc.clone())
        .route_service("/zotero/", svc.clone())
        .route_service("/zotero/{*ignore}", svc.clone())
        .nest("/api/v1/onedrive", onedrive_api_router(
            &onedrive_args,
            &exposed_url,
            None,  // let's pass None for now, effectively making refresh_token one-way passing only to dav_svc and saved file
            token_cb(svc.clone(), onedrive_args.clone())
        ))
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(64 * 1024 * 1024));

    // parse bind address and start hyper server with graceful shutdown
    let addr: std::net::SocketAddr = bind_addr.parse().expect("invalid bind address");
    log::info!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("failed to bind address");
    let server = axum::serve(listener, router).with_graceful_shutdown(shutdown_signal()).into_future();
    if let Err(e) = server.await {
        log::error!("server error: {}", e);
    }
}
