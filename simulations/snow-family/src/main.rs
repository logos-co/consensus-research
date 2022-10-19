mod app;
mod network_behaviour;
mod node;
mod output_processors;
mod runner;
mod settings;
mod warding;

use crate::app::SimulationApp;
use clap::Parser;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let app: SimulationApp = app::SimulationApp::parse();
    app.run()?;
    Ok(())
}
