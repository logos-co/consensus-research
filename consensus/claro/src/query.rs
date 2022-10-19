use crate::claro::Vote;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use tracing::debug;

// TODO: Check on proper types
/// Node ids type
pub type NodeId = String;
/// Node weight alias
/// Refers to amount of staking a node holds
pub type NodeWeight = f64;

/// Node ids <=> weights sampling information trait
pub trait NodesSample {
    fn nodes(&self) -> Vec<NodeId>;
    fn weights(&self) -> HashMap<&NodeId, NodeWeight>;
}

/// Selector of nodes, random sample for some size `K`
#[derive(Debug, Clone)]
pub struct NodeQuery {
    node_size: usize,
    node_id: NodeId,
}

impl NodeQuery {
    pub fn new(node_size: usize, node_id: NodeId) -> Self {
        Self { node_size, node_id }
    }

    pub fn query_size(&self) -> usize {
        self.node_size
    }

    pub fn node_id(&self) -> &NodeId {
        &self.node_id
    }

    pub fn sample<Sample: NodesSample>(&self, node_sample: &Sample) -> Vec<NodeId> {
        let node_ids = node_sample.nodes();
        let weights = node_sample.weights();
        // TODO: do we need to be reproducible?
        let mut rng = thread_rng();
        let node_ids = node_ids
            .as_slice()
            .choose_multiple_weighted(&mut rng, self.node_size + 1, |e| *weights.get(e).unwrap())
            .unwrap()
            .cloned()
            .filter(|node_id| node_id != &self.node_id)
            .take(self.node_size)
            .collect();
        debug!(query_node_ids = ?node_ids);
        node_ids
    }
}

/// Communication layer abstraction trait
/// Used by the claro algorithm runner to query for the votes of other nodes
#[async_trait::async_trait]
pub trait VoteQuery: Send + Sync {
    type Tx;
    async fn query(&mut self, node_query: &NodeQuery, tx: Self::Tx) -> Vec<Vote<Self::Tx>>;
}

#[cfg(test)]
mod test {
    use crate::query::{NodeId, NodeQuery, NodeWeight, NodesSample};
    use std::collections::{HashMap, HashSet};

    struct TestSample {
        node_ids: Vec<NodeId>,
        node_weights: Vec<NodeWeight>,
    }

    impl TestSample {
        fn len(&self) -> usize {
            assert_eq!(self.node_weights.len(), self.node_ids.len());
            self.node_ids.len()
        }
    }

    impl NodesSample for TestSample {
        fn nodes(&self) -> Vec<NodeId> {
            self.node_ids.clone()
        }

        fn weights(&self) -> HashMap<&NodeId, NodeWeight> {
            self.node_ids
                .iter()
                .zip(self.node_weights.iter().copied())
                .collect()
        }
    }

    #[test]
    fn unique_sample_set() {
        let query: NodeQuery = NodeQuery::new(10, "".into());
        let sample = TestSample {
            node_ids: (0..10).map(|i| i.to_string()).collect(),
            node_weights: (1..11usize).map(|i| i as f64).collect(),
        };

        let ids: HashSet<_> = query.sample(&sample).into_iter().collect();
        assert_eq!(ids.len(), sample.len());
    }
}
