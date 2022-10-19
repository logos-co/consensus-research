// std
// crates
use rand::rngs::SmallRng;
use serde::Serialize;
// internal
use crate::node::{query_network_state, ComputeNode, Decision, NetworkState, NoTx, NodeId};
use crate::output_processors::{NodeStateRecord, SerializedNodeState};
use claro::{ClaroSolver, ClaroState};

/// Claro consensus node
/// Wrapper over [`::claro::ClaroSolver`]
pub struct ClaroNode {
    solver: ClaroSolver<NoTx>,
    network_state: NetworkState,
    node_id: NodeId,
    rng: SmallRng,
}

impl ClaroNode {
    pub fn new(
        node_id: usize,
        solver: ClaroSolver<NoTx>,
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

impl ComputeNode for ClaroNode {
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
            self.solver.step(NoTx, &votes);
        }
    }

    fn decision(&self) -> Decision {
        self.solver.decision()
    }
}

#[derive(Serialize)]
struct OutClaroState {
    evidence: u64,
    evidence_accumulated: u64,
    confidence: u64,
}

impl From<&ClaroState> for OutClaroState {
    fn from(state: &ClaroState) -> Self {
        OutClaroState {
            evidence: state.evidence() as u64,
            evidence_accumulated: state.evidence_accumulated() as u64,
            confidence: state.confidence() as u64,
        }
    }
}

impl NodeStateRecord for ClaroNode {
    fn get_serialized_state_record(&self) -> SerializedNodeState {
        serde_json::to_value(OutClaroState::from(self.solver.state())).unwrap()
    }
}
