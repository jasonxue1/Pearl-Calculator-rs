use std::fmt;

use crate::Dimension;

#[derive(Debug, Clone)]
pub enum PearlError {
    UnsupportedConfigVersion(u64),
    InvalidDirectionVector([i8; 2]),
    InvalidDirectionCombination {
        x: i64,
        y: i64,
    },
    DuplicateDirectionQuadrant {
        quadrant: usize,
    },
    SimulationTimeZero,
    ToEndTimeAfterEnd {
        to_end_time: u64,
        time: u64,
    },
    EndPortalTeleportFromEnd,
    UnsupportedDimension {
        dimension: Dimension,
        context: &'static str,
    },
    InvalidMaxTntArgCount(usize),
    InvalidCapBit {
        bit: usize,
        max: usize,
    },
    DuplicateCapBit {
        bit: usize,
    },
    OverlappingCapBit {
        bit: usize,
    },
}

impl fmt::Display for PearlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedConfigVersion(version) => {
                write!(f, "unsupported config version: {version}")
            }
            Self::InvalidDirectionVector(value) => {
                write!(f, "invalid direction vector: [{}, {}]", value[0], value[1])
            }
            Self::InvalidDirectionCombination { x, y } => {
                write!(f, "invalid direction combination sum: ({x}, {y})")
            }
            Self::DuplicateDirectionQuadrant { quadrant } => {
                write!(f, "duplicate direction quadrant: {quadrant}")
            }
            Self::SimulationTimeZero => write!(f, "simulation time must be greater than 0"),
            Self::ToEndTimeAfterEnd { to_end_time, time } => write!(
                f,
                "to_end_time ({to_end_time}) cannot be greater than total time ({time})"
            ),
            Self::EndPortalTeleportFromEnd => {
                write!(f, "cannot trigger end-portal teleport when already in End")
            }
            Self::UnsupportedDimension { dimension, context } => {
                write!(f, "unsupported dimension {dimension} in {context}")
            }
            Self::InvalidMaxTntArgCount(count) => {
                write!(
                    f,
                    "invalid --max-tnt argument count: {count} (expected 0..=2)"
                )
            }
            Self::InvalidCapBit { bit, max } => {
                write!(f, "cap bit index out of range: {bit} (must be 1..={max})")
            }
            Self::DuplicateCapBit { bit } => {
                write!(f, "duplicate cap bit index in one cap group: {bit}")
            }
            Self::OverlappingCapBit { bit } => {
                write!(f, "cap bit index overlaps across groups: {bit}")
            }
        }
    }
}

impl std::error::Error for PearlError {}
