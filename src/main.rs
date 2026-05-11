mod layers;
mod routing;

use std::net::SocketAddr;

use anyhow::Result;
use bytes::Bytes as BytesType;
use http_body_util::{BodyExt, Full};
use hyper::{Response, body::Bytes};
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower::service_fn;

use crate::{layers::logger::Logger, routing::ProxyRouterService};

pub type ProxyError = Box<dyn std::error::Error + Send + Sync>;
pub type ProxyResponse = Response<http_body_util::combinators::BoxBody<BytesType, ProxyError>>;

async fn hello(_: hyper::Request<hyper::body::Incoming>) -> Result<ProxyResponse, ProxyError> {
    let body = Full::new(Bytes::from("Hello, World!"))
        .map_err(|e| Box::new(e) as ProxyError)
        .boxed();

    Ok(Response::new(body))
}

async fn users_handler(
    _: hyper::Request<hyper::body::Incoming>,
) -> Result<ProxyResponse, ProxyError> {
    let json = r#"{"users":["alice","bob"]}"#;
    let body = Full::new(Bytes::from(json))
        .map_err(|e| Box::new(e) as ProxyError)
        .boxed();

    Response::builder()
        .header("content-type", "application/json")
        .body(body)
        .map_err(|e| Box::new(e) as ProxyError)
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = TcpListener::bind(addr).await?;
    println!("🚀 Server listening on http://{}", addr);

    // ✅ 1. Crear y configurar el router UNA SOLA VEZ al inicio
    let mut router = ProxyRouterService::new();

    // Registrar handlers con BoxCloneService para que sean clonables
    router.proxy(
        "/users".to_string(),
        "/api/users".to_string(),
        tower::util::BoxCloneService::new(service_fn(users_handler)),
    );

    router.proxy(
        "/hello".to_string(),
        "/api/hello".to_string(),
        tower::util::BoxCloneService::new(service_fn(hello)),
    );

    let service = ServiceBuilder::new().layer_fn(Logger::new).service(router);

    let hyper_service = TowerToHyperService::new(service);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let svc = hyper_service.clone();

        tokio::task::spawn(async move {
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, svc)
                .await
            {
                eprintln!("server error: {}", err);
            }
        });
    }
}
