use itertools::iproduct;
use nalgebra::{Matrix2, Vector2, Vector3, vector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum FtlConfig {
    Nether(ConfigNether),
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

pub fn check_close(target_point: Vector2<i64>, pos: Vector3<f64>, error: u64) -> bool {
    let target_x = target_point.x as f64;
    let target_z = target_point.y as f64;

    (pos.x - target_x).powi(2) + (pos.z - target_z).powi(2) <= (error as f64).powi(2)
}
