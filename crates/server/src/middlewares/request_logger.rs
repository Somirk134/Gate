use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    tracing::Span,
};
use tracing::info;

pub async fn request_logger_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    info!("--> {} {}", method, uri);

    let response = next.run(request).await;

    info!("<-- {} {} {}", method, uri, response.status());

    response
}
