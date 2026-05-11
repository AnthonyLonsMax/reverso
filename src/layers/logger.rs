// layers/logger.rs
use hyper::{Request, body::Incoming};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;

// ✅ Logger especializado para hyper::Request<Incoming>
// Ya no es genérico sobre Req, así que podemos llamar a .uri() directamente
#[derive(Clone)]
pub struct Logger<S> {
    inner: S,
}

impl<S> Logger<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, Res, Err> Service<Request<Incoming>> for Logger<S>
where
    S: Service<Request<Incoming>, Response = Res, Error = Err> + Send + 'static,
    Err: std::fmt::Debug + Send + 'static,
    Res: Send + 'static,
{
    type Response = Res;
    type Error = Err;
    type Future = Pin<Box<dyn Future<Output = Result<Res, Err>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        // ✅ Ahora req es Request<Incoming>, podemos llamar a .uri()
        let path = req.uri().path().to_owned();
        let method = req.method().as_str().to_owned();
        println!(
            "📝 [LOG] {} {} @ {}",
            method,
            path,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        let future = self.inner.call(req);
        Box::pin(async move { future.await })
    }
}
