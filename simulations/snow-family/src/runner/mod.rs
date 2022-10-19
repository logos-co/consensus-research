mod async_runner;
mod glauber_runner;
mod layered_runner;
mod sync_runner;

// std
use std::sync::{Arc, RwLock};
// crates
use rand::prelude::SliceRandom;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use rayon::prelude::*;
// internal
use crate::network_behaviour::NetworkBehaviour;
use crate::node::{
    ComputeNode, MasterOmniscientNode, NetworkState, NoTx, Node, NodeId, Opinion, Vote,
};
use crate::output_processors::OutData;
use crate::settings::{
    ByzantineDistribution, ByzantineSettings, ConsensusSettings, SimulationSettings,
    SimulationStyle,
};
use crate::warding::{SimulationState, SimulationWard};
use claro::{ClaroSolver, NodeQuery};
use snowball::SnowballSolver;

/// Encapsulation solution for the simulations runner
/// Holds the network state, the simulating nodes and the simulation settings.
pub struct SimulationRunner {
    network_state: NetworkState,
    nodes: Arc<RwLock<Vec<Node>>>,
    master_omniscient: Option<MasterOmniscientNode>,
    settings: SimulationSettings,
    rng: SmallRng,
}

impl SimulationRunner {
    pub fn new(settings: SimulationSettings) -> Self {
        let seed = settings
            .seed
            .unwrap_or_else(|| rand::thread_rng().next_u64());

        println!("Seed: {}", seed);

        let mut rng = SmallRng::seed_from_u64(seed);

        let (nodes, network_state, master_omniscient) =
            Self::nodes_from_initial_settings(&settings, &mut rng);

        let nodes = Arc::new(RwLock::new(nodes));

        Self {
            network_state,
            nodes,
            master_omniscient,
            settings,
            rng,
        }
    }

    /// Initialize nodes from settings and calculate initial network state.
    fn nodes_from_initial_settings(
        settings: &SimulationSettings,
        mut seed: &mut SmallRng,
    ) -> (Vec<Node>, NetworkState, Option<MasterOmniscientNode>) {
        let SimulationSettings {
            consensus_settings,
            distribution,
            byzantine_settings:
                ByzantineSettings {
                    total_size,
                    distribution:
                        ByzantineDistribution {
                            honest,
                            infantile,
                            random,
                            omniscient,
                        },
                },
            ..
        } = settings;

        // shuffling is just for representation
        let mut node_ids: Vec<_> = (0..*total_size).collect();
        node_ids.shuffle(seed);
        let mut node_ids_iter = node_ids.into_iter();

        // total sized based sizes
        let [honest_size, infantile_size, random_size, omniscient_size] =
            [honest, infantile, random, omniscient]
                .map(|&x| (*total_size as f32 * x).round() as usize);

        dbg!([honest_size, infantile_size, random_size, omniscient_size]);

        let options = [Opinion::None(NoTx), Opinion::Yes(NoTx), Opinion::No(NoTx)];

        // build up initial hones nodes distribution
        let mut votes_distribution: Vec<Opinion> = options
            .into_iter()
            .flat_map(|opinion| {
                let size: usize =
                    (honest_size as f32 * distribution.weight_by_opinion(&opinion)) as usize;
                std::iter::repeat(opinion).take(size)
            })
            .chain(std::iter::repeat(Opinion::None(NoTx)))
            .take(honest_size)
            .collect();

        // check that we actually have all opinions as needed
        assert_eq!(votes_distribution.len(), honest_size);

        // shuffle distribution
        votes_distribution.shuffle(seed);

        // uninitialized network state, should be recalculated afterwards
        let network_state: NetworkState = Arc::new(RwLock::new(vec![None; *total_size]));

        // Allow needless collect: we actually need to do so in order to liberate the node_ids_iter
        // otherwise it is borrowed mutably more than once...apparently the compiler is not smart enough (still)
        // to catch that it should be safe to do so in this case. So, we collect.
        // This should not really impact on running performance other than getting the nodes prepared
        // would take a bit more.
        let hones_nodes_ids: Vec<_> = std::iter::from_fn(|| node_ids_iter.next())
            .take(honest_size)
            .collect();

        #[allow(clippy::needless_collect)]
        let honest_nodes: Vec<_> = Self::build_honest_nodes(
            hones_nodes_ids.iter().copied().zip(votes_distribution),
            *total_size,
            Arc::clone(&network_state),
            *consensus_settings,
            seed,
        )
        .collect();

        #[allow(clippy::needless_collect)]
        let infantile_nodes: Vec<_> = std::iter::from_fn(|| node_ids_iter.next())
            .take(infantile_size)
            .map(|node_id| {
                Node::new_infantile(
                    node_id,
                    consensus_settings.query_size(),
                    Arc::clone(&network_state),
                    SmallRng::from_rng(&mut seed).expect("Rng should build properly from seed rng"),
                )
            })
            .collect();

        #[allow(clippy::needless_collect)]
        let random_nodes: Vec<_> = std::iter::from_fn(|| node_ids_iter.next())
            .take(random_size)
            .map(Node::new_random)
            .collect();

        let (master_omniscient, omniscient_nodes) = {
            if omniscient_size > 0 {
                let omniscient_nodes_ids: Vec<_> = std::iter::from_fn(|| node_ids_iter.next())
                    .take(omniscient_size)
                    .collect();

                let omniscient_node = MasterOmniscientNode::new(
                    NodeId::MAX,
                    hones_nodes_ids,
                    omniscient_nodes_ids.clone(),
                    Arc::clone(&network_state),
                );

                #[allow(clippy::needless_collect)]
                let puppets: Vec<_> = omniscient_nodes_ids
                    .iter()
                    .map(|id| Node::new_omniscient_puppet(omniscient_node.puppet_node(*id)))
                    .collect();

                (Some(omniscient_node), puppets.into_iter())
            } else {
                (None, vec![].into_iter())
            }
        };

        let mut nodes: Vec<Node> = honest_nodes
            .into_iter()
            .chain(omniscient_nodes)
            .chain(infantile_nodes.into_iter())
            .chain(random_nodes.into_iter())
            .collect();

        nodes.sort_unstable_by_key(|node| node.inner_node().id());

        // set up network state with the current distribution
        let new_network_state = Self::network_state_from_nodes(&nodes);
        *network_state.write().unwrap() = new_network_state;
        (nodes, network_state, master_omniscient)
    }

