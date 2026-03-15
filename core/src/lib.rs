use nalgebra::Vector2;

use crate::{
    config::Config,
    pearl::{Dimension, SimulationReport},
    util::{Array, FtlConfigOutput, TNTNum, TNTNumRB, Time},
};

/// num means tnt counts in (1,1) and (1,-1)
pub mod config;
pub mod pearl;
pub mod util;

pub fn simulation(
    config: &Config,
    tnt_num: TNTNum,
    time: Time,
    to_end_time: Option<Time>,
) -> SimulationReport {
    let mut pearl = config.pearl;
    let motion_per_tnt = config.motion_per_tnt;
    let tnt_motion = Array::from_num(tnt_num, motion_per_tnt);
    pearl.simulation(tnt_motion, time, to_end_time)
}

pub fn calculation(
    config: &Config,
    max_tnt: Option<TNTNumRB>,
    target_point: Vector2<i64>,
    max_error: Option<f64>,
    max_time: Option<Time>,
    dimension: Option<Dimension>,
    show_first: Option<usize>,
) -> Vec<FtlConfigOutput> {
    let pearl = config.pearl;
    let motion_per_tnt = config.motion_per_tnt;
    let max_tnt = max_tnt.unwrap_or(config.max_tnt);
    let max_error = max_error.unwrap_or(config.max_error);
    let show_first = show_first.unwrap_or(config.show_first);
    let dimension = dimension.unwrap_or(Dimension::Nether);
    let max_time = max_time.unwrap_or(config.max_time);

    let res = pearl.calculation(target_point, motion_per_tnt, max_time, dimension);

    let mut result: Vec<FtlConfigOutput> = res
        .iter()
        .filter_map(|&x| {
            let new = x.generate(config, target_point);
            if new.error <= max_error && new.rb.is_available(max_tnt) {
                Some(new)
            } else {
                None
            }
        })
        .collect();
    FtlConfigOutput::sort_and_get_top(&mut result, show_first);
    result
}
