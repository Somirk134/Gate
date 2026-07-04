use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> (StatusCode, Json<Self>) {
        (
            StatusCode::OK,
            Json(Self {
                success: true,
                data: Some(data),
                error: None,
            }),
        )
    }

    pub fn err(status: StatusCode, message: &str) -> (StatusCode, Json<Self>) {
        (
            status,
            Json(Self {
                success: false,
                data: None,
                error: Some(message.to_string()),
            }),
        )
    }
}
