use std::{
    collections::HashMap,
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::{Request, Response, body::Incoming};
use matchit::Router;
use tower::Service;

pub type ProxyError = Box<dyn std::error::Error + Send + Sync>;

pub type ProxyResponse = Response<BoxBody<Bytes, ProxyError>>;
pub type ProxyFuture = Pin<Box<dyn Future<Output = Result<ProxyResponse, ProxyError>> + Send>>;
pub type ProxyRequest = Request<Incoming>;

pub type ProxyService = Box<
    dyn Service<ProxyRequest, Response = ProxyResponse, Error = ProxyError, Future = ProxyFuture>
        + Send,
>;

struct ProxyRouterService {
    router: Router<String>,
    handlers: HashMap<String, ProxyService>,
}

impl ProxyRouterService {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            handlers: HashMap::new(),
        }
    }
}

struct HelloWorld;

impl Service<ProxyRequest> for HelloWorld {
    type Response = ProxyResponse;
    type Error = ProxyError;
    type Future = ProxyFuture;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: ProxyRequest) -> Self::Future {
        Box::pin(async move {
            let body = Full::new(Bytes::from("SSR request"))
                .map_err(|e: Infallible| -> ProxyError { Box::new(e) })
                .boxed();
            Ok(Response::builder()
                .status(200)
                .body(body)
                .unwrap()
                .map(|b| BoxBody::new(b)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_route() {
        let mut router = ProxyRouterService::new();
        router
            .handlers
            .insert("/hello".to_owned(), Box::new(HelloWorld));
    }
}
