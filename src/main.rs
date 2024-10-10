use std::error::Error as StdError;

use anyhow::Result;
use axum_handler::DavHandlerWrapper;
use buf_layer::BufLayer;
use bytes::{Buf, Bytes};
use dav_server::memls::MemLs;
use dav_server::DavHandler;
use dav_server_opendalfs::OpendalFs;
use opendal::layers::LoggingLayer;
use opendal::raw::HttpClient;
use opendal::services::Onedrive;
use opendal::Operator;
// use reqwest::{Certificate, Proxy};
use tower_http::trace::TraceLayer;

mod axum_handler;
mod buf_layer;

fn dav_svc<B, D, E>(root: &String, access_token: &String) -> Result<DavHandlerWrapper> where
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
        .strip_prefix("/zotero/");
    let handler = strip_prefix
        .filesystem(webdavfs)
        .locksystem(MemLs::new())
        .build_handler();
    // let svc = into_service(handler);
    let svc = DavHandlerWrapper::new(handler);
    Ok(svc)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // get paraemters from env
    let onedrive_root = std::env::var("ONEDRIVE_ROOT").unwrap();
    let onedrive_access_token = std::env::var("ONEDRIVE_ACCESS_TOKEN").unwrap();
    let bind_addr = std::env::var("PAPERFS_BIND_ADDR").unwrap_or("0.0.0.0:3000".into());
    let dav_svc = dav_svc::<axum::body::Body, Bytes, axum::Error>(&onedrive_root, &onedrive_access_token).unwrap();
    let app = axum::Router::new()
        // .route_service("/zotero/*path", dav_svc)
        .route_service("/zotero/", dav_svc.clone())
        .route_service("/zotero/*ignore", dav_svc.clone())
        .layer(TraceLayer::new_for_http());
    log::info!("Server started at {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
