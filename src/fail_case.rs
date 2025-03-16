use std::{env, task::{Context, Poll}};

use anyhow::Ok;
use opendal::{layers::TracingLayer, raw::HttpClient, services::Onedrive, Builder, OperatorBuilder};
// use reqwest::Proxy;
use tower::{Layer, Service};

#[derive(Debug, Clone)]
struct DebugLayer;

impl DebugLayer {
    fn new() -> Self {
        DebugLayer
    }
}

impl<S> Layer<S> for DebugLayer {
    type Service = DebugService<S>;

    fn layer(&self, service: S) -> Self::Service {
        DebugService { service  }
    }
}

#[derive(Debug, Clone)]
struct DebugService<S> {
    service: S,
}

impl<S, Request> Service<Request> for DebugService<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        self.service.call(req)
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt::init();

    let onedrive = Onedrive::default()
        .http_client(HttpClient::with(reqwest::ClientBuilder::new()
            // .proxy(Proxy::https("http://192.168.50.100:7890")?)
            .connection_verbose(true)
            .connector_layer(DebugLayer::new())
            .build()?))
        .access_token(env::var("ONEDRIVE_TOKEN")?.as_str());
    let operator = OperatorBuilder::new(onedrive.build()?)
        .layer(TracingLayer{})
        .finish();
    let mut data = Vec::with_capacity(5*1024*1024);  // 5MB
    let dummy: [u8; 16] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
    for _ in 0..5*1024*1024/16 {
        data.extend_from_slice(&dummy);
    }
    operator.write("/test.data", data).await?;
    Ok(())
}
