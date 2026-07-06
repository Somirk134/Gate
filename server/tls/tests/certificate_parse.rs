use gate_server_tls::certificate::{CertificateAlgorithm, CertificateParser, CertificateStatus};

#[test]
fn parses_certificate_model_from_pem() {
    let certificate = rcgen::generate_simple_self_signed(vec![
        "example.com".to_string(),
        "www.example.com".to_string(),
    ])
    .expect("test certificate should be generated");
    let pem = certificate
        .serialize_pem()
        .expect("test certificate should serialize");

    let record =
        CertificateParser::parse_pem("example.com", &pem).expect("test certificate should parse");

    assert_eq!(record.domain, "example.com");
    assert!(record.issuer.contains("CN=rcgen self signed cert"));
    assert_eq!(record.status, CertificateStatus::Active);
    assert!(record.expire_time > record.create_time);
    assert_eq!(record.fingerprint.sha256.len(), 64);
    assert!(record.san.contains(&"example.com".to_string()));
    assert!(record.san.contains(&"www.example.com".to_string()));
    assert!(matches!(
        record.algorithm,
        CertificateAlgorithm::EcdsaP256
            | CertificateAlgorithm::EcdsaP384
            | CertificateAlgorithm::Rsa
            | CertificateAlgorithm::Ed25519
    ));
}
