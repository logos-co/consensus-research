// std
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use tracing::debug;
// crates
// internal
use crate::query::NodeQuery;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Vote<Tx> {
    Yes(Tx),
    No(Tx),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Opinion<Tx> {
    None(Tx),
    Yes(Tx),
    No(Tx),
}

impl<Tx> Opinion<Tx> {
    pub fn flip(self) -> Self {
        match self {
            Opinion::Yes(tx) => Opinion::No(tx),
            Opinion::No(tx) => Opinion::Yes(tx),
            none => none,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Decision<Tx> {
    Decided(Opinion<Tx>),
    Undecided(Opinion<Tx>),
}

impl<Tx> Display for Opinion<Tx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tag = match self {
            Opinion::Yes(_) => "yes",
            Opinion::No(_) => "no",
            Opinion::None(_) => "none",
        };
        write!(f, "{}", tag)
    }
}

impl<Tx> Display for Decision<Tx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tag = match self {
            Decision::Decided(_) => "decided",
            Decision::Undecided(_) => "undecided",
        };
        write!(f, "{}", tag)
    }
}

impl<Tx> From<Opinion<Tx>> for Option<Vote<Tx>> {
    fn from(opinion: Opinion<Tx>) -> Self {
        match opinion {
            Opinion::Yes(tx) => Some(Vote::Yes(tx)),
            Opinion::No(tx) => Some(Vote::No(tx)),
            Opinion::None(_) => None,
        }
    }
}

impl<Tx> From<Vote<Tx>> for Opinion<Tx> {
    fn from(vote: Vote<Tx>) -> Self {
        match vote {
            Vote::Yes(tx) => Opinion::Yes(tx),
            Vote::No(tx) => Opinion::No(tx),
        }
    }
}

impl<Tx> From<Decision<Tx>> for Option<Vote<Tx>> {
    fn from(decision: Decision<Tx>) -> Self {
        match decision {
            Decision::Decided(opinion) | Decision::Undecided(opinion) => opinion.into(),
        }
    }
}

#[allow(dead_code)]
/// Claro round computed evidence, confidence and alpha
pub struct ClaroRoundCalculation {
    confidence: f32,
    e1: f32,
    e2: f32,
    e: f32,
    alpha: f32,
}

/// Claro internal state
#[derive(Default, Debug)]
pub struct ClaroState {
    /// Positive votes seen
    evidence: usize,
    /// Total votes seen, positive and negative
    evidence_accumulated: usize,
    /// Votes ratio
    confidence: usize,
}

impl ClaroState {
    pub fn update_confidence<Tx>(&mut self, votes: &[Vote<Tx>]) {
        let total_votes = votes.len();
        self.confidence = self.confidence.saturating_add(total_votes);
    }

    pub fn update_evidence<Tx>(&mut self, votes: &[Vote<Tx>]) {
        let total_votes = votes.len();
        let total_yes = votes.iter().filter(|v| matches!(v, Vote::Yes(_))).count();
        self.evidence = self.evidence.saturating_add(total_yes);
        self.evidence_accumulated = self.evidence_accumulated.saturating_add(total_votes);
    }

    pub fn confidence(&self) -> usize {
        self.confidence
    }

    pub fn evidence(&self) -> usize {
        self.evidence
    }

    pub fn evidence_accumulated(&self) -> usize {
        self.evidence_accumulated
    }
}

/// Node query configuration
#[derive(Debug, Clone, Copy)]
pub struct QueryConfiguration {
    /// How many nodes to query
    pub query_size: usize,
    /// Initial query
    pub initial_query_size: usize,
    /// Growth increment per claro round
    pub query_multiplier: usize,
    /// Max value for [`QueryConfiguration::query_multiplier`]
    pub max_multiplier: usize,
}

impl QueryConfiguration {
    #[allow(dead_code)]
    pub fn new(query_size: usize) -> Self {
        Self {
            query_size,
            initial_query_size: query_size,
            // TODO: Should this be configurable? Runtime vs Compiled
            query_multiplier: 2,
            max_multiplier: 4,
        }
    }

    /// Increment query based upon configuration
    /// query_size = min(query_size * growth_constant, initial_query_size * growth_max)
    fn grow(&mut self) {
        self.query_size = (self.query_size * self.query_multiplier)
            .min(self.initial_query_size * self.max_multiplier);
    }
}

/// Claro algorithm configuration
#[derive(Debug, Clone, Copy)]
pub struct ClaroConfiguration {
    pub evidence_alpha: f32,
    pub evidence_alpha_2: f32,
    pub confidence_beta: f32,
    pub look_ahead: usize,
    pub query: QueryConfiguration,
}

/// Claro computation object
pub struct ClaroSolver<Tx> {
    _phantom: PhantomData<Tx>,
    /// Internal state
    state: ClaroState,
    /// Configuration, including node query configuration
    configuration: ClaroConfiguration,
    /// Current tx decision
    decision: Decision<Tx>,
    /// Node query setup for current node
    node_query: NodeQuery,
}

// TODO: can we remove clone here?
impl<Tx: Clone + Debug> ClaroSolver<Tx> {
    pub fn new(tx: Tx, configuration: ClaroConfiguration, node_query: NodeQuery) -> Self {
        Self {
            _phantom: Default::default(),
            state: Default::default(),
            decision: Decision::Undecided(Opinion::Yes(tx)),
            configuration,
            node_query,
        }
    }

