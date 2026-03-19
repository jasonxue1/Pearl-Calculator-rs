pub use code::*;
pub use config::*;
use nalgebra::Vector3;
pub use pearl::*;
pub use report::*;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{fmt, ops::Add};

mod code;
mod config;
mod pearl;
mod report;

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Array(pub Vector3<f64>);

#[derive(Serialize_repr, Deserialize_repr, Default, Debug, Clone, Copy)]
#[repr(i8)]
pub enum Dimension {
    Overworld = 0,
    #[default]
    Nether = -1,
    End = 1,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Time(pub u64);

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Overworld => "Overworld",
            Self::Nether => "Nether",
            Self::End => "End",
        };
        write!(f, "{}", s)
    }
}

impl Time {
    pub(crate) fn range(Self(a): Self, Self(b): Self) -> impl Iterator<Item = Self> {
        (a..b).map(Time)
    }
}

impl Add<u64> for Time {
    type Output = Self;

    fn add(self, rhs: u64) -> Self {
        Time(self.0 + rhs)
    }
}
