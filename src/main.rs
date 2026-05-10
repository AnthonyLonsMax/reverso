mod config;
mod layers;
mod routing;

use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => panic!("error creating the tcp listener {}", err),
    };
    println!("🚀 Server listening on http://{}", addr);

    loop {
        let (stream, address) = listener.accept().await?;
        let io = TokioIo::new(stream);
    }
}
