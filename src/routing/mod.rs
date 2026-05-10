use bytes::Bytes;
use http::{Request, Response, StatusCode};
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use matchit::Router;
use std::collections::HashMap;
use std::convert::Infallible;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tower::Service;

pub type HandlerError = Box<dyn Error + Send + Sync + 'static>;
pub type ProxyBody = BoxBody<Bytes, Infallible>;
pub type ProxyResponse = Response<ProxyBody>;

pub trait Handler: Send + Sync {
    fn handle(
        &self,
        req: Request<Incoming>,
    ) -> Pin<Box<dyn Future<Output = Result<ProxyResponse, HandlerError>> + Send>>;
}

#[derive(Clone)]
pub struct ServiceRoute {
    routes: Router<String>,
    handlers: HashMap<String, Arc<dyn Handler>>,
}

impl ServiceRoute {
    pub fn new() -> Self {
        Self {
            routes: Router::new(),
            handlers: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, path: &str, key: &str) -> &mut Self {
        self.routes.insert(path, key.to_string()).unwrap();
        self
    }

    pub fn add_handler(&mut self, key: &str, handler: impl Handler + 'static) -> &mut Self {
        self.handlers.insert(key.to_string(), Arc::new(handler));
        self
    }

    pub fn build(self) -> Self {
        self
    }
}

impl Service<Request<Incoming>> for ServiceRoute {
    type Response = ProxyResponse;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        let routes = self.routes.clone();
        let handlers = self.handlers.clone();
        let path = req.uri().path().to_string();

        Box::pin(async move {
            let handler = match routes.at(&path) {
                Ok(m) => handlers.get(m.value),
                Err(_) => None,
            };

            match handler {
                Some(h) => match h.handle(req).await {
                    Ok(res) => Ok(res),
                    Err(e) => {
                        eprintln!("Handler error: {}", e);
                        Ok(Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Full::new(Bytes::from("Internal Server Error")).boxed())
                            .unwrap())
                    }
                },
                None => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from("Not Found")).boxed())
                    .unwrap()),
            }
        })
    }
}
