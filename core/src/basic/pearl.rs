use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Pearl {
    pub position: Array,
    pub motion: Array,
    #[serde(default)]
    pub yaw: Yaw,
    #[serde(default)]
    pub dimension: Dimension,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default)]
#[serde(default)]
pub struct Yaw(pub f32);
