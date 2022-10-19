use crate::network_behaviour::NetworkBehaviour;
use crate::node::Vote;
use rand::prelude::IteratorRandom;
use rand::rngs::SmallRng;
use serde::Deserialize;

/// Randomly drop some of the network votes
/// Drop rate should be normalized
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RandomDrop {
    drop_rate: f32,
}

impl NetworkBehaviour for RandomDrop {
    fn modify_network_state(&mut self, network_state: &mut [Option<Vote>], rng: &mut SmallRng) {
        let amount: usize =
            (self.drop_rate.clamp(0f32, 1f32) * network_state.len() as f32).round() as usize;
        for i in (0..network_state.len()).choose_multiple(rng, amount) {
            *network_state.get_mut(i).unwrap() = None;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::network_behaviour::drop::RandomDrop;
    use crate::network_behaviour::NetworkBehaviour;
    use crate::node::{NoTx, Vote};
    use rand::prelude::SmallRng;
    use rand::SeedableRng;

    const SEED: u64 = 18042022;

    #[test]
    fn full_drop_rate() {
        let mut rng: SmallRng = SmallRng::seed_from_u64(SEED);
        let mut random_drop = RandomDrop { drop_rate: 1.0 };
        let mut votes: Vec<Option<Vote>> = (0..10).map(|_| Some(Vote::Yes(NoTx))).collect();
        random_drop.modify_network_state(&mut votes, &mut rng);
        assert!(votes.iter().all(Option::is_none));
    }

    #[test]
    fn none_drop_rate() {
        let mut rng: SmallRng = SmallRng::seed_from_u64(SEED);
        let mut random_drop = RandomDrop { drop_rate: 0.0 };
        let mut votes: Vec<Option<Vote>> = (0..10).map(|_| Some(Vote::Yes(NoTx))).collect();
        random_drop.modify_network_state(&mut votes, &mut rng);
        assert!(votes.iter().all(Option::is_some));
    }

    #[test]
    fn half_drop_rate() {
        let mut rng: SmallRng = SmallRng::seed_from_u64(SEED);
        let mut random_drop = RandomDrop { drop_rate: 0.5 };
        let mut votes: Vec<Option<Vote>> = (0..10).map(|_| Some(Vote::Yes(NoTx))).collect();
        random_drop.modify_network_state(&mut votes, &mut rng);
        assert_eq!(votes.iter().filter(|vote| vote.is_some()).count(), 5);
    }
}
