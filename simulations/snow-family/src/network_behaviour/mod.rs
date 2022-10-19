mod drop;

use crate::node::Vote;
use rand::rngs::SmallRng;
use serde::Deserialize;

/// Modify a ['crate::node::NetworkState'](network state), single exclusive access is guaranteed
pub trait NetworkBehaviour {
    fn modify_network_state(&mut self, network_state: &mut [Option<Vote>], rng: &mut SmallRng);
}

/// [`NetworkBehaviour`] dispatcher
/// Enum to avoid Boxing (Box<dyn NetworkBehaviour>) modifiers.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkModifiers {
    RandomDrop(drop::RandomDrop),
}

impl NetworkModifiers {
    /// Get inner [`NetworkBehaviour`] mut reference
    pub fn network_behaviour_mut(&mut self) -> &mut dyn NetworkBehaviour {
        match self {
            NetworkModifiers::RandomDrop(behaviour) => behaviour,
        }
    }
}

impl NetworkBehaviour for NetworkModifiers {
    fn modify_network_state(&mut self, network_state: &mut [Option<Vote>], rng: &mut SmallRng) {
        self.network_behaviour_mut()
            .modify_network_state(network_state, rng);
    }
}
