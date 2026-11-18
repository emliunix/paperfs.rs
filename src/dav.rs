use bytes::Buf;
use dav_server::DavHandler;
use http::Uri;
use std::convert::Infallible;
use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{service_fn, Service};

#[allow(dead_code)]
pub fn into_service<B, D, E>(
    handler: DavHandler,
) -> impl Service<http::Request<B>> + Clone + Send + Sync + 'static
where
    D: Buf + Send + 'static,
    E: StdError + Send + Sync + 'static,
    B: http_body::Body<Data = D, Error = E> + Send + 'static,
{
    let f = move |req| {
        let handler = handler.clone();
        async move {
            Ok::<http::Response<dav_server::body::Body>, Infallible>(handler.handle(req).await)
        }
    };
    service_fn(f)
}

#[derive(Clone)]
pub struct DavHandlerWrapper {
    inner: DavHandler,
}

impl DavHandlerWrapper {
    pub fn new(handler: DavHandler) -> Self {
        Self { inner: handler }
    }
}

impl<B, D, E> Service<http::Request<B>> for DavHandlerWrapper
where
    D: Buf + Send + 'static,
    E: StdError + Send + Sync + 'static,
    B: http_body::Body<Data = D, Error = E> + Send + 'static,
{
    type Response = http::Response<dav_server::body::Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, mut req: http::Request<B>) -> Self::Future {
        log::debug!("DAV {} {}", req.method(), req.uri());
        if req.method() == http::Method::from_bytes("MKCOL".as_bytes()).unwrap() {
            // patch MKCOL for mac not adding trailing / for MKCOL dir
            if !req.uri().path().ends_with("/") {
                let mut builder = Uri::builder();
                if let Some(scheme) = req.uri().scheme() {
                    builder = builder.scheme(scheme.clone());
                }
                if let Some(authority) = req.uri().authority() {
                    builder = builder.authority(authority.clone());
                }
                if let Some(_) = req.uri().path_and_query() {
                    let pnq = format!("{}/{}", req.uri().path(), req.uri().query().unwrap_or(""));
                    builder = builder.path_and_query(pnq);
                }
                *req.uri_mut() = builder.build().unwrap();
                log::debug!("DAV patched MKCOL {}", req.uri());
            }
        }
        let inner = self.inner.clone();
        Box::pin(async move { Ok(inner.handle(req).await) })
    }
}
