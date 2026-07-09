use crate::config::ChallengeType;
use crate::error::AcmeError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use instant_acme::{
    Account, AuthorizationStatus, ChallengeType as InstantChallengeType, Identifier, NewAccount,
    NewOrder, OrderStatus, RetryPolicy,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[async_trait]
pub trait AcmeProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn directory_url(&self) -> &str;

    async fn create_account(&self, contact: AcmeAccountContact) -> Result<AcmeAccount, AcmeError>;

    async fn start_order(&self, request: AcmeCertificateRequest) -> Result<AcmeOrder, AcmeError>;

    async fn prepare_challenge(
        &self,
        order_id: &str,
        challenge_type: ChallengeType,
    ) -> Result<AcmeChallenge, AcmeError>;

    async fn finalize_order(&self, order_id: &str, csr_der: &[u8]) -> Result<AcmeOrder, AcmeError>;

    async fn download_certificate(&self, order_id: &str) -> Result<String, AcmeError>;

    async fn request_certificate_http01(
        &self,
        request: AcmeCertificateRequest,
        contact: AcmeAccountContact,
        challenge_store: &(dyn Http01ChallengeStore + Send + Sync),
    ) -> Result<AcmeIssuedCertificate, AcmeError> {
        let _ = (request, contact, challenge_store);
        Err(AcmeError::ProviderUnavailable {
            provider: self.name().to_string(),
        })
    }
}

#[async_trait]
pub trait Http01ChallengeStore: Send + Sync {
    async fn put_http01_challenge(
        &self,
        domain: &str,
        token: &str,
        key_authorization: &str,
    ) -> Result<(), AcmeError>;

    async fn remove_http01_challenge(&self, domain: &str, token: &str) -> Result<(), AcmeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetsEncryptProvider {
    directory_url: String,
}

impl Default for LetsEncryptProvider {
    fn default() -> Self {
        Self {
            directory_url: "https://acme-v02.api.letsencrypt.org/directory".to_string(),
        }
    }
}

impl LetsEncryptProvider {
    pub fn staging() -> Self {
        Self {
            directory_url: "https://acme-staging-v02.api.letsencrypt.org/directory".to_string(),
        }
    }

    pub fn with_directory_url(directory_url: impl Into<String>) -> Self {
        Self {
            directory_url: directory_url.into(),
        }
    }
}

#[async_trait]
impl AcmeProvider for LetsEncryptProvider {
    fn name(&self) -> &'static str {
        "letsencrypt"
    }

    fn directory_url(&self) -> &str {
        &self.directory_url
    }

    async fn create_account(&self, contact: AcmeAccountContact) -> Result<AcmeAccount, AcmeError> {
        let contact_uri = contact_uri(&contact);
        let contacts = contact_uri.iter().map(String::as_str).collect::<Vec<_>>();
        let (_, _) = Account::builder()
            .map_err(acme_execution)?
            .create(
                &NewAccount {
                    contact: contacts.as_slice(),
                    terms_of_service_agreed: true,
                    only_return_existing: false,
                },
                self.directory_url.clone(),
                None,
            )
            .await
            .map_err(acme_execution)?;

        Ok(AcmeAccount {
            id: format!("{}:{}", self.name(), Utc::now().timestamp_millis()),
            contact,
            created_at: Utc::now(),
        })
    }

    async fn start_order(&self, request: AcmeCertificateRequest) -> Result<AcmeOrder, AcmeError> {
        let (account, _) = Account::builder()
            .map_err(acme_execution)?
            .create(
                &NewAccount {
                    contact: &[],
                    terms_of_service_agreed: true,
                    only_return_existing: false,
                },
                self.directory_url.clone(),
                None,
            )
            .await
            .map_err(acme_execution)?;
        let identifiers = certificate_identifiers(&request);
        let mut order = account
            .new_order(&NewOrder::new(identifiers.as_slice()))
            .await
            .map_err(acme_execution)?;
        let status = format!("{:?}", order.state().status).to_ascii_lowercase();
        Ok(AcmeOrder {
            id: order.url().to_string(),
            domain: request.domain,
            status,
            expires_at: None,
        })
    }

    async fn prepare_challenge(
        &self,
        order_id: &str,
        challenge_type: ChallengeType,
    ) -> Result<AcmeChallenge, AcmeError> {
        Err(AcmeError::InvalidOrder {
            order_id: order_id.to_string(),
            reason: format!(
                "stateless challenge preparation for {challenge_type:?} is not supported; use request_certificate_http01"
            ),
        })
    }

    async fn finalize_order(
        &self,
        order_id: &str,
        _csr_der: &[u8],
    ) -> Result<AcmeOrder, AcmeError> {
        Err(AcmeError::InvalidOrder {
            order_id: order_id.to_string(),
            reason: "stateless order finalization is not supported; use request_certificate_http01"
                .to_string(),
        })
    }

