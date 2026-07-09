use thiserror::Error;

#[derive(Debug, Error)]
pub enum TlsInfrastructureError {
    #[error(transparent)]
    Tls(#[from] TlsError),
    #[error(transparent)]
    Acme(#[from] AcmeError),
    #[error(transparent)]
    Certificate(#[from] CertificateError),
}

#[derive(Debug, Error)]
pub enum TlsError {
    #[error("certificate material for domain `{domain}` is missing")]
    MissingMaterial { domain: String },
    #[error("certificate validation failed: {0}")]
    Validation(#[from] CertificateError),
    #[error("certificate for domain `{domain}` expires in {days_remaining} days")]
    ExpiringSoon { domain: String, days_remaining: i64 },
    #[error("TLS provider operation is not connected to an HTTPS runtime")]
    RuntimeDetached,
}

#[derive(Debug, Error)]
pub enum AcmeError {
    #[error("ACME provider `{provider}` is not available")]
    ProviderUnavailable { provider: String },
    #[error("ACME network execution is disabled in this infrastructure phase")]
    NetworkDisabled,
    #[error("ACME provider `{provider}` failed: {reason}")]
    Execution { provider: String, reason: String },
    #[error("ACME challenge `{challenge}` is not supported")]
    UnsupportedChallenge { challenge: String },
    #[error("ACME account is required before creating an order")]
    AccountRequired,
    #[error("invalid ACME state transition from `{from}` to `{to}`")]
    InvalidStateTransition { from: String, to: String },
    #[error("ACME challenge `{challenge}` is not ready")]
    ChallengeNotReady { challenge: String },
    #[error("ACME order `{order_id}` is invalid: {reason}")]
    InvalidOrder { order_id: String, reason: String },
    #[error("certificate error: {0}")]
    Certificate(#[from] CertificateError),
}

#[derive(Debug, Error)]
pub enum CertificateError {
    #[error("certificate for domain `{domain}` was not found")]
    NotFound { domain: String },
    #[error("certificate PEM is invalid: {reason}")]
    InvalidPem { reason: String },
    #[error("certificate parse failed: {reason}")]
    Parse { reason: String },
    #[error("certificate for domain `{domain}` expired at {expired_at}")]
    Expired {
        domain: String,
        expired_at: chrono::DateTime<chrono::Utc>,
    },
    #[error("certificate names do not match domain `{domain}`")]
    DomainMismatch { domain: String },
    #[error("certificate algorithm `{algorithm}` is not allowed")]
    InvalidAlgorithm { algorithm: String },
    #[error("certificate store error: {reason}")]
    Store { reason: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
