use std::{
    collections::BTreeSet,
    f64::consts::{PI, TAU},
    ops::AddAssign,
};

use itertools::Itertools;
use minecraft_mth as mth;
use nalgebra::{Vector2, matrix, vector};

use crate::*;

use super::solver::TNTNum;

const G: f64 = 0.03;
const D: f64 = 0.99_f32 as f64;
const NUM_SEARCH_RADIUS: i64 = 16;

#[derive(Debug, Clone, Copy)]
struct Polar {
    pub r: f64,
    pub theta: f64,
}

fn wrap_angle(x: f64) -> f64 {
    (x + PI).rem_euclid(TAU) - PI
}

fn decay_sum(first_tick: u64, last_tick: u64) -> f64 {
    if first_tick > last_tick {
        0.0
    } else {
        (D.powi(first_tick as i32) - D.powi(last_tick as i32 + 1)) / (1.0 - D)
    }
}

impl Array {
    pub(crate) fn distance_xz(self, other: Self) -> f64 {
        let square = (self.0.x - other.0.x).powi(2) + (self.0.z - other.0.z).powi(2);
        square.sqrt()
    }

    pub(crate) fn to_nums(
        self,
        motion_per_tnt: MotionPerTnt,
        initial_motion: Array,
        start_time: Time,
        end_time: Time,
        rotate: bool,
    ) -> Vec<TNTNum> {
        let centers = if rotate {
            self.rotated_num_centers(motion_per_tnt, initial_motion, start_time, end_time)
        } else {
            self.linear_num_center(motion_per_tnt, initial_motion, start_time, end_time)
                .into_iter()
                .collect()
        };

        let mut nums = BTreeSet::new();
        for center in centers {
            // The inverse step computes approximate TNT-number centers. Keep a
            // conservative window here so the coarse pass does not miss valid
            // nearby solutions before precise simulation/error filtering.
            for (dz, dx) in (-NUM_SEARCH_RADIUS..=NUM_SEARCH_RADIUS)
                .cartesian_product(-NUM_SEARCH_RADIUS..=NUM_SEARCH_RADIUS)
            {
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
        initial_motion: Array,
        start_time: Time,
        end_time: Time,
    ) -> Option<Vector2<i64>> {
        let t = decay_sum(start_time.0 + 1, end_time.0);
        if t == 0.0 {
            return None;
        }

        let total_motion = vector![self.0.x / t, self.0.z / t];
        let initial_xz = vector![initial_motion.0.x, initial_motion.0.z];
        let tnt_motion = total_motion - initial_xz;
        let base = vector![
            (tnt_motion.x + tnt_motion.y) * 0.5,
            (tnt_motion.x - tnt_motion.y) * 0.5,
        ];
        let new_base = base / motion_per_tnt.x_z;
        Some(vector![
            new_base.x.round() as i64,
            new_base.y.round() as i64,
        ])
    }

    fn rotated_num_centers(
        self,
        motion_per_tnt: MotionPerTnt,
        initial_motion: Array,
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
            let tnt_x = initial.0.x - initial_motion.0.x;
            let tnt_z = initial.0.z - initial_motion.0.z;
            let base = vector![(tnt_x + tnt_z) * 0.5, (tnt_x - tnt_z) * 0.5,] / motion_per_tnt.x_z;
            centers.push(vector![base.x.round() as i64, base.y.round() as i64]);
        }

        centers.sort_unstable_by_key(|v| (v.x, v.y));
        centers.dedup_by_key(|v| (v.x, v.y));
        centers
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_num_center_accounts_for_initial_motion() {
        let motion_per_tnt = MotionPerTnt { x_z: 1.0, y: 0.0 };
        let start_time = Time(0);
        let end_time = Time(1);
        let t = decay_sum(start_time.0 + 1, end_time.0);

        let total_motion = vector![30.0, -10.0];
        let target = Array(vector![total_motion.x * t, 0.0, total_motion.y * t]);
        let initial_motion = Array(vector![3.0, 0.0, 1.0]);

        let actual = target
            .linear_num_center(motion_per_tnt, initial_motion, start_time, end_time)
            .expect("center");

        assert_eq!(actual, vector![8, 19]);
        assert_eq!(
            target.linear_num_center(
                motion_per_tnt,
                Array(vector![0.0, 0.0, 0.0]),
                start_time,
                end_time,
            ),
            Some(vector![10, 20])
        );
    }

    #[test]
    fn rotated_num_centers_include_solution_with_initial_motion() {
        let motion_per_tnt = MotionPerTnt { x_z: 1.0, y: 0.0 };
        let start_time = Time(0);
        let end_time = Time(2);
        let initial_motion = Array(vector![2.0, 0.0, 4.0]);
        let tnt_num = TNTNum(vector![10, 20]);
        let tnt_motion = Array::from_num(tnt_num, motion_per_tnt);
        let total_motion = Array(vector![
            initial_motion.0.x + tnt_motion.0.x,
            0.0,
            initial_motion.0.z + tnt_motion.0.z,
        ]);

        let Polar { r, theta } = total_motion.into();
        let t = decay_sum(start_time.0 + 2, end_time.0);
        let angle_scale = 2.0 - 0.8_f64.powi(start_time.0 as i32 + 1);
        let target = Array::from(Polar {
            r: r * t,
            theta: angle_scale * theta - PI / 2.0,
        });

        let centers =
            target.rotated_num_centers(motion_per_tnt, initial_motion, start_time, end_time);

        assert!(centers.contains(&tnt_num.0));
        assert!(
            !target
                .rotated_num_centers(
                    motion_per_tnt,
                    Array(vector![0.0, 0.0, 0.0]),
                    start_time,
                    end_time,
                )
                .contains(&tnt_num.0)
        );
    }
}
