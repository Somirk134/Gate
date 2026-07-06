use gate_server_tls::cert_store::{CertificateStore, FileCertificateStore};
use gate_server_tls::certificate::{CertificateParser, StoredCertificate};

#[test]
fn file_store_saves_loads_lists_and_deletes_certificates() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let store = FileCertificateStore::new(temp_dir.path());
    let certificate = rcgen::generate_simple_self_signed(vec!["store.example.com".to_string()])
        .expect("test certificate should be generated");
    let certificate_pem = certificate
        .serialize_pem()
        .expect("test certificate should serialize");
    let private_key_pem = certificate.serialize_private_key_pem();
    let record = CertificateParser::parse_pem("store.example.com", &certificate_pem)
        .expect("test certificate should parse");
    let stored = StoredCertificate {
        record,
        certificate_pem,
        private_key_pem,
    };

    store.save(&stored).expect("certificate should save");

    assert!(store
        .contains("store.example.com")
        .expect("contains should succeed"));
    let loaded = store
        .load("store.example.com")
        .expect("certificate should load");
    assert_eq!(loaded.record.domain, "store.example.com");
    assert_eq!(loaded.certificate_pem, stored.certificate_pem);
    assert_eq!(store.list().expect("list should succeed").len(), 1);

    store
        .delete("store.example.com")
        .expect("certificate should delete");
    assert!(!store
        .contains("store.example.com")
        .expect("contains should succeed"));
}
