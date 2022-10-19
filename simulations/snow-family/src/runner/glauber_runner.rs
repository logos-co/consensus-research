use crate::node::{ComputeNode, Node, NodeId};
use crate::output_processors::OutData;
use crate::runner::SimulationRunner;
use crate::warding::SimulationState;
use rand::prelude::IteratorRandom;
use std::collections::BTreeSet;
use std::sync::Arc;

/// [Glauber dynamics simulation](https://en.wikipedia.org/wiki/Glauber_dynamics)
pub fn simulate(
    runner: &mut SimulationRunner,
    update_rate: usize,
    maximum_iterations: usize,
    mut out_data: Option<&mut Vec<OutData>>,
) {
    let mut simulation_state = SimulationState {
        network_state: Arc::clone(&runner.network_state),
        nodes: Arc::clone(&runner.nodes),
        iteration: 0,
        round: 0,
    };
    let mut nodes_remaining: BTreeSet<NodeId> = (0..runner
        .nodes
        .read()
        .expect("Read access to nodes vector")
        .len())
        .collect();
    let iterations: Vec<_> = (0..maximum_iterations).collect();
    'main: for chunk in iterations.chunks(update_rate) {
        for i in chunk {
            simulation_state.iteration = *i;
            if nodes_remaining.is_empty() {
                break 'main;
            }

            let node_id = *nodes_remaining.iter().choose(&mut runner.rng).expect(
                "Some id to be selected as it should be impossible for the set to be empty here",
            );

            {
                let vote = {
                    let mut shared_nodes =
                        runner.nodes.write().expect("Write access to nodes vector");
                    let node: &mut Node = shared_nodes
                        .get_mut(node_id)
                        .expect("Node should be present");

                    node.step();
                    if matches!(node.decision(), claro::Decision::Decided(_)) {
                        nodes_remaining.remove(&node_id);
                    }
                    node.vote()
                };
                runner.update_single_network_state_vote(node_id, vote);
            }

            // check if any condition makes the simulation stop
            if runner.check_wards(&simulation_state) {
                break 'main;
            }
            // run modifiers over the current step network state
            runner.run_network_behaviour_modifiers();
        }
        runner.dump_state_to_out_data(&simulation_state, &mut out_data);
    }
}
