use std::convert::Infallible;
use std::error::Error as StdError;
use std::fs::File;
use std::future::IntoFuture;
use std::io::Write;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::routing::get;
use dav::DavHandlerWrapper;
use buf_layer::BufLayer;
use bytes::{Buf, Bytes};
use dav_server::memls::MemLs;
use dav_server::DavHandler;
use dav_server_opendalfs::OpendalFs;
use futures::FutureExt;
use http::{Request, StatusCode, Uri};
use odrive::{ODrivePersist, ODriveSession};
use odrive_handler::onedrive_api_router;
use opendal::layers::LoggingLayer;
use opendal::raw::HttpClient;
use opendal::services::Onedrive;
use opendal::Operator;
use tokio::join;
use tower::ServiceExt;
// use reqwest::{Certificate, Proxy};
use tower_http::trace::TraceLayer;
use types::AppEvents;
use uninit_svc::UninitSvc;
use utils::LogError;

mod dav;
mod buf_layer;
mod odrive;
mod odrive_handler;
mod uninit_svc;
mod types;
mod utils;

fn dav_svc<B, D, E>(root: &String, access_token: &str) -> Result<DavHandlerWrapper> where
    D: Buf + Send + 'static,
    E: StdError + Send + Sync + 'static,
    B: http_body::Body<Data=D, Error=E> + Send + 'static,
{
    // let cert = Certificate::from_pem(include_bytes!("../cert.pem"))?;
    // 1drive fs
    let mut builder = Onedrive::default();
    builder.root(root)
        .access_token(access_token)
        .http_client(
            HttpClient::build(
                reqwest::ClientBuilder::new()
                    // .proxy(Proxy::https("http://localhost:8080")?)
                    // .add_root_certificate(cert)
            )?);
    let op = Operator::new(builder)?
        .layer(BufLayer::default())
        .layer(LoggingLayer::default())
        .finish();
    // dav fs
    let webdavfs = OpendalFs::new(op);
    // http handler
    let strip_prefix = DavHandler::builder()
        .strip_prefix("/zotero");
    let handler = strip_prefix
        .filesystem(webdavfs)
        .locksystem(MemLs::new())
        .build_handler();
    // let svc = into_service(handler);
    let svc = DavHandlerWrapper::new(handler);
    Ok(svc)
}

#[derive(Clone)]
struct App {
    od_root: String,
    od_sess: ODriveSession,
    svc: UninitSvc<DavHandlerWrapper>,
}

impl AppEvents for App {
    fn on_refresh(&self) {
        let self_ = self.clone();
        tokio::spawn(async move {
            if let None = self_.od_sess.access_token().await {
                log::warn!("Not login, abort refresh");
                return anyhow::Ok(());
            }
            self_.od_sess.refresh().await?;
            self_.on_token_change();
            anyhow::Ok(())
        }.map(|res| {res.log_err("failed to refresh"); Ok::<(), Infallible>(()) }));
    }

    fn on_token_change(&self) {
        let self_ = self.clone();
        tokio::spawn(async move {
            let access_token = self_.od_sess.access_token().await.context("No access token")?;
            self_.svc.init(dav_svc::<axum::body::Body, Bytes, axum::Error>(
                &self_.od_root, &access_token, 
            )?).await;
            // persist tokens
            let persist = self_.od_sess.to_persist().await?;
            let mut file = File::create("app_data.json")?;
            file.write_all(serde_json::to_string(&persist)?.as_bytes())?;
            file.flush()?;
            drop(file);
            anyhow::Ok(())
        }.map(|res| {res.log_err("failed to process token change"); Ok::<(), Infallible>(()) }));
    }

    fn on_auth(&self, code: String) {
        let self_ = self.clone();
        tokio::spawn(async move {
            self_.od_sess.auth(code.as_str()).await?;
            self_.on_token_change();
            anyhow::Ok(())
        }.map(|res| {res.log_err("failed to auth"); Ok::<(), Infallible>(()) }));
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // get paraemters from env
    let onedrive_root = std::env::var("ONEDRIVE_ROOT").unwrap();
    let onedrive_client_id = std::env::var("ONEDRIVE_CLIENT_ID").unwrap();
    // let onedrive_access_token = std::env::var("ONEDRIVE_ACCESS_TOKEN").unwrap();
    let bind_addr = std::env::var("PAPERFS_BIND_ADDR").unwrap_or("0.0.0.0:3000".into());
    let exposed_url = std::env::var("PAPERFS_EXPOSED_URL").unwrap_or("http://localhost:3000".into());

    // check if persisted app data exists, and 
    let persisted_data = match std::fs::read_to_string("app_data.json") {
        Ok(data) => {
            log::info!("Recover persisted app data");
            log::debug!("Persisted data: {}", data);
            let persist: ODrivePersist = serde_json::from_str(data.as_str()).unwrap();
            Some(persist)
        },
        Err(_) => None,
    };
    // onedrive session
    let od_sess = ODriveSession::new(
        reqwest::Client::new(),
        &onedrive_client_id,
        format!("{}/api/v1/onedrive/callback", exposed_url).as_str(),
        None,
        persisted_data.clone(),
    );

    // dav service
    let svc = UninitSvc::new();

    let app = App { 
        od_root: onedrive_root.clone(),
        od_sess: od_sess.clone(),
        svc: svc.clone(),
    };

    if persisted_data.is_some() {
        log::info!("Trigger refresh due to recovering persisted data");
        app.on_refresh();
    }
    // refresh onedrive token and dav svc
    let refresh_thread = async {
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
            app.on_refresh();
        }
    };

    // axum router
    let router = axum::Router::new()
        .route_service("/zotero", svc.clone().map_request(|req: Request<axum::body::Body>| {
            let (mut parts, body) = req.into_parts();

            log::info!("rewrite request uri: before: {}", parts.uri);
            let mut builder = Uri::builder();
            if let Some(scheme) = parts.uri.scheme() {
                builder = builder.scheme(scheme.clone());
            }
            if let Some(authority) = parts.uri.authority() {
                builder = builder.authority(authority.clone());
            }
            parts.uri = builder
                .path_and_query("/zotero")
                .build()
                .unwrap();
            log::info!("after: {}", parts.uri);
            Request::from_parts(parts, body)
        }))
        .route_service("/zotero/", svc.clone())
        .route_service("/zotero/*ignore", svc.clone())
        .nest("/api/v1/onedrive", onedrive_api_router(od_sess.clone(), app.clone()))
        .route("/token", get({
            let app = app.clone();
            || async move {
                match app.od_sess.access_token().await {
                    Some(token) => (StatusCode::OK, token),
                    None => (StatusCode::NOT_FOUND, "".to_string()),
                }
            }
        }))
        .layer(TraceLayer::new_for_http());

    // start server
    log::info!("Server started at {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();

    let (res, _) = join!(
        axum::serve(listener, router).into_future(),
        refresh_thread
    );
    res.unwrap();
}
