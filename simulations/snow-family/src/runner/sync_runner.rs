use super::SimulationRunner;
use crate::output_processors::OutData;
use crate::warding::SimulationState;
use std::sync::Arc;

/// Simulate with option of dumping the network state as a `::polars::Series`
pub fn simulate(runner: &mut SimulationRunner, mut out_data: Option<&mut Vec<OutData>>) {
    let mut state = SimulationState {
        network_state: Arc::clone(&runner.network_state),
        nodes: Arc::clone(&runner.nodes),
        iteration: 0,
        round: 0,
    };

    runner.dump_state_to_out_data(&state, &mut out_data);

    for i in 1.. {
        state.round = i;
        state.iteration = i;
        runner.step();
        runner.dump_state_to_out_data(&state, &mut out_data);
        // check if any condition makes the simulation stop
        if runner.check_wards(&state) {
            break;
        }
        // run modifiers over the current step network state
        runner.run_network_behaviour_modifiers();
    }
}
