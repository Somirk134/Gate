use axum::{extract::State, Json};
use serde::Serialize;

use crate::application::AppState;

#[derive(Serialize)]
pub struct TunnelResponse {
    pub id: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: String,
    pub status: String,
}

pub async fn list_tunnels(State(_state): State<AppState>) -> Json<Vec<TunnelResponse>> {
    todo!("implement list tunnels handler")
}
