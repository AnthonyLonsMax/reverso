use hyper::body::Incoming;
use hyper::http::Request;

pub async fn handler(req: Request<Incoming>) {
    let body: Incoming = req.into_body();
}
