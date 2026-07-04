use axum::{extract::State, Json};
use serde::Serialize;

use crate::application::AppState;

#[derive(Serialize)]
pub struct ClientResponse {
    pub id: String,
    pub name: String,
    pub status: String,
}

pub async fn list_clients(State(_state): State<AppState>) -> Json<Vec<ClientResponse>> {
    todo!("implement list clients handler")
}
