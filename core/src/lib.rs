use nalgebra::Vector2;

use crate::{
    config::Config,
    convert::num_to_motion,
    pearl::SimulationReport,
};

/// num means tnt counts in (1,1) and (1,-1)
pub mod config;
pub mod convert;
pub mod pearl;

pub fn simulation(
    config: Config,
    tnt_num: Vector2<i32>,
    time: u32,
    to_end_time: u32,
) -> SimulationReport {
    let mut pearl = config.pearl;
    let motion_per_tnt = config.motion_per_tnt;
    let tnt_motion = num_to_motion(tnt_num, motion_per_tnt);
    pearl.simulation(tnt_motion, time, to_end_time)
}
