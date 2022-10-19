use crate::warding::{SimulationState, SimulationWard};
use serde::Deserialize;

/// Time to finality ward. It monitors the amount of rounds of the simulations, triggers when surpassing
/// the set threshold.
#[derive(Debug, Deserialize, Copy, Clone)]
pub struct TimeToFinalityWard {
    ttf_threshold: usize,
}

impl SimulationWard for TimeToFinalityWard {
    type SimulationState = SimulationState;
    fn analyze(&mut self, state: &SimulationState) -> bool {
        state.round > self.ttf_threshold
    }
}

#[cfg(test)]
mod test {
    use crate::node::NetworkState;
    use crate::warding::ttf::TimeToFinalityWard;
    use crate::warding::{SimulationState, SimulationWard};
    use std::sync::{Arc, RwLock};

    #[test]
    fn rebase_threshold() {
        let network_state = NetworkState::new(RwLock::new(vec![]));
        let mut ttf = TimeToFinalityWard { ttf_threshold: 10 };
        let mut cond = false;
        let mut state = SimulationState {
            network_state,
            nodes: Arc::new(Default::default()),
            iteration: 0,
            round: 0,
        };
        for _ in 0..11 {
            state.round += 1;
            cond = ttf.analyze(&state);
        }
        assert!(cond);
    }
}
