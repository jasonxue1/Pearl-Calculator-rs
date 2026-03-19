pub use code::*;
pub use config::*;
pub use convert::*;
use nalgebra::Vector3;
pub use pearl::*;
pub use report::*;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

mod code;
mod config;
mod convert;
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
