use crate::error::AcmeError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcmeLifecycleState {
    New,
    AccountReady,
    OrderCreated,
    ChallengePrepared,
    ChallengeValidated,
    Finalized,
    CertificateIssued,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcmeLifecycleEvent {
    AccountCreated,
    OrderStarted,
    ChallengePrepared,
    ChallengeValidated,
    OrderFinalized,
    CertificateDownloaded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeStateMachine {
    state: AcmeLifecycleState,
}

impl Default for AcmeStateMachine {
    fn default() -> Self {
        Self {
            state: AcmeLifecycleState::New,
        }
    }
}

impl AcmeStateMachine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> AcmeLifecycleState {
        self.state
    }

    pub fn apply(&mut self, event: AcmeLifecycleEvent) -> Result<AcmeLifecycleState, AcmeError> {
        let next = match (self.state, event) {
            (AcmeLifecycleState::New, AcmeLifecycleEvent::AccountCreated) => {
                AcmeLifecycleState::AccountReady
            }
            (AcmeLifecycleState::AccountReady, AcmeLifecycleEvent::OrderStarted) => {
                AcmeLifecycleState::OrderCreated
            }
            (AcmeLifecycleState::OrderCreated, AcmeLifecycleEvent::ChallengePrepared) => {
                AcmeLifecycleState::ChallengePrepared
            }
            (AcmeLifecycleState::ChallengePrepared, AcmeLifecycleEvent::ChallengeValidated) => {
                AcmeLifecycleState::ChallengeValidated
            }
            (AcmeLifecycleState::ChallengeValidated, AcmeLifecycleEvent::OrderFinalized) => {
                AcmeLifecycleState::Finalized
            }
            (AcmeLifecycleState::Finalized, AcmeLifecycleEvent::CertificateDownloaded) => {
                AcmeLifecycleState::CertificateIssued
            }
            (_, AcmeLifecycleEvent::Failed) => AcmeLifecycleState::Failed,
            (from, _) => {
                return Err(AcmeError::InvalidStateTransition {
                    from: format!("{from:?}"),
                    to: format!("{event:?}"),
                });
            }
        };

        self.state = next;
        Ok(next)
    }
}
