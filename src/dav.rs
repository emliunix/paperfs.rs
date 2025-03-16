use http::{Request, Uri};
use tower::{Service, service_fn};
use dav_server::DavHandler;
use bytes::{Buf, BufMut};
use std::convert::Infallible;
use std::error::Error as StdError;
use std::fmt::Debug;
use std::future::{poll_fn, Future};
use std::pin::{pin, Pin};
use std::task::{Context, Poll};

#[allow(dead_code)]
pub fn into_service<B, D, E>(handler: DavHandler) -> impl Service<http::Request<B>> + Clone + Send + Sync + 'static where
    D: Buf + Send + 'static,
    E: StdError + Send + Sync + 'static,
    B: http_body::Body<Data=D, Error=E> + Send + 'static,
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
        Self {
            inner: handler,
        }
    }
}

impl<B, D, E> Service<http::Request<B>> for DavHandlerWrapper where
    D: Buf + Send + Debug + 'static,
    E: StdError + Send + Sync + 'static,
    B: http_body::Body<Data=D, Error=E> + Send + 'static,
{
    type Response = http::Response<dav_server::body::Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, mut req: http::Request<B>) -> Self::Future {
        log::debug!("DAV {} {}", req.method(), req.uri());
        log::debug!("DAV headers: {:?}", req.headers());
        if req.method() == http::Method::from_bytes("MKCOL".as_bytes()).unwrap() {
            // patch MKCOL for mac not adding trailing / for MKCOL dir
            if !req.uri().path().ends_with("/") {
                let mut builder = Uri::builder();
                if let Some(scheme) = req.uri().scheme() { builder = builder.scheme(scheme.clone()); }
                if let Some(authority) = req.uri().authority() { builder = builder.authority(authority.clone()); }
                if let Some(_) = req.uri().path_and_query() {
                    let pnq = format!("{}/{}", req.uri().path(), req.uri().query().unwrap_or(""));
                    builder = builder.path_and_query(pnq); 
                }
                *req.uri_mut() = builder.build().unwrap();
            }
            log::debug!("DAV patched MKCOL {}", req.uri());
        }
        let inner = self.inner.clone();
        let fut = async move {
            let mut builder = Request::builder()
                .method(req.method())
                .uri(req.uri());
            *builder.headers_mut().unwrap() = req.headers().clone();
            let mut buf = req.body_mut().size_hint().exact().map(|sz| Vec::with_capacity(sz as usize)).unwrap_or_else(Vec::new);
            let mut body = pin!(req.into_body());
            while !body.is_end_stream() {
                log::debug!("DAV poll frame");
                if let Some(data) = poll_fn(|cx| body.as_mut().poll_frame(cx)).await {
                    log::debug!("DAV frame: {:?}", data);
                    buf.put(data.unwrap().into_data().unwrap());
                }
            }
            log::debug!("DAV body collected: {:?} bytes", buf.len());
            match String::from_utf8(buf.clone()) {
                Ok(s) => log::debug!("DAV body collected: {:}", s),
                Err(err) => log::debug!("DAV body collected: {:?}", err),
            }
            Ok(inner.handle(builder.body(axum::body::Body::from(buf)).unwrap()).await)
        };
        Box::pin(fut)
    }
}
