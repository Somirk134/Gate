use serde::{Deserialize, Serialize};

/// Declared connection simulation scales for Alpha V1 performance testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulationScale {
    C100,
    C500,
    C1000,
    C5000,
}

impl SimulationScale {
    pub fn connections(self) -> usize {
        match self {
            Self::C100 => 100,
            Self::C500 => 500,
            Self::C1000 => 1000,
            Self::C5000 => 5000,
        }
    }
}

/// Non-executing performance simulation plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionSimulationPlan {
    pub scale: SimulationScale,
    pub connections: usize,
    pub ramp_up_seconds: u64,
    pub hold_seconds: u64,
    pub execute_load: bool,
}

impl ConnectionSimulationPlan {
    pub fn new(scale: SimulationScale) -> Self {
        Self {
            scale,
            connections: scale.connections(),
            ramp_up_seconds: 30,
            hold_seconds: 120,
            execute_load: false,
        }
    }

    pub fn alpha_matrix() -> Vec<Self> {
        [
            SimulationScale::C100,
            SimulationScale::C500,
            SimulationScale::C1000,
            SimulationScale::C5000,
        ]
        .into_iter()
        .map(Self::new)
        .collect()
    }
}
