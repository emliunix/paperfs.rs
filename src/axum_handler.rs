use tower::{Service, service_fn};
use dav_server::DavHandler;
use bytes::Buf;
use std::convert::Infallible;
use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

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
    D: Buf + Send + 'static,
    E: StdError + Send + Sync + 'static,
    B: http_body::Body<Data=D, Error=E> + Send + 'static,
{
    type Response = http::Response<dav_server::body::Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        log::debug!("{} {}", req.method(), req.uri());
        let self_ = self.clone();
        let fut = async move {
            Ok(self_.inner.handle(req).await)
        };
        Box::pin(fut)
    }
}
