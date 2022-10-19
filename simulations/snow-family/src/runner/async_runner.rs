use crate::node::{ComputeNode, Vote};
use crate::output_processors::OutData;
use crate::runner::SimulationRunner;
use crate::warding::SimulationState;
use rand::prelude::SliceRandom;
use rayon::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;

pub fn simulate(
    runner: &mut SimulationRunner,
    chunk_size: usize,
    mut out_data: Option<&mut Vec<OutData>>,
) {
    let mut node_ids: Vec<usize> = (0..runner
        .nodes
        .read()
        .expect("Read access to nodes vector")
        .len())
        .collect();
    let mut simulation_state = SimulationState {
        network_state: Arc::clone(&runner.network_state),
        nodes: Arc::clone(&runner.nodes),
        iteration: 0,
        round: 0,
    };

    runner.dump_state_to_out_data(&simulation_state, &mut out_data);

    loop {
        node_ids.shuffle(&mut runner.rng);
        for ids_chunk in node_ids.chunks(chunk_size) {
            if let Some(master_omniscient) = runner.master_omniscient.as_mut() {
                master_omniscient.step();
            }
            let ids: HashSet<usize> = ids_chunk.iter().copied().collect();
            let new_state: Vec<Option<Vote>> = runner
                .nodes
                .write()
                .expect("Write access to nodes vector")
                .par_iter_mut()
                .enumerate()
                .map(|(id, node)| {
                    if ids.contains(&id) {
                        node.step();
                        node.vote()
                    } else {
                        node.vote()
                    }
                })
                .collect();
            runner.set_new_network_state(new_state);
            runner.dump_state_to_out_data(&simulation_state, &mut out_data);
            simulation_state.iteration += 1;
        }
        simulation_state.round += 1;
        // check if any condition makes the simulation stop
        if runner.check_wards(&simulation_state) {
            break;
        }
        // run modifiers over the current step network state
        runner.run_network_behaviour_modifiers();
    }
}
