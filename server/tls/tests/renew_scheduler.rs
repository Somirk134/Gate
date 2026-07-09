use chrono::{Duration, Utc};
use gate_server_tls::certificate::{
    CertificateAlgorithm, CertificateFingerprint, CertificateRecord, CertificateStatus,
};
use gate_server_tls::renew::{CertificateRenewer, RenewAttemptStatus, RenewConfig, RenewScheduler};
use std::collections::HashSet;

#[test]
fn scheduler_marks_certificates_inside_renew_window_without_executing() {
    let now = Utc::now();
    let scheduler = RenewScheduler::new(RenewConfig::default());
    let due = certificate_record("due.example.com", now + Duration::days(10));
    let later = certificate_record("later.example.com", now + Duration::days(90));

    let plan = scheduler.plan(&[due, later], now);

    assert_eq!(plan.next_check_at, now + Duration::days(1));
    assert_eq!(plan.decisions.len(), 2);
    assert!(plan.decisions[0].should_renew);
    assert!(!plan.decisions[1].should_renew);
}

#[test]
fn disabled_scheduler_never_marks_renewal() {
    let now = Utc::now();
    let scheduler = RenewScheduler::new(RenewConfig {
        enabled: false,
        ..RenewConfig::default()
    });
    let due = certificate_record("disabled.example.com", now + Duration::days(1));

    let plan = scheduler.plan(&[due], now);

    assert!(!plan.decisions[0].should_renew);
    assert_eq!(plan.decisions[0].reason, "renew scheduler is disabled");
}

#[tokio::test]
async fn scheduler_executes_due_renewals_and_records_failures() {
    let now = Utc::now();
    let scheduler = RenewScheduler::new(RenewConfig::default());
    let due = certificate_record("due.example.com", now + Duration::days(10));
    let failing = certificate_record("fail.example.com", now + Duration::days(2));
    let later = certificate_record("later.example.com", now + Duration::days(90));
    let renewer = SimulatedRenewer {
        failures: HashSet::from(["fail.example.com".to_string()]),
    };

    let report = scheduler
        .execute_due(&[due, failing, later], now, &renewer)
        .await;

    assert_eq!(report.attempts.len(), 2);
    assert_eq!(report.attempts[0].status, RenewAttemptStatus::Success);
    assert_eq!(report.attempts[1].status, RenewAttemptStatus::Failed);
    assert!(report.attempts[1].error.is_some());
}

struct SimulatedRenewer {
    failures: HashSet<String>,
}

#[async_trait::async_trait]
impl CertificateRenewer for SimulatedRenewer {
    async fn renew_certificate(&self, domain: &str) -> Result<(), String> {
        if self.failures.contains(domain) {
            Err("simulated renewal failure".to_string())
        } else {
            Ok(())
        }
    }
}

fn certificate_record(domain: &str, expire_time: chrono::DateTime<Utc>) -> CertificateRecord {
    CertificateRecord {
        domain: domain.to_string(),
        issuer: "test issuer".to_string(),
        expire_time,
        create_time: Utc::now() - Duration::days(1),
        renew_time: None,
        status: CertificateStatus::Active,
        fingerprint: CertificateFingerprint {
            sha256: "0".repeat(64),
        },
        algorithm: CertificateAlgorithm::EcdsaP256,
        san: vec![domain.to_string()],
        cert_path: None,
        key_path: None,
        serial_number: Some("01".to_string()),
        last_error: None,
    }
}
