mod layers;
mod routing;

use crate::layers::logger::Logger;
use std::{convert::Infallible, net::SocketAddr};

use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    server::conn::http1,
};

use http_body_util::Full;
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use tokio::net::TcpListener;
use tower::ServiceBuilder;

use anyhow::Result;

async fn hello(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => panic!("error creating the tcp listener {}", err),
    };
    println!("🚀 Server listening on http://{}", addr);
    let mut router = routing::RouterService::new();

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            let svc = tower::service_fn(hello);
            let svc = ServiceBuilder::new().layer_fn(Logger::new).service(svc);
            let svc = TowerToHyperService::new(svc);
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("server error: {}", err);
            }
        });
    }
}
