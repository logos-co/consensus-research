use rand::rngs::SmallRng;
// std
// crates
use serde::Serialize;
// internal
use crate::node::{query_network_state, ComputeNode, Decision, NetworkState, NoTx, NodeId};
use crate::output_processors::{NodeStateRecord, SerializedNodeState};
use snowball::SnowballSolver;

/// Snowball consensus node
/// Wrapper over [`::snowball::SnowballSolver`]
pub struct SnowballNode {
    solver: SnowballSolver<NoTx>,
    network_state: NetworkState,
    node_id: NodeId,
    rng: SmallRng,
}

impl SnowballNode {
    pub fn new(
        node_id: usize,
        solver: SnowballSolver<NoTx>,
        network_state: NetworkState,
        rng: SmallRng,
    ) -> Self {
        Self {
            node_id,
            solver,
            network_state,
            rng,
        }
    }
}

impl ComputeNode for SnowballNode {
    fn id(&self) -> usize {
        self.node_id
    }

    fn step(&mut self) {
        if matches!(self.solver.decision(), Decision::Undecided(_)) {
            let votes = query_network_state(
                &self.network_state,
                self.solver.node_query().query_size(),
                self.node_id,
                &mut self.rng,
            );
            self.solver.step(&votes);
        }
    }

    fn decision(&self) -> Decision {
        self.solver.decision()
    }
}

#[derive(Serialize)]
struct OutSnowballState {
    consecutive_success: u64,
}

impl NodeStateRecord for SnowballNode {
    fn get_serialized_state_record(&self) -> SerializedNodeState {
        let consecutive_success = self.solver.consecutive_success();
        serde_json::to_value(OutSnowballState {
            consecutive_success,
        })
        .unwrap()
    }
}
