use crate::{Dimension, RB};
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Clone, Error, Diagnostic)]
pub enum PearlError {
    #[error("unsupported config version: {0}")]
    UnsupportedConfigVersion(u64),
    #[error("invalid direction vector: {0:?}")]
    InvalidDirectionVector([i8; 2]),
    #[error("invalid direction combination sum: ({x}, {y})")]
    InvalidDirectionCombination { x: i64, y: i64 },
    #[error("duplicate direction quadrant: {quadrant}")]
    DuplicateDirectionQuadrant { quadrant: usize },
    #[error("simulation time must be greater than 0")]
    SimulationTimeZero,
    #[error("to_end_time ({to_end_time}) cannot be greater than total time ({time})")]
    ToEndTimeAfterEnd { to_end_time: u64, time: u64 },
    #[error("cannot trigger end-portal teleport when already in End")]
    EndPortalTeleportFromEnd,
    #[error("unimplemented feature: {feature}")]
    Unimplemented { feature: &'static str },
    #[error("unsupported dimension {dimension} in {context}")]
    UnsupportedDimension {
        dimension: Dimension,
        context: &'static str,
    },
    #[error(
        "unsupported calculation dimension transition: {start_dimension} -> {target_dimension}"
    )]
    UnsupportedCalculationDimensionTransition {
        start_dimension: Dimension,
        target_dimension: Dimension,
    },
    #[error("invalid --max-tnt argument count: {0} (expected 0..=2)")]
    InvalidMaxTntArgCount(usize),
    #[error("cap bit index out of range: {bit} (must be 1..={max})")]
    InvalidCapBit { bit: usize, max: usize },
    #[error("duplicate cap bit index in one cap group: {bit}")]
    DuplicateCapBit { bit: usize },
    #[error("cap bit index overlaps across groups: {bit}")]
    OverlappingCapBit { bit: usize },
    #[error("code length mismatch: expected {expected} bits from rule, got {actual}")]
    CodeLengthMismatch { expected: usize, actual: usize },
    #[error("all bits in one cap group must have the same type")]
    MixedCapKinds,
    #[error("direction value out of range: {value} (must be 0..=3)")]
    DirectionOutOfRange { value: u64 },
    #[error("numeric overflow while accumulating code counts")]
    ValueOverflow,
    #[error(
        "cannot encode exact RB value: direction={}, red={}, blue={}",
        rb.direction,
        rb.num.red,
        rb.num.blue
    )]
    NoExactEncoding { rb: RB },
}
