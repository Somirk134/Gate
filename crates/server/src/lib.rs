pub mod application;
pub mod authentication;
pub mod common;
pub mod config;
pub mod connection;
pub mod domain;
pub mod errors;
pub mod events;
pub mod infrastructure;
pub mod logging;
pub mod middlewares;
pub mod security;
pub mod statistics;
pub mod storage;
pub mod transport;
pub mod tunnel;
pub mod utils;

use anyhow::Result;

pub async fn run() -> Result<()> {
    let config = config::load()?;
    logging::init(&config)?;

    let app_state = application::AppState::new(&config).await?;
    let router = transport::http::router(app_state.clone());

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
    tracing::info!("Gate server starting on {}:{}", config.host, config.port);

    axum::serve(listener, router)
        .await?;

    Ok(())
}
