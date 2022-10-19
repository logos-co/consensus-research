// std
use std::sync::{Arc, RwLock};
// crates
// internal
use crate::node::{ComputeNode, Decision, NetworkState, NoTx, NodeId, Opinion, Vote};
use crate::output_processors::NodeStateRecord;

/// Node that knows the network state all the time.
/// It orchestrates responses based on that.
/// As an optimization just a single node takes care of everything, then we place Puppet nodes
/// in the list of nodes that just replies with whatever the Master omniscient node decides.
pub struct MasterOmniscientNode {
    honest_nodes_ids: Vec<NodeId>,
    omniscient_nodes_ids: Vec<NodeId>,
    network_state: NetworkState,
    decision: Arc<RwLock<Decision>>,
    node_id: NodeId,
}

/// Omniscient puppet node. Node that just replies with whatever the `MasterOmniscientNode` decides.
#[derive(Clone)]
pub struct OmniscientPuppetNode {
    node_id: NodeId,
    decision: Arc<RwLock<Decision>>,
}

impl MasterOmniscientNode {
    pub fn new(
        node_id: NodeId,
        honest_nodes_ids: Vec<NodeId>,
        omniscient_nodes_ids: Vec<NodeId>,
        network_state: NetworkState,
    ) -> Self {
        Self {
            node_id,
            honest_nodes_ids,
            omniscient_nodes_ids,
            network_state,
            decision: Arc::new(RwLock::new(Decision::Undecided(Opinion::None(NoTx)))),
        }
    }

    fn analyze_and_write_votes(&mut self) {
        let mut state = self
            .network_state
            .write()
            .expect("Only access to network state resource from omniscient node");

        let honest_votes: Vec<Option<Vote>> = self
            .honest_nodes_ids
            .iter()
            .map(|node_id| state.get(*node_id).expect("Node id should be within range"))
            .copied()
            .collect();

        let yes_votes = honest_votes
            .iter()
            .filter(|v| matches!(v, Some(Vote::Yes(_))))
            .count();
        let no_votes = honest_votes
            .iter()
            .filter(|v| matches!(v, Some(Vote::No(_))))
            .count();

        let vote = if yes_votes > no_votes {
            *self.decision.write().unwrap() = Decision::Undecided(Opinion::No(NoTx));
            Some(Vote::No(NoTx))
        } else {
            *self.decision.write().unwrap() = Decision::Undecided(Opinion::Yes(NoTx));
            Some(Vote::Yes(NoTx))
        };

        for &i in &self.omniscient_nodes_ids {
            if let Some(old_vote) = state.get_mut(i) {
                *old_vote = vote;
            }
        }
    }

    pub fn puppet_node(&self, node_id: NodeId) -> OmniscientPuppetNode {
        OmniscientPuppetNode {
            node_id,
            decision: Arc::clone(&self.decision),
        }
    }
}

impl ComputeNode for MasterOmniscientNode {
    fn id(&self) -> usize {
        self.node_id
    }

    fn step(&mut self) {
        self.analyze_and_write_votes();
    }

    fn decision(&self) -> Decision {
        *self.decision.read().unwrap()
    }
}

impl ComputeNode for OmniscientPuppetNode {
    fn id(&self) -> usize {
        self.node_id
    }

    fn step(&mut self) {}

    fn decision(&self) -> Decision {
        *self.decision.read().unwrap()
    }
}

impl NodeStateRecord for OmniscientPuppetNode {}
