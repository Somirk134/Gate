use crate::certificate::{CertificateRecord, CertificateStatus};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenewConfig {
    pub enabled: bool,
    pub check_interval_seconds: i64,
    pub renew_before_days: i64,
}

impl Default for RenewConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_seconds: 24 * 60 * 60,
            renew_before_days: 30,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenewScheduler {
    config: RenewConfig,
}

impl Default for RenewScheduler {
    fn default() -> Self {
        Self::new(RenewConfig::default())
    }
}

impl RenewScheduler {
    pub fn new(config: RenewConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> RenewConfig {
        self.config
    }

    pub fn next_check_after(&self, from: DateTime<Utc>) -> DateTime<Utc> {
        from + Duration::seconds(self.config.check_interval_seconds)
    }

    pub fn plan(&self, certificates: &[CertificateRecord], now: DateTime<Utc>) -> RenewPlan {
        let threshold = now + Duration::days(self.config.renew_before_days);
        let mut decisions = Vec::with_capacity(certificates.len());

        for certificate in certificates {
            let should_renew = self.config.enabled
                && certificate.status != CertificateStatus::Deleted
                && certificate.status != CertificateStatus::Revoked
                && certificate.expire_time <= threshold;

            decisions.push(RenewDecision {
                domain: certificate.domain.clone(),
                expires_at: certificate.expire_time,
                should_renew,
                reason: if should_renew {
                    "certificate is inside renewal window".to_string()
                } else if !self.config.enabled {
                    "renew scheduler is disabled".to_string()
                } else {
                    "certificate is outside renewal window".to_string()
                },
            });
        }

        RenewPlan {
            checked_at: now,
            next_check_at: self.next_check_after(now),
            decisions,
        }
    }

    pub async fn execute_due<R>(
        &self,
        certificates: &[CertificateRecord],
        now: DateTime<Utc>,
        renewer: &R,
    ) -> RenewExecutionReport
    where
        R: CertificateRenewer,
    {
        let plan = self.plan(certificates, now);
        let mut attempts = Vec::new();

        for decision in plan
            .decisions
            .iter()
            .filter(|decision| decision.should_renew)
        {
            // 续期只触发到期窗口内证书，具体 ACME/存储细节由调用方注入。
            match renewer.renew_certificate(&decision.domain).await {
                Ok(()) => attempts.push(RenewAttempt {
                    domain: decision.domain.clone(),
                    status: RenewAttemptStatus::Success,
                    error: None,
                }),
                Err(error) => attempts.push(RenewAttempt {
                    domain: decision.domain.clone(),
                    status: RenewAttemptStatus::Failed,
                    error: Some(error),
                }),
            }
        }

        RenewExecutionReport { plan, attempts }
    }
}

#[async_trait]
pub trait CertificateRenewer: Send + Sync {
    async fn renew_certificate(&self, domain: &str) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenewPlan {
    pub checked_at: DateTime<Utc>,
    pub next_check_at: DateTime<Utc>,
    pub decisions: Vec<RenewDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenewDecision {
    pub domain: String,
    pub expires_at: DateTime<Utc>,
    pub should_renew: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenewExecutionReport {
    pub plan: RenewPlan,
    pub attempts: Vec<RenewAttempt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenewAttempt {
    pub domain: String,
    pub status: RenewAttemptStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenewAttemptStatus {
    Success,
    Failed,
}
