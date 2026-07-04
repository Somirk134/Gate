use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

pub struct AuthenticatedClient {
    pub client_id: String,
}

impl<S> FromRequestParts<S> for AuthenticatedClient
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        todo!("extract authenticated client from request headers")
    }
}
