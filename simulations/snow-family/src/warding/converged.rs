use crate::node::{ComputeNode, Decision, Node};
use crate::warding::{SimulationState, SimulationWard};
use serde::de::Error;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct ConvergedWard {
    #[serde(deserialize_with = "deserialize_normalized_value")]
    ratio: f32,
}

impl ConvergedWard {
    pub fn converged(&self, len: usize, decisions: impl Iterator<Item = Decision>) -> bool {
        let total_decided = decisions
            .filter(|decision| matches!(decision, Decision::Decided(_)))
            .count();

        (total_decided as f32 / len as f32) >= self.ratio
    }
}

impl SimulationWard for ConvergedWard {
    type SimulationState = SimulationState;

    fn analyze(&mut self, state: &Self::SimulationState) -> bool {
        let nodes = state.nodes.read().expect("Read access to nodes vec");
        self.converged(nodes.len(), nodes.iter().map(Node::decision))
    }
}

// TODO: Probably a good idea to have a serde_utils crate
fn deserialize_normalized_value<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = f32::deserialize(deserializer)?;
    (0f32..=1f32)
        .contains(&value)
        .then_some(value)
        .ok_or_else(|| {
            D::Error::custom(&format!(
                "Only normalized values [0.0, 1.0] are valid, got: {}",
                value
            ))
        })
}

#[cfg(test)]
mod tests {
    use crate::node::NoTx;
    use crate::warding::converged::ConvergedWard;
    use claro::{Decision, Opinion};

    #[test]
    fn converge_full() {
        let decisions = vec![
            Decision::Decided(Opinion::Yes(NoTx)),
            Decision::Decided(Opinion::Yes(NoTx)),
        ];
        let ward = ConvergedWard { ratio: 1.0 };

        assert!(ward.converged(2, decisions.into_iter()));
    }

    #[test]
    fn converge_ratio() {
        let decisions = vec![
            Decision::Decided(Opinion::Yes(NoTx)),
            Decision::Decided(Opinion::Yes(NoTx)),
            Decision::Undecided(Opinion::Yes(NoTx)),
        ];
        let ward = ConvergedWard { ratio: 0.5 };

        assert!(ward.converged(2, decisions.into_iter()));
    }

    #[test]
    fn not_converge() {
        let decisions = vec![
            Decision::Decided(Opinion::Yes(NoTx)),
            Decision::Undecided(Opinion::Yes(NoTx)),
        ];
        let ward = ConvergedWard { ratio: 1.0 };

        assert!(!ward.converged(2, decisions.into_iter()));
    }
}
