use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use futures::FutureExt;
use http::Request;
use tokio::sync::{Mutex, MutexGuard};
use tower_service::Service;
use axum::body::Body;

#[derive(Clone)]
pub struct UninitSvc<S> {
    inner: Arc<Mutex<UninitSvcInner<S>>>,
}

pub enum UninitSvcInner<S> {
    Uninit,
    Inited(S),
}

impl<S> UninitSvc<S> {
    pub fn new() -> Self {
        UninitSvc {
            inner: Arc::new(Mutex::new(UninitSvcInner::Uninit)),
        }
    }

    pub async fn init(&self, svc: S) {
        let mut guard = self.inner.lock().await;
        *guard = UninitSvcInner::Inited(svc);
    }
}

impl<S> Service<Request<Body>> for UninitSvc<S>
where
    S: Service<Request<Body>> + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let inner = self.inner.clone();
        Box::pin(async move {
            match &mut *inner.lock().await {
                UninitSvcInner::Uninit => panic!("UninitSvc not inited"),
                UninitSvcInner::Inited(svc) => svc.call(req).await,
            }
        })
    }
}
