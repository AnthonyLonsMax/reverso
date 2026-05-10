mod config;
mod handlers;
mod layers;
mod routing;

use anyhow::Result;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use handlers::{HelloWorldHandler, UpstreamProxyHandler};
use routing::{Handler, ServiceRoute};

#[tokio::main]
async fn main() -> Result<()> {
    let mut router = ServiceRoute::new();

    router
        .add_route("/", "hello")
        .add_route("/health", "hello")
        .add_handler("hello", HelloWorldHandler);

    router.add_route("/api/{rest...}", "proxy").add_handler(
        "proxy",
        UpstreamProxyHandler {
            target: "http://localhost:8080".parse().unwrap(),
            client: hyper_util::client::legacy::Client::builder(
                hyper_util::rt::TokioExecutor::new(),
            )
            .build(hyper_util::client::legacy::connect::HttpConnector::new()),
        },
    );

    let service = router.build();

    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = TcpListener::bind(addr).await?;
    println!("🚀 Server listening on http://{}", addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let service = service.clone();

        tokio::spawn(async move {
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .with_upgrades() // Permite WebSocket/HTTP2 upgrade
                .await
            {
                eprintln!("❌ Error serving connection from {}: {:?}", peer_addr, err);
            }
        });
    }
}
