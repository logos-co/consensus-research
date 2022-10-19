use crate::node::{NetworkState, Node};
use serde::Deserialize;
use std::sync::{Arc, RwLock};

mod converged;
mod stabilised;
mod ttf;

pub struct SimulationState {
    pub network_state: NetworkState,
    pub nodes: Arc<RwLock<Vec<Node>>>,
    pub iteration: usize,
    pub round: usize,
}

/// A ward is a computation over the `NetworkState`, it must return true if the state satisfies
/// the warding conditions. It is used to stop the consensus simulation if such condition is reached.
pub trait SimulationWard {
    type SimulationState;
    fn analyze(&mut self, state: &Self::SimulationState) -> bool;
}

/// Ward dispatcher
/// Enum to avoid Boxing (Box<dyn SimulationWard>) wards.
#[derive(Debug, Deserialize)]
pub enum Ward {
    #[serde(rename = "time_to_finality")]
    Ttf(ttf::TimeToFinalityWard),
    #[serde(rename = "stabilised")]
    Stabilised(stabilised::StabilisedWard),
    #[serde(rename = "converged")]
    Converged(converged::ConvergedWard),
}

impl Ward {
    pub fn simulation_ward_mut(
        &mut self,
    ) -> &mut dyn SimulationWard<SimulationState = SimulationState> {
        match self {
            Ward::Ttf(ward) => ward,
            Ward::Stabilised(stabilised) => stabilised,
            Ward::Converged(converged) => converged,
        }
    }
}

impl SimulationWard for Ward {
    type SimulationState = SimulationState;
    fn analyze(&mut self, state: &Self::SimulationState) -> bool {
        self.simulation_ward_mut().analyze(state)
    }
}
