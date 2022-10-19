// std
// crates
// internal
use crate::node::{ComputeNode, Decision, NoTx, NodeId, Opinion};
use crate::output_processors::NodeStateRecord;

/// Nodes that takes a random decision each step
pub struct RandomNode {
    decision: Decision,
    node_id: NodeId,
}

impl RandomNode {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            decision: Decision::Undecided(Opinion::None(NoTx)),
            node_id,
        }
    }

    fn rand_opinion() -> Opinion {
        let bool_opinion: bool = rand::random();
        if bool_opinion {
            Opinion::Yes(NoTx)
        } else {
            Opinion::No(NoTx)
        }
    }
}

impl ComputeNode for RandomNode {
    fn id(&self) -> usize {
        self.node_id
    }

    fn step(&mut self) {
        self.decision = Decision::Undecided(RandomNode::rand_opinion());
    }

    fn decision(&self) -> Decision {
        self.decision
    }
}

impl NodeStateRecord for RandomNode {}
