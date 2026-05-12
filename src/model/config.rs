use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub pearl: Pearl,
    pub directions: Directions,
    pub code: CodeRule,
    pub motion_per_tnt: MotionPerTnt,
    pub max_tnt: TNTNumRB,
    pub max_error: f64,
    pub show_first: usize,
    #[serde(default)]
    pub min_time: Time,
    pub max_time: Time,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    version: u64,
    config: Config,
}

impl TryFrom<Root> for Config {
    type Error = PearlError;

    fn try_from(value: Root) -> Result<Self, Self::Error> {
        match value.version {
            1 => Ok(value.config),
            _ => Err(PearlError::UnsupportedConfigVersion(value.version)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct MotionPerTnt {
    pub x_z: f64,
    pub y: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Direction {
    pub red: [i8; 2],
    pub blue: [i8; 2],
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Directions(pub [Direction; 4]);

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct TNTNumRB {
    pub red: u64,
    pub blue: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TNTNumCode(pub Vec<bool>);

#[derive(Clone, Copy, Debug)]
pub struct RB {
    pub num: TNTNumRB,
    pub direction: usize,
}

impl Config {
    pub fn check(&self) -> Result<(), PearlError> {
        self.directions.resolve()?;
        self.code.check()?;
        Ok(())
    }
}
