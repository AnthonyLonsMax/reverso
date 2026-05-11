use std::sync::Arc;
use tower::util::BoxCloneService;

pub type ProxyError = Box<dyn std::error::Error + Send + Sync>;
pub type ProxyResponse = Response<http_body_util::combinators::BoxBody<bytes::Bytes, ProxyError>>;
pub type ProxyRequest = Request<Incoming>;
pub type ProxyService = BoxCloneService<ProxyRequest, ProxyResponse, ProxyError>;

#[derive(Clone)]
pub struct ProxyRouterService {
    inner: Arc<InnerRouter>,
}

struct InnerRouter {
    router: matchit::Router<String>,
    handlers: std::collections::HashMap<String, ProxyService>,
}

impl ProxyRouterService {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(InnerRouter {
                router: matchit::Router::new(),
                handlers: std::collections::HashMap::new(),
            }),
        }
    }

    // ✅ Para registrar rutas, necesitamos mutabilidad: devolvemos un nuevo Arc con los cambios
    pub fn with_route(mut self, path: String, destination: String, service: ProxyService) -> Self {
        // Arc::make_mut clona el contenido si hay múltiples referencias
        let inner = Arc::make_mut(&mut self.inner);
        inner.router.insert(path, destination.clone()).ok();
        inner.handlers.insert(destination, service);
        self
    }
}

impl tower::Service<ProxyRequest> for ProxyRouterService {
    type Response = ProxyResponse;
    type Error = ProxyError;
    type Future =
        std::pin::Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Con Arc, no podemos obtener &mut InnerRouter fácilmente para poll_ready
        // En producción real, considera usar tower::buffer o quitar poll_ready si tus handlers siempre están ready
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: ProxyRequest) -> Self::Future {
        // ✅ Extraer datos sincrónicamente antes del async move
        let path = req.uri().path().to_owned();
        let inner = Arc::clone(&self.inner);

        Box::pin(async move {
            // ✅ Hacer el match dentro del future con el Arc clonado
            match inner.router.at(&path) {
                Ok(match_) => {
                    if let Some(mut handler) = inner.handlers.get(match_.value).cloned() {
                        return handler.call(req).await;
                    }
                }
                Err(_) => {}
            }

            // 404 fallback
            let body = http_body_util::Full::new(bytes::Bytes::from("Not Found"))
                .map_err(|_| unreachable!())
                .boxed();
            Response::builder()
                .status(404)
                .body(body)
                .map_err(|e| Box::new(e) as ProxyError)
        })
    }
}
