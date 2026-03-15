use std::{
    collections::BTreeSet,
    f64::consts::{PI, TAU},
    fmt,
    ops::{Add, AddAssign},
};

use itertools::Itertools;
use minecraft_mth as mth;
use nalgebra::{Matrix, Matrix2, Vector2, matrix, vector};
use serde::{Deserialize, Serialize};

use crate::*;

const G: f64 = 0.03;
const D: f64 = 0.99_f32 as f64;

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Overworld => "Overworld",
            Self::Nether => "Nether",
            Self::End => "End",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FtlConfig {
    pub tnt_num: TNTNum,
    pub end_time: Time,
    pub to_end_time: Option<Time>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub(crate) struct TNTNum(pub Vector2<i64>);

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
struct Polar {
    pub r: f64,
    pub theta: f64,
}

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

fn wrap_angle(x: f64) -> f64 {
    (x + PI).rem_euclid(TAU) - PI
}

impl Array {
    fn distance_xz(self, other: Self) -> f64 {
        let square = (self.0.x - other.0.x).powi(2) + (self.0.z - other.0.z).powi(2);
        square.sqrt()
    }

    pub(crate) fn to_nums(
        self,
        motion_per_tnt: MotionPerTnt,
        start_time: Time,
        end_time: Time,
        rotate: bool,
    ) -> Vec<TNTNum> {
        let centers = if rotate {
            self.rotated_num_centers(motion_per_tnt, start_time, end_time)
        } else {
            self.linear_num_center(motion_per_tnt, start_time, end_time)
                .into_iter()
                .collect()
        };

        let mut nums = BTreeSet::new();
        for center in centers {
            for (dz, dx) in (-5..=5).cartesian_product(-5..=5) {
                nums.insert((center.x + dx, center.y + dz));
            }
        }

        nums.into_iter()
            .map(|(x, z)| TNTNum(vector![x, z]))
            .collect()
    }

    pub(crate) fn array_to(self, other: Self) -> Self {
        Self(other.0 - self.0)
    }

    pub(crate) fn tick_motion(&mut self) {
        self.0.y -= G;
        self.0 *= D;
    }

    pub(crate) fn from_num(num: TNTNum, motion_per_tnt: MotionPerTnt) -> Self {
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

    fn linear_num_center(
        self,
        motion_per_tnt: MotionPerTnt,
        start_time: Time,
        end_time: Time,
    ) -> Option<Vector2<i64>> {
        let t = decay_sum(start_time.0 + 1, end_time.0);
        if t == 0.0 {
            return None;
        }

        let base = vector![(self.0.x + self.0.z) * 0.5, (self.0.x - self.0.z) * 0.5];
        let new_base = base / motion_per_tnt.x_z / t;
        Some(vector![
            new_base.x.round() as i64,
            new_base.y.round() as i64,
        ])
    }

    fn rotated_num_centers(
        self,
        motion_per_tnt: MotionPerTnt,
        start_time: Time,
        end_time: Time,
    ) -> Vec<Vector2<i64>> {
        let t = decay_sum(start_time.0 + 2, end_time.0);
        if t == 0.0 {
            return Vec::new();
        }

        let Polar { r, theta } = self.into();
        let angle_scale = 2.0 - 0.8_f64.powi(start_time.0 as i32 + 1);
        let radius = r / t;
        let mut centers = Vec::new();

        for branch in -1..=1 {
            let initial_theta = wrap_angle((theta + PI / 2.0 + TAU * branch as f64) / angle_scale);
            let initial = Array::from(Polar {
                r: radius,
                theta: initial_theta,
            });
            let base = vector![
                (initial.0.x + initial.0.z) * 0.5,
                (initial.0.x - initial.0.z) * 0.5,
            ] / motion_per_tnt.x_z;
            centers.push(vector![base.x.round() as i64, base.y.round() as i64]);
        }

        centers.sort_unstable_by_key(|v| (v.x, v.y));
        centers.dedup_by_key(|v| (v.x, v.y));
        centers
    }
}

fn decay_sum(first_tick: u64, last_tick: u64) -> f64 {
    if first_tick > last_tick {
        0.0
    } else {
        (D.powi(first_tick as i32) - D.powi(last_tick as i32 + 1)) / (1.0 - D)
    }
}

impl Angle {
    pub(crate) fn lerp_rotation(&mut self, motion: Array) {
        let target_yaw = Angle(mth::atan2(motion.0.x, motion.0.z) as f32 * mth::RAD_TO_DEG);
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

impl RB {
    fn from_num(num: TNTNum, directions: Directions) -> Self {
        let direction_type = if num.0.x > 0 {
            if num.0.y > 0 { 0 } else { 1 }
        } else {
            if num.0.y > 0 { 2 } else { 3 }
        };
        let direction_num = directions.resolve()[direction_type];
        let direction = directions.0[direction_num].match_direction();
        RB {
            num: TNTNumRB::from(direction.transpose() * num.0),
            direction: direction_num,
        }
    }

    pub(crate) fn to_num(self, directions: Directions) -> TNTNum {
        let base = directions.0[self.direction].match_direction();
        TNTNum(base * Vector2::from(self.num))
    }

    pub(crate) fn is_available(self, max: TNTNumRB) -> bool {
        self.num.is_available(max)
    }
}

fn match_direction(input: [i8; 2]) -> Vector2<i64> {
    match input {
        [1, 1] => vector![1, 0],
        [-1, -1] => vector![-1, 0],
        [1, -1] => vector![0, 1],
        [-1, 1] => vector![0, -1],
        _ => panic!(),
    }
}

impl Direction {
    fn match_direction(self) -> Matrix2<i64> {
        let red = match_direction(self.red);
        let blue = match_direction(self.blue);
        Matrix::from_columns(&[red, blue])
    }
}

impl Directions {
    fn resolve(self) -> [usize; 4] {
        let mut indices = [0; 4];
        let mut seen = [false; 4];

        for (i, m) in self.0.iter().enumerate() {
            let matrix = m.match_direction();
            let sum = matrix.column(0) + matrix.column(1);
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
    pub(crate) fn generate(self, config: &Config, target_point: Vector2<i64>) -> CalculationReport {
        let rb = RB::from_num(self.tnt_num, config.directions);
        let simulation_report = simulation(config, rb, Some(self.end_time), self.to_end_time);

        let final_pos = simulation_report.final_pos;

        CalculationReport {
            rb,
            end_time: self.end_time,
            error: final_pos.distance_xz(target_point.into()),
            final_pos,
            to_end_time: self.to_end_time,
            end_portal_pos: simulation_report.end_portal_pos,
        }
    }
}

impl CalculationReport {
    pub(crate) fn sort_and_get_top(v: &mut Vec<Self>, show_first: usize) {
        v.sort_unstable_by(|a, b| {
            a.end_time
                .0
                .cmp(&b.end_time.0)
                .then_with(|| a.error.total_cmp(&b.error))
                .then_with(|| a.rb.direction.cmp(&b.rb.direction))
                .then_with(|| a.rb.num.red.cmp(&b.rb.num.red))
                .then_with(|| a.rb.num.blue.cmp(&b.rb.num.blue))
        });
        v.dedup_by(|a, b| {
            a.end_time.0 == b.end_time.0
                && a.rb.direction == b.rb.direction
                && a.rb.num.red == b.rb.num.red
                && a.rb.num.blue == b.rb.num.blue
        });
        v.truncate(show_first);
    }
}

impl From<Polar> for Array {
    fn from(Polar { r, theta }: Polar) -> Self {
        Array(vector![theta.sin(), 0.0, theta.cos()] * r)
    }
}

impl From<Array> for Polar {
    fn from(Array(v): Array) -> Self {
        let (x, z) = (v.x, v.z);
        let r = x.hypot(z);
        let theta = x.atan2(z);
        Polar { r, theta }
    }
}

impl From<Vector2<i64>> for Array {
    fn from(val: Vector2<i64>) -> Self {
        Array(vector![val.x as f64, 0.0, val.y as f64])
    }
}

impl Time {
    pub(crate) fn range(Self(a): Self, Self(b): Self) -> impl Iterator<Item = Self> {
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
    fn is_available(self, max: Self) -> bool {
        self.red <= max.red && self.blue <= max.blue
    }
}
