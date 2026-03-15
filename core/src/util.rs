use std::ops::{Add, AddAssign};

use itertools::Itertools;
use minecraft_mth as mth;
use nalgebra::{Matrix2, Vector2, Vector3, matrix, vector};
use serde::{Deserialize, Serialize};

use crate::{
    config::{Config, MotionPerTnt},
    simulation,
};

const G: f64 = 0.03;
const D: f64 = 0.99_f32 as f64;

#[derive(Debug, Clone, Copy)]
pub struct FtlConfigOutput {
    pub rb: RB,
    pub end_time: Time,
    pub error: f64,
    pub final_pos: Array,
    pub to_end_time: Option<Time>,
    pub end_portal_pos: Option<Array>,
}

#[derive(Debug, Clone, Copy)]
pub struct FtlConfig {
    pub tnt_num: TNTNum,
    pub end_time: Time,
    pub to_end_time: Option<Time>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct TNTNumRB {
    pub red: u64,
    pub blue: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct RB {
    pub num: TNTNumRB,
    pub direction: usize,
}
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct TNTNum(pub Vector2<i64>);

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Array(pub Vector3<f64>);

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Polar {
    pub r: f64,
    pub theta: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default)]
#[serde(default)]
pub struct Yaw(pub f32);

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Time(pub u64);

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Direction(pub Matrix2<i64>);

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Directions(pub [Direction; 4]);

impl From<TNTNumRB> for Vector2<i64> {
    fn from(value: TNTNumRB) -> Self {
        vector![value.red as i64, value.blue as i64]
    }
}

impl From<Vector2<i64>> for TNTNumRB {
    fn from(value: Vector2<i64>) -> Self {
        TNTNumRB {
            red: value.x as u64,
            blue: value.y as u64,
        }
    }
}

impl Array {
    pub fn distance_xz(self, other: Self) -> f64 {
        let square = (self.0.x - other.0.x).powi(2) + (self.0.x - other.0.x).powi(2);
        square.sqrt()
    }

    pub fn to_nums(
        self,
        motion_per_tnt: MotionPerTnt,
        start_time: Time,
        end_time: Time,
    ) -> Vec<TNTNum> {
        let base = vector![(self.0.x + self.0.z) * 0.5, (self.0.x - self.0.z) * 0.5];
        let t: f64 = (D.powi(start_time.0 as i32 + 1) - D.powi(end_time.0 as i32 + 1)) / (1.0 - D);
        let new_base = base / motion_per_tnt.x_z / t;
        let (cx, cz) = (new_base.x.round() as i64, new_base.y.round() as i64);
        (-5..=5)
            .cartesian_product(-5..=5)
            .map(|(dz, dx)| TNTNum(vector![cx + dx, cz + dz]))
            .collect()
    }

    pub fn array_to(self, other: Self) -> Self {
        Self(other.0 - self.0)
    }

    pub fn tick(&mut self) {
        self.0.y -= G;
        self.0 *= D;
    }

    pub fn from_num(num: TNTNum, motion_per_tnt: MotionPerTnt) -> Self {
        let total_count = num.0.x.abs() + num.0.y.abs();
        let tnt_count = matrix![
            1,1;
            0,0;
            1,-1
        ] * num.0
            + vector![0, total_count, 0];

        Array(vector![
            motion_per_tnt.x_z * tnt_count.x as f64,
            motion_per_tnt.y * tnt_count.y as f64,
            motion_per_tnt.x_z * tnt_count.z as f64,
        ])
    }
}

impl Yaw {
    pub fn lerp_rotation(&mut self, motion: Array) {
        let target_yaw = Yaw(mth::atan2(motion.0.x, motion.0.z) as f32 * mth::RAD_TO_DEG);
        self.0 = mth::lerp(
            0.2,
            target_yaw.0 - mth::wrap_degrees(target_yaw.0 - self.0),
            target_yaw.0,
        );
    }
}

impl AddAssign<Array> for Array {
    fn add_assign(&mut self, rhs: Array) {
        self.0 += rhs.0
    }
}

impl TNTNum {
    pub fn from_rb(rb: RB, directions: Directions) -> Self {
        let base = directions.0[rb.direction].0;
        Self(base * Vector2::from(rb.num))
    }
}

impl RB {
    pub fn from_num(num: TNTNum, directions: Directions) -> Self {
        let direction_type = if num.0.x > 0 {
            if num.0.y > 0 { 0 } else { 1 }
        } else {
            if num.0.y > 0 { 2 } else { 3 }
        };
        let direction_num = directions.resolve()[direction_type];
        let direction = directions.0[direction_num];
        RB {
            num: TNTNumRB::from(direction.0.transpose() * num.0),
            direction: direction_num,
        }
    }

    pub fn is_available(self, max: TNTNumRB) -> bool {
        self.num.is_available(max)
    }
}

impl Directions {
    pub fn resolve(self) -> [usize; 4] {
        let mut indices = [0; 4];
        let mut seen = [false; 4];

        for (i, m) in self.0.iter().enumerate() {
            let sum = m.0.column(0) + m.0.column(1);
            let idx = match (sum.x, sum.y) {
                (1, 1) => 0,
                (1, -1) => 1,
                (-1, 1) => 2,
                (-1, -1) => 3,
                _ => panic!(),
            };

            if seen[idx] {
                panic!()
            }
            seen[idx] = true;
            indices[idx] = i;
        }

        indices
    }
}

impl FtlConfig {
    pub fn generate(self, config: &Config, target_point: Vector2<i64>) -> FtlConfigOutput {
        let simulation_report = simulation(config, self.tnt_num, self.end_time, self.to_end_time);

        let final_pos = simulation_report.final_pos;

        let rb = RB::from_num(self.tnt_num, config.directions);
        FtlConfigOutput {
            rb,
            end_time: self.end_time,
            error: final_pos.distance_xz(target_point.into()),
            final_pos,
            to_end_time: self.to_end_time,
            end_portal_pos: simulation_report.end_portal_pos,
        }
    }
}

impl FtlConfigOutput {
    pub fn sort_and_get_top(v: &mut Vec<Self>, show_first: usize) {
        v.sort_unstable_by(|&a, &b| {
            a.end_time
                .0
                .cmp(&b.end_time.0)
                .then_with(|| a.error.total_cmp(&b.error))
        });
        v.truncate(show_first);
    }
}

impl From<Polar> for Array {
    fn from(Polar { r, theta }: Polar) -> Self {
        Array(vector![theta.sin(), 0.0, theta.cos()] * r)
    }
}

impl From<Vector2<i64>> for Array {
    fn from(val: Vector2<i64>) -> Self {
        Array(vector![val.x as f64, 0.0, val.y as f64])
    }
}

impl Time {
    pub fn range(Self(a): Self, Self(b): Self) -> impl Iterator<Item = Self> {
        (a..b).map(Time)
    }
}

impl Add<u64> for Time {
    type Output = Self;
    fn add(self, rhs: u64) -> Self {
        Time(self.0 + rhs)
    }
}

impl TNTNumRB {
    pub fn is_available(self, max: Self) -> bool {
        self.red <= max.red && self.blue <= max.blue
    }
}
