use super::{
    AcmeCertificateRequest, AcmeLifecycleEvent, AcmeLifecycleState, AcmeProvider, AcmeStateMachine,
};
use crate::error::AcmeError;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AcmeClient<P> {
    provider: P,
    state_machine: AcmeStateMachine,
}

impl<P> AcmeClient<P>
where
    P: AcmeProvider,
{
    pub fn new(provider: P) -> Self {
        Self {
            provider,
            state_machine: AcmeStateMachine::new(),
        }
    }

    pub fn provider(&self) -> &P {
        &self.provider
    }

    pub fn state(&self) -> AcmeLifecycleState {
        self.state_machine.state()
    }

    pub fn plan_lifecycle(&self, request: AcmeCertificateRequest) -> AcmeLifecyclePlan {
        AcmeLifecyclePlan {
            provider: self.provider.name().to_string(),
            domain: request.domain,
            steps: vec![
                AcmeLifecycleState::New,
                AcmeLifecycleState::AccountReady,
                AcmeLifecycleState::OrderCreated,
                AcmeLifecycleState::ChallengePrepared,
                AcmeLifecycleState::ChallengeValidated,
                AcmeLifecycleState::Finalized,
                AcmeLifecycleState::CertificateIssued,
            ],
        }
    }

    pub fn mark_account_ready(&mut self) -> Result<AcmeLifecycleState, AcmeError> {
        self.state_machine.apply(AcmeLifecycleEvent::AccountCreated)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcmeLifecyclePlan {
    pub provider: String,
    pub domain: String,
    pub steps: Vec<AcmeLifecycleState>,
}
