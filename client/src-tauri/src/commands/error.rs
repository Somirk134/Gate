use chrono::Utc;
use serde::Serialize;
use serde_json::{json, Value};

pub type CommandResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message_key: String,
    pub details: Value,
    pub timestamp: i64,
}

impl AppError {
    pub fn new(code: impl Into<String>, message_key: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message_key: message_key.into(),
            details: json!({}),
            timestamp: Utc::now().timestamp_millis(),
        }
    }

    pub fn with_details(
        code: impl Into<String>,
        message_key: impl Into<String>,
        details: Value,
    ) -> Self {
        Self {
            code: code.into(),
            message_key: message_key.into(),
            details,
            timestamp: Utc::now().timestamp_millis(),
        }
    }

    pub fn from_source(
        code: impl Into<String>,
        message_key: impl Into<String>,
        source: impl std::fmt::Display,
    ) -> Self {
        Self::with_details(
            code,
            message_key,
            json!({
                "source": source.to_string(),
            }),
        )
    }
}
