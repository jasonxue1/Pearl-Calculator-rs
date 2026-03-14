use itertools::iproduct;
use nalgebra::{Matrix2, Vector2, Vector3, vector};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    convert::{RB, num_to_rb},
    simulation,
};

#[derive(Debug, Clone, Copy)]
pub enum FtlConfigOutput {
    Nether(ConfigOutputNether),
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigOutputNether {
    pub rb: RB,
    pub time: u64,
    pub error: f64,
    pub final_pos: Vector3<f64>,
}

impl FtlConfigOutput {
    pub fn get_error(self) -> f64 {
        match self {
            Self::Nether(f) => f.error,
        }
    }

    pub fn get_time(self) -> u64 {
        match self {
            Self::Nether(f) => f.time,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigNether {
    pub num: Vector2<i64>,
    pub time: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct MaxTnt {
    pub red: u64,
    pub blue: u64,
}

#[inline(always)]
pub fn generate(
    max_tnt: MaxTnt,
    directions: &[Matrix2<i64>; 4],
) -> impl Iterator<Item = Vector2<i64>> {
    let max_red = max_tnt.red as i64;
    let max_blue = max_tnt.blue as i64;

    iproduct!(0..=max_red, 0..=max_blue, directions.iter())
        .map(|(red, blue, direction_base)| direction_base * vector![red, blue])
}

/// num means in a(1,1) and b(1,-1)
/// target:(x,z)
/// line (a-b)x - (a+b)z=0
/// |(a-b)x-(a+b)z|<e*sqrt((a-b)^2+(a+b)^2)
/// |(a-b)x-(a+b)z|<e*sqrt(2*(a^2+b^2))
#[inline(always)]
pub fn check_nether(num: Vector2<i64>, target_point: Vector2<i64>, error: u64) -> bool {
    let forward = (num.x + num.y) * target_point.x + (num.x - num.y) * target_point.y >= 0;
    forward && {
        let lhs = (num.x - num.y) * target_point.x - (num.x + num.y) * target_point.y;
        let rhs = error * error * 2 * (num.x * num.x + num.y * num.y) as u64;
        (lhs * lhs) as u64 <= rhs
    }
}

pub fn generate_output_config_nether(
    config: &Config,
    config_nether: ConfigNether,
    target_point: Vector2<i64>,
) -> ConfigOutputNether {
    let final_pos = simulation(config, config_nether.num, config_nether.time, 0).final_pos;
    let rb = num_to_rb(config_nether.num, config.directions);
    let target_pos = target_point.cast::<f64>();
    let final_pos_x_z = vector![final_pos.x, final_pos.z];
    ConfigOutputNether {
        rb,
        time: config_nether.time,
        error: final_pos_x_z.metric_distance(&target_pos),
        final_pos,
    }
}

pub fn sort_output(v: &mut [FtlConfigOutput]) {
    v.sort_unstable_by(|a, b| {
        a.get_time()
            .cmp(&b.get_time())
            .then_with(|| a.get_error().total_cmp(&b.get_error()))
    });
}
