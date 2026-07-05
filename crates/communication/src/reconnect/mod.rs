//! Retry policy reserved for reconnect and request retry workflows.

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetryPolicy {
    None,
    Linear {
        initial_delay_ms: u64,
        max_delay_ms: u64,
        max_attempts: u32,
    },
    Exponential {
        base_delay_ms: u64,
        max_delay_ms: u64,
        factor: f64,
        max_attempts: u32,
    },
    Custom {
        delays_ms: Vec<u64>,
    },
}

impl RetryPolicy {
    pub fn delay_for_attempt(&self, attempt: u32) -> Option<Duration> {
        if attempt == 0 {
            return None;
        }

        match self {
            Self::None => None,
            Self::Linear {
                initial_delay_ms,
                max_delay_ms,
                max_attempts,
            } => {
                if attempt > *max_attempts {
                    return None;
                }

                Some(Duration::from_millis((*initial_delay_ms).min(*max_delay_ms)))
            }
            Self::Exponential {
                base_delay_ms,
                max_delay_ms,
                factor,
                max_attempts,
            } => {
                if attempt > *max_attempts {
                    return None;
                }

                let exponent = attempt.saturating_sub(1) as i32;
                let delay = (*base_delay_ms as f64 * factor.powi(exponent)) as u64;
                Some(Duration::from_millis(delay.min(*max_delay_ms)))
            }
            Self::Custom { delays_ms } => delays_ms
                .get(attempt.saturating_sub(1) as usize)
                .copied()
                .map(Duration::from_millis),
        }
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        self.delay_for_attempt(attempt).is_some()
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::Exponential {
            base_delay_ms: 500,
            max_delay_ms: 30_000,
            factor: 2.0,
            max_attempts: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryContext {
    pub attempt: u32,
    pub last_error: Option<String>,
}

impl RetryContext {
    pub fn first() -> Self {
        Self {
            attempt: 1,
            last_error: None,
        }
    }
}