    async fn download_certificate(&self, order_id: &str) -> Result<String, AcmeError> {
        Err(AcmeError::InvalidOrder {
            order_id: order_id.to_string(),
            reason: "download requires the live ACME order context; use request_certificate_http01"
                .to_string(),
        })
    }

    async fn request_certificate_http01(
        &self,
        request: AcmeCertificateRequest,
        contact: AcmeAccountContact,
        challenge_store: &(dyn Http01ChallengeStore + Send + Sync),
    ) -> Result<AcmeIssuedCertificate, AcmeError> {
        if request.challenge_type != ChallengeType::Http01 {
            return Err(AcmeError::UnsupportedChallenge {
                challenge: format!("{:?}", request.challenge_type),
            });
        }

        let contact_uri = contact_uri(&contact);
        let contacts = contact_uri.iter().map(String::as_str).collect::<Vec<_>>();
        let (account, _) = Account::builder()
            .map_err(acme_execution)?
            .create(
                &NewAccount {
                    contact: contacts.as_slice(),
                    terms_of_service_agreed: true,
                    only_return_existing: false,
                },
                self.directory_url.clone(),
                None,
            )
            .await
            .map_err(acme_execution)?;

        let identifiers = certificate_identifiers(&request);
        let mut order = account
            .new_order(&NewOrder::new(identifiers.as_slice()))
            .await
            .map_err(acme_execution)?;
        let order_id = order.url().to_string();

        let mut provisioned = Vec::new();
        let result = async {
            let mut authorizations = order.authorizations();
            while let Some(result) = authorizations.next().await {
                let mut authorization = result.map_err(acme_execution)?;
                match authorization.status {
                    AuthorizationStatus::Valid => continue,
                    AuthorizationStatus::Pending => {}
                    status => {
                        return Err(AcmeError::InvalidOrder {
                            order_id: order_id.clone(),
                            reason: format!("authorization status is {status:?}"),
                        });
                    }
                }

                let mut challenge = authorization
                    .challenge(InstantChallengeType::Http01)
                    .ok_or_else(|| AcmeError::ChallengeNotReady {
                        challenge: "http-01".to_string(),
                    })?;
                let domain = challenge.identifier().to_string();
                let token = challenge.token.clone();
                let key_authorization = challenge.key_authorization().as_str().to_string();
                challenge_store
                    .put_http01_challenge(&domain, &token, &key_authorization)
                    .await?;
                provisioned.push((domain, token));
                challenge.set_ready().await.map_err(acme_execution)?;
            }

            let status = order
                .poll_ready(&RetryPolicy::default())
                .await
                .map_err(acme_execution)?;
            if status != OrderStatus::Ready {
                return Err(AcmeError::InvalidOrder {
                    order_id,
                    reason: format!("unexpected order status: {status:?}"),
                });
            }

            let private_key_pem = order.finalize().await.map_err(acme_execution)?;
            let certificate_pem = order
                .poll_certificate(&RetryPolicy::default())
                .await
                .map_err(acme_execution)?;
            Ok(AcmeIssuedCertificate {
                domain: request.domain.clone(),
                certificate_pem,
                private_key_pem,
                issued_at: Utc::now(),
            })
        }
        .await;

        for (domain, token) in provisioned {
            let _ = challenge_store
                .remove_http01_challenge(&domain, &token)
                .await;
        }

        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeAccountContact {
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeAccount {
    pub id: String,
    pub contact: AcmeAccountContact,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeCertificateRequest {
    pub domain: String,
    pub san: Vec<String>,
    pub challenge_type: ChallengeType,
    pub preferred_chain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeOrder {
    pub id: String,
    pub domain: String,
    pub status: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeChallenge {
    pub order_id: String,
    pub challenge_type: ChallengeType,
    pub token: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeIssuedCertificate {
    pub domain: String,
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub issued_at: DateTime<Utc>,
}

fn contact_uri(contact: &AcmeAccountContact) -> Vec<String> {
    let email = contact.email.trim();
    if email.is_empty() {
        Vec::new()
    } else if email.starts_with("mailto:") {
        vec![email.to_string()]
    } else {
        vec![format!("mailto:{email}")]
    }
}

fn certificate_identifiers(request: &AcmeCertificateRequest) -> Vec<Identifier> {
    let mut domains = BTreeSet::new();
    domains.insert(request.domain.clone());
    for san in &request.san {
        domains.insert(san.clone());
    }

    domains
        .into_iter()
        .filter(|domain| !domain.trim().is_empty())
        .map(Identifier::Dns)
        .collect()
}

fn acme_execution(error: impl std::error::Error) -> AcmeError {
    AcmeError::Execution {
        provider: "letsencrypt".to_string(),
        reason: error.to_string(),
    }
}
