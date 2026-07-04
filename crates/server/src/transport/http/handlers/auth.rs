use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

pub async fn login(Json(_req): Json<LoginRequest>) -> Json<LoginResponse> {
    todo!("implement login handler")
}
