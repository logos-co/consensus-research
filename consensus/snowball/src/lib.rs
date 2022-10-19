#[allow(dead_code)]
mod snowball;

pub use crate::snowball::{SnowballConfiguration, SnowballSolver};

/// Snowball logging filtering tag
pub const SNOWBALL_TARGET_TAG: &str = "SNOWBALL_TARGET";
