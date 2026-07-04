use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::application::AppState;

mod inner {
    use super::*;
    use crate::transport::http::handlers;

    pub fn api_router() -> Router<AppState> {
        Router::new()
            .route("/health", get(handlers::health::health_check))
            .route("/api/v1/auth/login", get(handlers::auth::login))
            .route("/api/v1/clients", get(handlers::client::list_clients))
            .route("/api/v1/tunnels", get(handlers::tunnel::list_tunnels))
            .route("/api/v1/connections", get(handlers::connection::list_connections))
            .route("/api/v1/statistics", get(handlers::statistics::get_statistics))
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/", inner::api_router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
