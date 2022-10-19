use claro::{Decision, NodeQuery, Opinion, Vote};
use std::fmt::Debug;

/// Snowball algorithm configuration
#[derive(Debug, Clone, Copy)]
pub struct SnowballConfiguration {
    pub quorum_size: usize,
    pub sample_size: usize,
    pub decision_threshold: usize,
}

/// Snowball computation object
pub struct SnowballSolver<Tx> {
    configuration: SnowballConfiguration,
    decision: Decision<Tx>,
    consecutive_success: u64,
    node_query: NodeQuery,
}

impl<Tx: Clone + Debug> SnowballSolver<Tx> {
    pub fn new(tx: Tx, configuration: SnowballConfiguration, node_query: NodeQuery) -> Self {
        Self {
            configuration,
            decision: Decision::Undecided(Opinion::None(tx)),
            consecutive_success: 0,
            node_query,
        }
    }

    pub fn with_initial_opinion(
        configuration: SnowballConfiguration,
        node_query: NodeQuery,
        opinion: Opinion<Tx>,
    ) -> Self {
        Self {
            configuration,
            decision: Decision::Undecided(opinion),
            consecutive_success: 0,
            node_query,
        }
    }

    fn count_opinion_votes(&self, votes: &[Vote<Tx>]) -> usize {
        votes
            .iter()
            .filter(|v| {
                matches!(
                    (v, self.vote()),
                    (Vote::Yes(_), Some(Vote::Yes(_))) | (Vote::No(_), Some(Vote::No(_)))
                )
            })
            .count()
    }

    pub fn step(&mut self, votes: &[Vote<Tx>]) {
        assert!(matches!(self.decision, Decision::Undecided(_)));

        let preference_count = self.count_opinion_votes(votes);
        let not_preference_count = votes.len() - preference_count;

        if preference_count >= self.configuration.quorum_size {
            self.consecutive_success += 1;
        } else if not_preference_count >= self.configuration.quorum_size {
            self.decision = Decision::Undecided(self.opinion().flip());
            self.consecutive_success = 1;
        } else {
            self.consecutive_success = 0
        }

        if self.consecutive_success > self.configuration.decision_threshold as u64 {
            self.decision = Decision::Decided(self.opinion())
        }
    }

    pub fn consecutive_success(&self) -> u64 {
        self.consecutive_success
    }

    pub fn decision(&self) -> Decision<Tx> {
        self.decision.clone()
    }

    pub fn opinion(&self) -> Opinion<Tx> {
        match &self.decision {
            Decision::Decided(o) | Decision::Undecided(o) => o.clone(),
        }
    }

    /// Derive vote from it's current decision
    pub fn vote(&self) -> Option<Vote<Tx>> {
        self.decision().into()
    }

    pub fn node_query(&self) -> &NodeQuery {
        &self.node_query
    }
}

#[cfg(test)]
mod test {
    use super::{SnowballConfiguration, SnowballSolver};
    use claro::{Decision, NodeQuery, Opinion, Vote};

    #[test]
    fn test_change_opinion() {
        let configuration = SnowballConfiguration {
            quorum_size: 1,
            sample_size: 10,
            decision_threshold: 10,
        };

        let mut solver = SnowballSolver::with_initial_opinion(
            configuration,
            NodeQuery::new(0, "0".to_string()),
            Opinion::Yes(true),
        );

        let votes = vec![Vote::No(true); 10];
        solver.step(&votes);
        assert!(matches!(solver.decision(), Decision::Undecided(_)));
        assert_eq!(solver.consecutive_success, 1);
        assert_eq!(solver.opinion(), Opinion::No(true));
    }

    #[test]
    fn test_makes_decision() {
        let configuration = SnowballConfiguration {
            quorum_size: 1,
            sample_size: 10,
            decision_threshold: 10,
        };
        let beta = configuration.decision_threshold;

        let mut solver = SnowballSolver::with_initial_opinion(
            configuration,
            NodeQuery::new(0, "0".to_string()),
            Opinion::Yes(true),
        );

        let votes = vec![Vote::No(true); 10];
        for _ in 0..beta + 1 {
            solver.step(&votes);
        }

        assert_eq!(solver.decision(), Decision::Decided(Opinion::No(true)));
        assert_eq!(solver.consecutive_success, beta as u64 + 1);
        assert_eq!(solver.opinion(), Opinion::No(true));
    }

    #[test]
    fn test_reset_consecutive_counter() {
        let configuration = SnowballConfiguration {
            quorum_size: 2,
            sample_size: 10,
            decision_threshold: 10,
        };

        let mut solver = SnowballSolver::with_initial_opinion(
            configuration,
            NodeQuery::new(0, "0".to_string()),
            Opinion::Yes(true),
        );

        let votes = vec![Vote::No(true), Vote::Yes(true)];

        solver.step(&votes);

        assert_eq!(solver.consecutive_success, 0);
        assert_eq!(solver.opinion(), Opinion::Yes(true));
        assert!(matches!(solver.decision(), Decision::Undecided(_)));
    }
}
