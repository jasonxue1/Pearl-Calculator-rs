use nalgebra::Vector2;

use crate::{
    Array, CalculationReport, Config, Dimension, PearlError, RB, SimulationReport, TNTNumRB, Time,
};

pub fn simulation(
    config: &Config,
    rb: RB,
    time: Option<Time>,
    to_end_time: Option<Time>,
) -> Result<SimulationReport, PearlError> {
    let time = time.unwrap_or(config.max_time);
    let mut pearl = config.pearl;
    let motion_per_tnt = config.motion_per_tnt;
    let tnt_motion = Array::from_num(rb.to_num(config.directions)?, motion_per_tnt);
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
) -> Result<Vec<CalculationReport>, PearlError> {
    let pearl = config.pearl;
    let motion_per_tnt = config.motion_per_tnt;
    let max_tnt = max_tnt.unwrap_or(config.max_tnt);
    let max_error = max_error.unwrap_or(config.max_error);
    let show_first = show_first.unwrap_or(config.show_first);
    let dimension = dimension.unwrap_or(Dimension::Nether);
    let max_time = max_time.unwrap_or(config.max_time);

    let res = pearl.calculation(target_point, motion_per_tnt, max_time, dimension)?;

    let mut result = Vec::new();
    for &x in &res {
        let new = x.generate(config, target_point)?;
        if new.error <= max_error && new.rb.is_available(max_tnt) {
            result.push(new);
        }
    }
    CalculationReport::sort_and_get_top(&mut result, show_first);
    Ok(result)
}
