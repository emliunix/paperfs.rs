use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use http::{Request, Response, StatusCode};
use tokio::sync::Mutex;
use tower_service::Service;
use axum::{body::Body, response::IntoResponse};

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
    S::Response: IntoResponse,
{
    type Response = axum::response::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let inner = self.inner.clone();
        Box::pin(async move {
            let res = match &mut *inner.lock().await {
                UninitSvcInner::Uninit => Ok((StatusCode::SERVICE_UNAVAILABLE, "Service not inited").into_response()),
                UninitSvcInner::Inited(svc) => svc.call(req).await.map(|resp| {
                    resp.into_response()
                }),
            };
            res
        })
    }
}
