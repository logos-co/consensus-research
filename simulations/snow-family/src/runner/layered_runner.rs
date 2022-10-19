//! # Layered simulation runner
//!
//! A revision of the [`glauber`](super::glauber_runner) simulation runner.
//!
//! **`glauber`** simulations have some drawbacks:
//!
//! * Completely random, difficult to control
//! * Not close to how real nodes would perform in reality
//! * Difficult to analise recorded data, as data it is updated by chunks of iterations
//!
//! To solve this we can use a concept of layered *glauber* executions.
//! The algorithm roughly works as follows:
//!
//! ```
//! nodes <- [nodes]
//! layers <- [[nodes_ids], [], ...]
//! while nodes_to_compute(layers):
//!     layer_index <- pick_rand_layer(layers)
//!     node_index <- pop_rand_node(rand_layer)
//!     step(nodes[node_index])
//!     if not node_decided(node):
//!         push(layers[layer_index+1], node_index)
//! ```
//!
//! From within this, controlling the *number of layers*, and *weighting* them (how often are they picked),
//! we can control the flow of the simulations.
//! Also we can consider that once the bottom layer is empty a fully step have been concluded and we can record
//! the data of that step simulation.

// std
use std::collections::BTreeSet;
use std::ops::Not;
use std::sync::Arc;
// crates
use fixed_slice_deque::FixedSliceDeque;
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::rngs::SmallRng;
// internal
use crate::node::{ComputeNode, Node, NodeId};
use crate::output_processors::OutData;
use crate::runner::SimulationRunner;
use crate::warding::SimulationState;

pub fn simulate(
    runner: &mut SimulationRunner,
    gap: usize,
    distribution: Option<Vec<f32>>,
    mut out_data: Option<&mut Vec<OutData>>,
) {
    let distribution =
        distribution.unwrap_or_else(|| std::iter::repeat(1.0f32).take(gap).collect());

    let layers: Vec<usize> = (0..gap).collect();

    let mut deque = build_node_ids_deque(gap, runner);

    let mut simulation_state = SimulationState {
        network_state: Arc::clone(&runner.network_state),
        nodes: Arc::clone(&runner.nodes),
        iteration: 0,
        round: 0,
    };

    loop {
        let (group_index, node_id) =
            choose_random_layer_and_node_id(&mut runner.rng, &distribution, &layers, &mut deque);

        // remove node_id from group
        deque.get_mut(group_index).unwrap().remove(&node_id);

        let vote = {
            let mut shared_nodes = runner.nodes.write().expect("Write access to nodes vector");
            let node: &mut Node = shared_nodes
                .get_mut(node_id)
                .expect("Node should be present");

            node.step();
            if !matches!(node.decision(), claro::Decision::Decided(_)) {
                // pass node to next step group
                deque.get_mut(group_index + 1).unwrap().insert(node_id);
            }
            node.vote()
        };
        runner.update_single_network_state_vote(node_id, vote);

        // check if any condition makes the simulation stop
        if runner.check_wards(&simulation_state) {
            break;
        }
        // run modifiers over the current step network state
        runner.run_network_behaviour_modifiers();
        simulation_state.iteration += 1;

        // if initial is empty then we finished a full round, append a new set to the end so we can
        // compute the most advanced nodes again
        if deque.first().unwrap().is_empty() {
            let _ = deque.push_back(BTreeSet::default());
            runner.dump_state_to_out_data(&simulation_state, &mut out_data);
            simulation_state.round += 1;
        }

        // if no more nodes to compute
        if deque.iter().all(BTreeSet::is_empty) {
            break;
        }
    }
    // write latest state
    runner.dump_state_to_out_data(&simulation_state, &mut out_data);
}

fn choose_random_layer_and_node_id(
    rng: &mut SmallRng,
    distribution: &[f32],
    layers: &[usize],
    deque: &mut FixedSliceDeque<BTreeSet<NodeId>>,
) -> (usize, NodeId) {
    let i = *layers
        .iter()
        // filter out empty round groups
        .filter_map(|&i| {
            let g = deque.get(i).unwrap();
            g.is_empty().not().then_some(i)
        })
        // intermediate collect necessary for choose_weighted
        .collect::<Vec<_>>()
        .choose_weighted(rng, |&i| distribution.get(i).unwrap())
        .expect("Distribution choose to work");

    let group: &mut BTreeSet<NodeId> = deque.get_mut(i).unwrap();

    let node_id = group.iter().choose(rng).unwrap();
    (i, *node_id)
}

fn build_node_ids_deque(
    gap: usize,
    runner: &SimulationRunner,
) -> FixedSliceDeque<BTreeSet<NodeId>> {
    // add a +1 so we always have
    let mut deque = FixedSliceDeque::new(gap + 1);
    // push first layer
    let node_ids: BTreeSet<NodeId> = runner
        .nodes
        .write()
        .expect("Single access to runner nodes")
        .iter()
        .map(|node| node.id())
        .collect();

    deque.push_back(node_ids);
    // allocate default sets
    while deque.try_push_back(BTreeSet::new()).is_ok() {}
    deque
}
