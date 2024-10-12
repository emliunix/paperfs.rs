use std::error::Error as StdError;
use std::future::{Future, IntoFuture};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum_handler::DavHandlerWrapper;
use buf_layer::BufLayer;
use bytes::{Buf, Bytes};
use dav_server::memls::MemLs;
use dav_server::DavHandler;
use dav_server_opendalfs::OpendalFs;
use odrive::ODriveSession;
use opendal::layers::LoggingLayer;
use opendal::raw::HttpClient;
use opendal::services::Onedrive;
use opendal::Operator;
use tokio::join;
use tokio::sync::Mutex;
// use reqwest::{Certificate, Proxy};
use tower_http::trace::TraceLayer;
use uninit_svc::UninitSvc;

mod axum_handler;
mod buf_layer;
mod odrive;
mod uninit_svc;

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
    let onedrive_client_id = std::env::var("ONEDRIVE_CLIENT_ID").unwrap();
    let onedrive_access_token = std::env::var("ONEDRIVE_ACCESS_TOKEN").unwrap();
    let bind_addr = std::env::var("PAPERFS_BIND_ADDR").unwrap_or("0.0.0.0:3000".into());

    let scopes = &["Files.ReadWrite.All"];
    let authority = "https://login.microsoftonline.com/consumers";

    // onedrive session
    let od_sess = ODriveSession::new(onedrive_access_token.clone(), authority, onedrive_client_id.as_str(), scopes).await;
    // initial onedrive token
    od_sess.update().await;

    // dav service
    let svc = UninitSvc::new();

    // refresh onedrive token and dav svc
    let refresh_thread = async {
        let svc = svc.clone();
        let od_sess = od_sess.clone();
        loop {
            od_sess.update().await;
            let access_token = od_sess.access_token().await.unwrap();
            svc.init(dav_svc::<axum::body::Body, Bytes, axum::Error>(
                &onedrive_root, &access_token, 
            ).unwrap()).await;
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    };

    // axum app
    let app = axum::Router::new()
        // .route_service("/zotero/*path", dav_svc)
        .route_service("/zotero/", svc.clone())
        .route_service("/zotero/*ignore", svc.clone())
        .layer(TraceLayer::new_for_http());

    // start server
    log::info!("Server started at {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();

    let (res, _) = join!(
        axum::serve(listener, app).into_future(),
        refresh_thread
    );
    res.unwrap();
}
