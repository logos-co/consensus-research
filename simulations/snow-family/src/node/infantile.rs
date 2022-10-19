use rand::rngs::SmallRng;
// std
// crates
// internal
use crate::node::{
    query_network_state, ComputeNode, Decision, NetworkState, NoTx, NodeId, Opinion, Vote,
};
use crate::output_processors::NodeStateRecord;

/// Node that replies with the opposite of the step query.
/// For each query:
///    if majority == yes: reply no
///    if majority == no: reply yes  
pub struct InfantileNode {
    network_state: NetworkState,
    query_size: usize,
    node_id: NodeId,
    decision: Decision,
    rng: SmallRng,
}

impl InfantileNode {
    pub fn new(
        node_id: usize,
        query_size: usize,
        network_state: NetworkState,
        rng: SmallRng,
    ) -> Self {
        let decision = Decision::Undecided(Opinion::None(NoTx));
        Self {
            node_id,
            query_size,
            network_state,
            decision,
            rng,
        }
    }

    fn flip_majority(votes: &[Vote]) -> Opinion {
        let yes_votes = votes
            .iter()
            .filter(|vote| matches!(vote, Vote::Yes(_)))
            .count();
        let len = votes.len();
        if yes_votes > len / 2 {
            Opinion::No(NoTx)
        } else {
            Opinion::Yes(NoTx)
        }
    }
}

impl ComputeNode for InfantileNode {
    fn id(&self) -> usize {
        self.node_id
    }

    fn step(&mut self) {
        let votes = query_network_state(
            &self.network_state,
            self.query_size,
            self.node_id,
            &mut self.rng,
        );
        self.decision = Decision::Undecided(InfantileNode::flip_majority(&votes));
    }

    fn decision(&self) -> Decision {
        self.decision
    }
}

impl NodeStateRecord for InfantileNode {}
