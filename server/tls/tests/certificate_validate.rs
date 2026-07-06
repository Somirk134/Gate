use chrono::{Duration, Utc};
use gate_server_tls::certificate::{CertificateAlgorithm, CertificateParser, CertificateValidator};

#[test]
fn validator_accepts_matching_san_domain() {
    let certificate = rcgen::generate_simple_self_signed(vec![
        "example.com".to_string(),
        "api.example.com".to_string(),
    ])
    .expect("test certificate should be generated");
    let pem = certificate
        .serialize_pem()
        .expect("test certificate should serialize");

    let record = CertificateValidator::default()
        .validate_pem_for_domain(&pem, "api.example.com")
        .expect("certificate should validate");

    assert_eq!(record.domain, "api.example.com");
}

#[test]
fn validator_rejects_domain_mismatch() {
    let certificate = rcgen::generate_simple_self_signed(vec!["example.com".to_string()])
        .expect("test certificate should be generated");
    let pem = certificate
        .serialize_pem()
        .expect("test certificate should serialize");
    let record =
        CertificateParser::parse_pem("example.com", &pem).expect("test certificate should parse");

    let result = CertificateValidator::default().validate_record(&record, "other.example.com");

    assert!(result.is_err());
}

#[test]
fn validator_rejects_disallowed_algorithm() {
    let certificate = rcgen::generate_simple_self_signed(vec!["algorithm.example.com".to_string()])
        .expect("test certificate should be generated");
    let pem = certificate
        .serialize_pem()
        .expect("test certificate should serialize");
    let mut record = CertificateParser::parse_pem("algorithm.example.com", &pem)
        .expect("test certificate should parse");
    record.algorithm = CertificateAlgorithm::Unknown("unsupported".to_string());
    let validator = CertificateValidator::default();

    let result = validator.validate_record(&record, "algorithm.example.com");

    assert!(result.is_err());
}

#[test]
fn validator_rejects_expired_certificate() {
    let certificate = rcgen::generate_simple_self_signed(vec!["expired.example.com".to_string()])
        .expect("test certificate should be generated");
    let pem = certificate
        .serialize_pem()
        .expect("test certificate should serialize");
    let mut record = CertificateParser::parse_pem("expired.example.com", &pem)
        .expect("test certificate should parse");
    record.expire_time = Utc::now() - Duration::days(1);

    let result = CertificateValidator::default().validate_record(&record, "expired.example.com");

    assert!(result.is_err());
}
