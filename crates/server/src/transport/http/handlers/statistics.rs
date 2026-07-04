use axum::{extract::State, Json};
use serde::Serialize;

use crate::application::AppState;

#[derive(Serialize)]
pub struct StatisticsResponse {
    pub active_connections: u64,
    pub total_clients: u64,
    pub total_tunnels: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

pub async fn get_statistics(State(_state): State<AppState>) -> Json<StatisticsResponse> {
    todo!("implement statistics handler")
}
