// std
use std::collections::HashSet;
// crates
use fixed_slice_deque::FixedSliceDeque;
use serde::{Deserialize, Deserializer};
// internal
use crate::node::{NetworkState, Vote};
use crate::warding::{SimulationState, SimulationWard};

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StabilisedCheck {
    Iterations {
        chunk: usize,
    },
    Rounds {
        #[serde(default, skip_deserializing)]
        last_round: usize,
    },
}

impl StabilisedCheck {
    pub fn should_check(&mut self, state: &SimulationState) -> bool {
        match self {
            StabilisedCheck::Iterations { chunk } => (state.iteration % *chunk) == 0,
            StabilisedCheck::Rounds { last_round } => {
                let different_round = *last_round < state.round;
                *last_round = state.round;
                different_round
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct StabilisedWard {
    #[serde(deserialize_with = "deserialize_fixed_slice_from_usize")]
    buffer: FixedSliceDeque<(usize, usize)>,
    check: StabilisedCheck,
}

impl StabilisedWard {
    fn is_stabilised(&self) -> bool {
        if self.buffer.is_full() {
            let set: HashSet<_> = self.buffer.iter().copied().collect();
            return set.len() == 1;
        }
        false
    }

    fn count_state(network_state: NetworkState) -> (usize, usize) {
        network_state
            .read()
            .unwrap()
            .iter()
            .fold((0, 0), |count @ (yes, no), vote| match vote {
                None => count,
                Some(Vote::Yes(_)) => (yes + 1, no),
                Some(Vote::No(_)) => (yes, no + 1),
            })
    }
}

impl SimulationWard for StabilisedWard {
    type SimulationState = SimulationState;

    fn analyze(&mut self, state: &Self::SimulationState) -> bool {
        if !self.check.should_check(state) {
            return false;
        }
        self.buffer
            .push_back(StabilisedWard::count_state(state.network_state.clone()));
        self.is_stabilised()
    }
}

fn deserialize_fixed_slice_from_usize<'d, T, D: Deserializer<'d>>(
    d: D,
) -> Result<FixedSliceDeque<T>, D::Error> {
    let value = usize::deserialize(d)?;
    Ok(FixedSliceDeque::new(value))
}

#[cfg(test)]
mod tests {
    use crate::node::{NoTx, Vote};
    use crate::warding::stabilised::{StabilisedCheck, StabilisedWard};
    use crate::warding::{SimulationState, SimulationWard};
    use fixed_slice_deque::FixedSliceDeque;
    use std::sync::{Arc, RwLock};

    #[test]
    fn check_rounds() {
        let mut ward = StabilisedWard {
            buffer: FixedSliceDeque::new(2),
            check: StabilisedCheck::Rounds { last_round: 0 },
        };

        let mut simulation_state = SimulationState {
            network_state: Arc::new(RwLock::new(vec![Some(Vote::Yes(NoTx))])),
            nodes: Arc::new(RwLock::new(vec![])),
            iteration: 0,
            round: 0,
        };

        for i in 0..2 {
            simulation_state.round = i;
            assert!(!ward.analyze(&simulation_state));
        }

        simulation_state.round = 3;
        assert!(ward.analyze(&simulation_state));
    }

    #[test]
    fn check_iterations() {
        let mut ward = StabilisedWard {
            buffer: FixedSliceDeque::new(2),
            check: StabilisedCheck::Iterations { chunk: 3 },
        };

        let mut simulation_state = SimulationState {
            network_state: Arc::new(RwLock::new(vec![Some(Vote::Yes(NoTx))])),
            nodes: Arc::new(RwLock::new(vec![])),
            iteration: 0,
            round: 0,
        };

        for i in 0..3 {
            simulation_state.iteration = i;
            assert!(!ward.analyze(&simulation_state));
        }

        simulation_state.iteration = 3;
        assert!(ward.analyze(&simulation_state));
    }

    #[test]
    fn deserialize() {
        let rounds = r#"{ "buffer" : 3, "check" : { "type": "rounds" } }"#;
        let iterations = r#"{ "buffer" : 3, "check" : { "type": "iterations", "chunk":  100 } }"#;
        for s in [rounds, iterations] {
            let ward: StabilisedWard =
                serde_json::from_str(s).expect("Should deserialize correctly");
            assert_eq!(ward.buffer.capacity(), 3);
        }
    }
}
