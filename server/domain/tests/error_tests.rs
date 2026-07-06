use crate::error::{DomainError, DomainFailure, StorageError, ValidateError};

#[test]
fn error_codes_are_stable() {
    assert_eq!(ValidateError::EmptyHost.code(), "DOMAIN_VALIDATE_ERROR");
    assert_eq!(
        DomainError::Storage(StorageError::Unavailable("offline".to_string())).code(),
        "DOMAIN_STORAGE_ERROR"
    );
}

#[test]
fn validate_error_converts_to_domain_error() {
    let error: DomainError = ValidateError::EmptyHost.into();
    assert!(matches!(error, DomainError::Validation(ValidateError::EmptyHost)));
}
