use std::error::Error;
use std::fmt::Debug;
// std
// crates
use crate::network_behaviour::NetworkModifiers;
use crate::node::Opinion;
use crate::warding::Ward;
use serde::Deserialize;
// internal

/// Foreign Serialize, Deserialize implementation for `::snowball::SnowballConfiguration`
#[derive(Debug, Deserialize)]
#[serde(remote = "::snowball::SnowballConfiguration")]
pub struct SnowballConfigurationDeSer {
    pub quorum_size: usize,
    pub sample_size: usize,
    pub decision_threshold: usize,
}

/// Foreign Serialize, Deserialize implementation for `::claro::QueryConfiguration`
#[derive(Debug, Deserialize)]
#[serde(remote = "::claro::QueryConfiguration")]
pub struct QueryConfigurationDeSer {
    pub query_size: usize,
    pub initial_query_size: usize,
    pub query_multiplier: usize,
    pub max_multiplier: usize,
}

/// Foreign Serialize, Deserialize implementation for `::claro::ClaroConfiguration`
#[derive(Debug, Deserialize)]
#[serde(remote = "::claro::ClaroConfiguration")]
pub struct ClaroConfigurationDeSer {
    pub evidence_alpha: f32,
    pub evidence_alpha_2: f32,
    pub confidence_beta: f32,
    pub look_ahead: usize,
    #[serde(with = "QueryConfigurationDeSer")]
    pub query: ::claro::QueryConfiguration,
}

/// Consensus selector
#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusSettings {
    SnowBall(#[serde(with = "SnowballConfigurationDeSer")] ::snowball::SnowballConfiguration),
    Claro(#[serde(with = "ClaroConfigurationDeSer")] ::claro::ClaroConfiguration),
}

impl ConsensusSettings {
    pub fn query_size(&self) -> usize {
        match self {
            ConsensusSettings::SnowBall(snowball) => snowball.sample_size,
            ConsensusSettings::Claro(claro) => claro.query.query_size,
        }
    }
}

/// Initial normalized distribution settings for hones nodes. Must sum up to `1.0`
#[derive(Debug, Deserialize)]
pub struct InitialDistribution {
    pub yes: f32,
    pub no: f32,
    pub none: f32,
}

impl InitialDistribution {
    pub fn weight_by_opinion(&self, opinion: &Opinion) -> f32 {
        match opinion {
            Opinion::None(_) => self.none,
            Opinion::Yes(_) => self.yes,
            Opinion::No(_) => self.no,
        }
    }

    pub fn check_distribution(&self) -> Result<(), Box<dyn Error>> {
        let values = [self.none, self.yes, self.no];
        check_normalized_distribution(self, &values)
    }
}

/// Byzantine nodes normalized distribution. Must sum up to `1.0`
#[derive(Debug, Deserialize)]
pub struct ByzantineDistribution {
    pub honest: f32,
    pub infantile: f32,
    pub random: f32,
    pub omniscient: f32,
}

impl ByzantineDistribution {
    pub fn check_distribution(&self) -> Result<(), Box<dyn Error>> {
        let values = [self.honest, self.infantile, self.random, self.omniscient];
        check_normalized_distribution(self, &values)
    }
}

/// Byzantine settings, size of simulation and byzantine distribution
#[derive(Debug, Deserialize)]
pub struct ByzantineSettings {
    pub total_size: usize,
    pub distribution: ByzantineDistribution,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub enum SimulationStyle {
    #[default]
    Sync,
    Async {
        chunks: usize,
    },
    Glauber {
        maximum_iterations: usize,
        update_rate: usize,
    },
    Layered {
        rounds_gap: usize,
        distribution: Option<Vec<f32>>,
    },
}

/// Full simulation settings:
/// * consensus settings
/// * initial distribution
/// * byzantine setting
/// * simulation wards
/// * simulation network behaviour modifiers
/// * simulation style
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SimulationSettings {
    pub consensus_settings: ConsensusSettings,
    pub distribution: InitialDistribution,
    pub byzantine_settings: ByzantineSettings,
    #[serde(default)]
    pub wards: Vec<Ward>,
    #[serde(default)]
    pub network_modifiers: Vec<NetworkModifiers>,
    #[serde(default)]
    pub simulation_style: SimulationStyle,
    #[serde(default)]
    pub seed: Option<u64>,
}

/// Check if a settings distribution is normalized (sum up to `1.0`)  
fn check_normalized_distribution<T: Debug>(
    holder: T,
    distribution: &[f32],
) -> Result<(), Box<dyn Error>> {
    let value: f32 = distribution.iter().sum();
    if value != 1.0f32 {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{holder:?} distribution is not normalized, values sum {value} != 1.0"),
        )))
    } else {
        Ok(())
    }
}