    pub fn with_initial_opinion(
        configuration: ClaroConfiguration,
        node_query: NodeQuery,
        opinion: Opinion<Tx>,
    ) -> Self {
        Self {
            _phantom: Default::default(),
            state: Default::default(),
            decision: Decision::Undecided(opinion),
            configuration,
            node_query,
        }
    }

    /// Compute a single round state from already queried nodes votes
    fn round_state(&self, votes: &[Vote<Tx>]) -> ClaroRoundCalculation {
        let total_votes = votes.len();
        let yes_votes = votes.iter().filter(|&v| matches!(v, Vote::Yes(_))).count();
        let confidence = self.state.confidence() as f32
            / (self.state.confidence() as f32 + self.configuration.look_ahead as f32);

        let e1 = yes_votes as f32 / total_votes as f32;
        let e2 = self.state.evidence() as f32 / self.state.evidence_accumulated() as f32;
        let e = e1 * (1f32 - confidence) + e2 * confidence;
        let alpha = self.configuration.evidence_alpha * (1f32 - confidence)
            + self.configuration.evidence_alpha_2 * confidence;

        ClaroRoundCalculation {
            confidence,
            e1,
            e2,
            e,
            alpha,
        }
    }

    /// Compute a single round
    /// mutates the decision parameter upon this round data
    pub fn step(&mut self, tx: Tx, votes: &[Vote<Tx>]) {
        assert!(matches!(self.decision, Decision::Undecided(_)));
        debug!(votes = ?votes);
        if let Decision::Undecided(Opinion::None(_)) = self.decision() {
            if let Some(vote) = votes.first().cloned() {
                self.decision = Decision::Undecided(vote.into());
            }
        }

        if !votes.is_empty() {
            self.state.update_evidence(votes);
            self.state.update_confidence(votes);

            let ClaroRoundCalculation {
                e,
                alpha,
                confidence,
                ..
            } = self.round_state(votes);
            debug!(e = e, alpha = alpha);
            if e > alpha {
                self.decision = Decision::Undecided(Opinion::Yes(tx));
            } else if e < 1f32 - alpha {
                self.decision = Decision::Undecided(Opinion::No(tx));
            } else {
                self.configuration.query.grow();
            }
            if confidence > self.configuration.confidence_beta {
                self.decision = Decision::Decided(self.opinion());
            }
        }
    }

    /// Derive vote from it's current decision
    pub fn vote(&self) -> Option<Vote<Tx>> {
        self.decision.clone().into()
    }

    pub fn decision(&self) -> Decision<Tx> {
        self.decision.clone()
    }

    pub fn opinion(&self) -> Opinion<Tx> {
        match &self.decision {
            Decision::Decided(o) | Decision::Undecided(o) => o.clone(),
        }
    }

    pub fn state(&self) -> &ClaroState {
        &self.state
    }

    pub fn node_query(&self) -> &NodeQuery {
        &self.node_query
    }
}

#[cfg(test)]
mod test {
    use crate::claro::{ClaroConfiguration, ClaroSolver, Decision, QueryConfiguration, Vote};
    use crate::query::NodeQuery;
    use crate::testing::query::*;
    use crate::{Opinion, VoteQuery};
    use std::fmt::Debug;

    #[derive(Clone, Eq, PartialEq, Debug)]
    struct EmptyTx;

    fn test_all_votes<Tx: Clone + PartialEq + Debug + Send + Sync + 'static>(
        tx: Tx,
        votes: &[Vote<Tx>],
        expected: Decision<Tx>,
    ) {
        let config = ClaroConfiguration {
            evidence_alpha: 0.01,
            evidence_alpha_2: 0.01,
            confidence_beta: 0.01,
            look_ahead: 1,
            query: QueryConfiguration::new(10),
        };
        let node_query = NodeQuery::new(config.query.query_size, "node_1".into());
        let mut solver = ClaroSolver::new(tx.clone(), config, node_query);

        assert_eq!(
            solver.decision,
            Decision::Undecided(Opinion::Yes(tx.clone()))
        );
        solver.step(tx, votes);
        assert_eq!(solver.decision, expected);
    }

    #[test]
    fn all_approved() {
        let votes: Vec<_> = (0..10).map(|_| Vote::<bool>::Yes(true)).collect();
        test_all_votes::<bool>(true, &votes, Decision::Decided(Opinion::Yes(true)));
    }

    #[test]
    fn all_rejected() {
        let votes: Vec<_> = (0..10).map(|_| Vote::<bool>::No(true)).collect();
        test_all_votes::<bool>(true, &votes, Decision::Decided(Opinion::No(true)));
    }

    #[tokio::test]
    async fn loop_all_approved() {
        let vote = Vote::Yes(EmptyTx);
        let mut fixed_query = FixedQuery::new(vote.clone());
        let config = ClaroConfiguration {
            evidence_alpha: 0.01,
            evidence_alpha_2: 0.01,
            confidence_beta: 0.01,
            look_ahead: 1,
            query: QueryConfiguration::new(10),
        };

        let node_query = NodeQuery::new(config.query.query_size, "node_1".into());
        let mut solver = ClaroSolver::new(EmptyTx, config, node_query);

        let query = fixed_query.query(&solver.node_query, EmptyTx).await;
        solver.step(EmptyTx, &query);
        assert_eq!(solver.vote(), Some(vote))
    }
}
