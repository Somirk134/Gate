use axum::{extract::State, Json};
use serde::Serialize;

use crate::application::AppState;

#[derive(Serialize)]
pub struct ConnectionResponse {
    pub id: String,
    pub client_id: String,
    pub remote_addr: String,
    pub connected_at: String,
}

pub async fn list_connections(State(_state): State<AppState>) -> Json<Vec<ConnectionResponse>> {
    todo!("implement list connections handler")
}
