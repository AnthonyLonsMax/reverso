use bytes::Bytes;
use http::{Request, Response};
use http_body_util::Full;
use hyper::body::Incoming;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

use crate::routing::{Handler, HandlerError, ProxyResponse};

pub struct HelloWorldHandler;

impl Handler for HelloWorldHandler {
    fn handle(
        &self,
        _req: Request<Incoming>,
    ) -> Pin<Box<dyn Future<Output = Result<ProxyResponse, HandlerError>> + Send>> {
        Box::pin(async move {
            Ok(Response::builder()
                .header("content-type", "text/plain")
                .body(Full::new(Bytes::from("👋 Hello, World!")).boxed())
                .unwrap())
        })
    }
}

pub struct UpstreamProxyHandler {
    pub target: http::Uri,
    pub client: hyper_util::client::legacy::Client<
        hyper_util::client::legacy::connect::HttpConnector,
        Full<Bytes>,
    >,
}

impl Handler for UpstreamProxyHandler {
    fn handle(
        &self,
        mut req: Request<Incoming>,
    ) -> Pin<Box<dyn Future<Output = Result<ProxyResponse, HandlerError>> + Send>> {
        let client = self.client.clone();
        let target = self.target.clone();

        Box::pin(async move {
            let mut parts = req.uri().clone().into_parts();
            parts.scheme = target.scheme().cloned();
            parts.authority = target.authority().cloned();
            *req.uri_mut() = http::Uri::from_parts(parts).unwrap();

            let upstream_res = client.call(req).await?;

            let body = upstream_res
                .into_body()
                .map_err(|e| Box::new(e) as HandlerError)
                .boxed();

            Ok(Response::builder()
                .status(upstream_res.status())
                .body(body)
                .unwrap())
        })
    }
}
