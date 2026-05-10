use std::net::SocketAddr;
use tokio::net::TcpListener;

use anyhow::Result;
use matchit::Router;

mod layers;

#[tokio::main]
async fn main() -> Result<()> {
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => panic!("error creating the tcp listener {}", err),
    };
    println!("🚀 Server listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
    }
}

fn router_gen() -> Result<Router<i32>> {
    let mut router = Router::new();
    router.insert("/users/{id}", 42)?;
    let matched = router.at("/users/1")?;
    return Ok(router);
}