    fn build_honest_nodes<'a>(
        node_data: impl Iterator<Item = (NodeId, Opinion)> + 'a,
        total_size: usize,
        network_state: NetworkState,
        consensus_settings: ConsensusSettings,
        mut seed: &'a mut SmallRng,
    ) -> impl Iterator<Item = Node> + 'a {
        match consensus_settings {
            ConsensusSettings::SnowBall(snowball_settings) => {
                node_data.map(Box::new(move |(node_id, opinion)| {
                    Node::new_snowball(
                        node_id,
                        SnowballSolver::with_initial_opinion(
                            snowball_settings,
                            NodeQuery::new(total_size, node_id.to_string()),
                            opinion,
                        ),
                        Arc::clone(&network_state),
                        SmallRng::from_rng(&mut seed)
                            .expect("Rng should build properly from seed rng"),
                    )
                })
                    as Box<dyn FnMut((usize, Opinion)) -> Node>)
            }
            ConsensusSettings::Claro(claro_settings) => {
                node_data.map(Box::new(move |(node_id, opinion)| {
                    Node::new_claro(
                        node_id,
                        ClaroSolver::with_initial_opinion(
                            claro_settings,
                            NodeQuery::new(total_size, node_id.to_string()),
                            opinion,
                        ),
                        Arc::clone(&network_state),
                        SmallRng::from_rng(&mut seed)
                            .expect("Rng should build properly from seed rng"),
                    )
                })
                    as Box<dyn FnMut((usize, Opinion)) -> Node>)
            }
        }
    }

    #[inline]
    fn network_state_from_nodes(nodes: &[Node]) -> Vec<Option<Vote>> {
        dbg!(nodes.len());
        nodes.par_iter().map(|node| node.vote()).collect()
    }

    pub fn simulate(&mut self, out_data: Option<&mut Vec<OutData>>) {
        match self.settings.simulation_style.clone() {
            SimulationStyle::Sync => {
                sync_runner::simulate(self, out_data);
            }
            SimulationStyle::Async { chunks } => {
                async_runner::simulate(self, chunks, out_data);
            }
            SimulationStyle::Glauber {
                maximum_iterations,
                update_rate,
            } => {
                glauber_runner::simulate(self, update_rate, maximum_iterations, out_data);
            }
            SimulationStyle::Layered {
                rounds_gap,
                distribution,
            } => {
                layered_runner::simulate(self, rounds_gap, distribution, out_data);
            }
        }
    }

    fn dump_state_to_out_data(
        &self,
        simulation_state: &SimulationState,
        out_ata: &mut Option<&mut Vec<OutData>>,
    ) {
        if let Some(out) = out_ata.as_deref_mut() {
            let nodes = self.nodes.read().unwrap();
            let iteration = simulation_state.iteration as u64;
            let round = simulation_state.round as u64;
            let updated = nodes.iter().map(|node| {
                let node_type = node.type_as_string();
                let vote = match node.vote() {
                    None => 0u8,
                    Some(Vote::Yes(_)) => 1,
                    Some(Vote::No(_)) => 2,
                };
                OutData {
                    id: node.inner_node().id() as u64,
                    iteration,
                    _type: node_type,
                    round,
                    vote,
                    state: node.serialized_state().get_serialized_state_record(),
                }
            });

            out.extend(updated);
        }
    }

    fn check_wards(&mut self, state: &SimulationState) -> bool {
        self.settings
            .wards
            .par_iter_mut()
            .map(|ward| ward.analyze(state))
            .any(|x| x)
    }

    fn run_network_behaviour_modifiers(&mut self) {
        let mut network_state = self
            .network_state
            .write()
            .expect("Single access to network state for running behaviour modifiers");

        for modifier in self.settings.network_modifiers.iter_mut() {
            modifier.modify_network_state(&mut network_state, &mut self.rng);
        }
    }

    pub fn step(&mut self) {
        let new_network_state: Vec<Option<Vote>> = self.run_step();
        self.set_new_network_state(new_network_state);
    }

    fn set_new_network_state(&mut self, new_network_state: Vec<Option<Vote>>) {
        let mut network_state = self
            .network_state
            .write()
            .expect("No threads could be accessing the network state");

        *network_state = new_network_state;
    }

    fn update_single_network_state_vote(&mut self, id: NodeId, vote: Option<Vote>) {
        let mut network_state = self
            .network_state
            .write()
            .expect("No threads could be accessing the network state");

        *network_state.get_mut(id).unwrap() = vote;
    }

    fn run_step(&mut self) -> Vec<Option<Vote>> {
        if let Some(master_omniscient) = self.master_omniscient.as_mut() {
            master_omniscient.step();
        }
        self.nodes
            .write()
            .expect("Single access to nodes vector")
            .par_iter_mut()
            .map(|node| {
                node.step();
                node.vote()
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::node::{ComputeNode, Node, Vote};
    use crate::runner::SimulationRunner;
    use crate::settings::{
        ByzantineDistribution, ByzantineSettings, ConsensusSettings, InitialDistribution,
        SimulationSettings,
    };
    use claro::{ClaroConfiguration, QueryConfiguration};
    use rand::rngs::SmallRng;
    use rand::{thread_rng, SeedableRng};

    #[test]
    fn nodes_distribution_from_initial_settings() {
        let initial_settings = SimulationSettings {
            simulation_style: Default::default(),
            consensus_settings: ConsensusSettings::Claro(ClaroConfiguration {
                evidence_alpha: 0.0,
                evidence_alpha_2: 0.0,
                confidence_beta: 0.0,
                look_ahead: 0,
                query: QueryConfiguration {
                    query_size: 0,
                    initial_query_size: 0,
                    query_multiplier: 0,
                    max_multiplier: 0,
                },
            }),
            distribution: InitialDistribution {
                yes: 0.5,
                no: 0.5,
                none: 0.0,
            },
            byzantine_settings: ByzantineSettings {
                total_size: 999,
                distribution: ByzantineDistribution {
                    honest: 0.7,
                    infantile: 0.1,
                    random: 0.1,
                    omniscient: 0.1,
                },
            },
            wards: vec![],
            network_modifiers: vec![],
            seed: None,
        };
        let mut rng = SmallRng::from_rng(&mut thread_rng()).unwrap();
        let (nodes, _, _) =
            SimulationRunner::nodes_from_initial_settings(&initial_settings, &mut rng);
        let honest_nodes: Vec<_> = nodes
            .iter()
            .filter(|node| matches!(node, Node::Claro(_)))
            .collect();

        assert_eq!(
            honest_nodes.len(),
            (initial_settings.byzantine_settings.total_size as f32
                * initial_settings.byzantine_settings.distribution.honest) as usize
        );

        let half_count = honest_nodes.len() / 2;

        let yes_count = honest_nodes
            .iter()
            .filter(|node| matches!(node.vote(), Some(Vote::Yes(_))))
            .count();

        assert_eq!(yes_count, half_count);

        let no_count = honest_nodes
            .iter()
            .filter(|node| matches!(node.vote(), Some(Vote::No(_))))
            .count();

        assert_eq!(no_count, half_count);

        let byzantine_rate_size = 100;

        let infantile_nodes_count = nodes
            .iter()
            .filter(|node| matches!(node, Node::Infantile(_)))
            .count();

        assert_eq!(infantile_nodes_count, byzantine_rate_size);

        let random_nodes_count = nodes
            .iter()
            .filter(|node| matches!(node, Node::Random(_)))
            .count();

        assert_eq!(random_nodes_count, byzantine_rate_size);

        let omniscient_nodes_count = nodes
            .iter()
            .filter(|node| matches!(node, Node::OmniscientPuppet(_)))
            .count();

        assert_eq!(omniscient_nodes_count, byzantine_rate_size);
    }
}
