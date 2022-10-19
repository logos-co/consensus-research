// std
use std::sync::{Arc, RwLock};
// crates
use ::claro::ClaroSolver;
use rand::prelude::IteratorRandom;
use rand::rngs::SmallRng;
use rand::RngCore;
// internal
use crate::node::claro::ClaroNode;
use crate::node::infantile::InfantileNode;
pub use crate::node::omniscient::{MasterOmniscientNode, OmniscientPuppetNode};
use crate::node::random::RandomNode;
use crate::node::snowball::SnowballNode;
use crate::output_processors::NodeStateRecord;
use ::snowball::SnowballSolver;

mod claro;
mod infantile;
mod omniscient;
mod random;
mod snowball;

/// Consensus experiments consist on just one round, we just care about voting itself not the content
/// hence we need a Transaction that carries no information.
#[derive(Copy, Clone, Debug)]
pub struct NoTx;

/// NoTx vote
pub type Vote = ::claro::Vote<NoTx>;

/// NoTx decision
pub type Decision = ::claro::Decision<NoTx>;

/// NoTx opinion
pub type Opinion = ::claro::Opinion<NoTx>;

pub type NodeId = usize;

/// Shared hook to the simulation state
pub type NetworkState = Arc<RwLock<Vec<Option<Vote>>>>;

/// Node computation abstraction layer
pub trait ComputeNode {
    fn id(&self) -> usize;

    fn step(&mut self);

    fn vote(&self) -> Option<Vote> {
        self.opinion().into()
    }

    fn opinion(&self) -> Opinion {
        match self.decision() {
            Decision::Decided(opinion) | Decision::Undecided(opinion) => opinion,
        }
    }

    fn decision(&self) -> Decision;
}

/// Query the network state for a fixed size skipping self node id
pub fn query_network_state(
    network_state: &NetworkState,
    query_size: usize,
    node_id: NodeId,
    rng: &mut impl RngCore,
) -> Vec<Vote> {
    network_state
        .read()
        .unwrap()
        .iter()
        .enumerate()
        .choose_multiple(rng, query_size + 1)
        .into_iter()
        .filter_map(|(id, vote)| if id != node_id { *vote } else { None })
        .take(query_size)
        .collect()
}

/// Node dispatcher
/// Enum to avoid Boxing (Box<dyn ComputeNode>) the nodes.
pub enum Node {
    Snowball(snowball::SnowballNode),
    Claro(claro::ClaroNode),
    Random(random::RandomNode),
    Infantile(infantile::InfantileNode),
    OmniscientPuppet(omniscient::OmniscientPuppetNode),
}

impl Node {
    pub fn new_snowball(
        node_id: NodeId,
        solver: SnowballSolver<NoTx>,
        network_state: NetworkState,
        rng: SmallRng,
    ) -> Self {
        Self::Snowball(SnowballNode::new(node_id, solver, network_state, rng))
    }

    pub fn new_claro(
        node_id: NodeId,
        solver: ClaroSolver<NoTx>,
        network_state: NetworkState,
        seed: SmallRng,
    ) -> Self {
        Self::Claro(ClaroNode::new(node_id, solver, network_state, seed))
    }

    pub fn new_random(node_id: NodeId) -> Self {
        Self::Random(RandomNode::new(node_id))
    }

    pub fn new_infantile(
        node_id: NodeId,
        query_size: usize,
        network_state: NetworkState,
        rng: SmallRng,
    ) -> Self {
        Self::Infantile(InfantileNode::new(node_id, query_size, network_state, rng))
    }

    pub fn new_omniscient_puppet(puppet: OmniscientPuppetNode) -> Self {
        Self::OmniscientPuppet(puppet)
    }

    /// Get `ComputeNode` inner mut reference
    pub fn inner_node_mut(&mut self) -> &mut dyn ComputeNode {
        let node: &mut dyn ComputeNode = match self {
            Node::Snowball(node) => node,
            Node::Claro(node) => node,
            Node::Random(node) => node,
            Node::Infantile(node) => node,
            Node::OmniscientPuppet(node) => node,
        };
        node
    }

    /// Get `ComputeNode` inner reference
    pub fn inner_node(&self) -> &dyn ComputeNode {
        let node: &dyn ComputeNode = match self {
            Node::Snowball(node) => node,
            Node::Claro(node) => node,
            Node::Random(node) => node,
            Node::Infantile(node) => node,
            Node::OmniscientPuppet(node) => node,
        };
        node
    }

    pub fn serialized_state(&self) -> &dyn NodeStateRecord {
        match self {
            Node::Snowball(node) => node,
            Node::Claro(node) => node,
            Node::Random(node) => node,
            Node::Infantile(node) => node,
            Node::OmniscientPuppet(node) => node,
        }
    }

    pub fn type_as_string(&self) -> String {
        match self {
            Node::Snowball(_) => "snowball",
            Node::Claro(_) => "claro",
            Node::Random(_) => "random",
            Node::Infantile(_) => "infantile",
            Node::OmniscientPuppet(_) => "omniscient",
        }
        .to_string()
    }
}

impl ComputeNode for Node {
    fn id(&self) -> usize {
        self.inner_node().id()
    }

    fn step(&mut self) {
        self.inner_node_mut().step()
    }

    fn decision(&self) -> Decision {
        self.inner_node().decision()
    }
}
