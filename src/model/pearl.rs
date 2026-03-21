use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Pearl {
    pub position: Array,
    pub motion: Array,
    #[serde(default)]
    pub yaw: Angle,
    #[serde(default)]
    pub dimension: Dimension,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default)]
#[serde(default)]
pub struct Angle(pub f32);
