use nalgebra::Vector2;

use crate::{
    config::Config,
    convert::num_to_motion,
    pearl::{Dimension, SimulationReport},
    util::FtlConfig,
};

/// num means tnt counts in (1,1) and (1,-1)
pub mod config;
pub mod convert;
pub mod pearl;
pub mod util;

pub fn simulation(
    config: &Config,
    tnt_num: Vector2<i64>,
    time: u64,
    to_end_time: u64,
) -> SimulationReport {
    let mut pearl = config.pearl;
    let motion_per_tnt = config.motion_per_tnt;
    let tnt_motion = num_to_motion(tnt_num, motion_per_tnt);
    pearl.simulation(tnt_motion, time, to_end_time)
}

pub fn calculation(
    config: &Config,
    max_tnt_num: Vector2<u64>,
    target_point: Vector2<i64>,
    error: u64,
    max_time: u64,
    dimension: Dimension,
) -> Vec<FtlConfig> {
    let pearl = config.pearl;
    let motion_per_tnt = config.motion_per_tnt;
    match dimension {
        Dimension::Nether => {
            let res = pearl.calculation_nether(
                max_tnt_num,
                target_point,
                motion_per_tnt,
                error,
                max_time,
            );
            res.iter().map(|&x| FtlConfig::Nether(x)).collect()
        }
        _ => {
            todo!()
        }
    }
}
