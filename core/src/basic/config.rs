use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub pearl: Pearl,
    pub directions: Directions,
    pub code: Code,
    pub motion_per_tnt: MotionPerTnt,
    pub max_tnt: TNTNumRB,
    pub max_error: f64,
    pub show_first: usize,
    pub max_time: Time,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    version: u64,
    config: Config,
}

impl From<Root> for Config {
    fn from(value: Root) -> Self {
        match value.version {
            1 => value.config,
            _ => panic!(),
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

#[derive(Clone, Copy, Debug)]
pub struct RB {
    pub num: TNTNumRB,
    pub direction: usize,
}
