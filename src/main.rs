use std::pin::Pin;

use hyper::{Request, Response, body::Incoming, server::conn::http1, service::Service};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, Hello).await {
                println!("error creating the request service handler {err}")
            };
        });
    }
}

pub struct Hello;

impl Service<Request<Incoming>> for Hello {
    type Response = Response<String>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, _req: Request<Incoming>) -> Self::Future {
        Box::pin(async move {
            let response = Response::builder()
                .status(200)
                .body(String::from("Hello response"))
                .map_err(|e| Box::new(e))
                .unwrap();
            Ok(response)
        })
    }
}
