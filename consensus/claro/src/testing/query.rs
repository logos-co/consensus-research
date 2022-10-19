use crate::{NodeQuery, Vote, VoteQuery};
use std::marker::PhantomData;

pub struct NoQuery<Tx>(PhantomData<Tx>);

impl<Tx> Default for NoQuery<Tx> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[async_trait::async_trait]
impl<Tx: Send + Sync> VoteQuery for NoQuery<Tx> {
    type Tx = Tx;

    async fn query(&mut self, _node_query: &NodeQuery, _tx: Self::Tx) -> Vec<Vote<Self::Tx>> {
        vec![]
    }
}

pub struct FixedQuery<Tx: Clone + Send + Sync>(Vote<Tx>);

impl<Tx: Clone + Send + Sync> FixedQuery<Tx> {
    pub fn new(vote: Vote<Tx>) -> Self {
        Self(vote)
    }
}

#[async_trait::async_trait]
impl<Tx: Clone + Send + Sync> VoteQuery for FixedQuery<Tx> {
    type Tx = Tx;

    async fn query(&mut self, node_query: &NodeQuery, _tx: Self::Tx) -> Vec<Vote<Self::Tx>> {
        (0..node_query.query_size())
            .map(|_| self.0.clone())
            .collect()
    }
}
