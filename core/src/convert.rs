use nalgebra::{Matrix2, Vector2, Vector3, matrix, vector};

use crate::config::{MotionPerTnt, resolve_directions};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RB {
    pub count: Vector2<i32>,
    pub direction: usize,
}

pub fn rb_to_num(rb: RB, directions: [Matrix2<i32>; 4]) -> Vector2<i32> {
    let base = directions[rb.direction];
    base * rb.count
}

pub fn num_to_rb(num: Vector2<i32>, directions: [Matrix2<i32>; 4]) -> RB {
    let direction_type = if num.x > 0 {
        if num.y > 0 { 0 } else { 1 }
    } else {
        if num.y > 0 { 2 } else { 3 }
    };
    let direction_num = resolve_directions(directions)[direction_type];
    let direction = directions[direction_num];
    RB {
        count: direction.transpose() * num,
        direction: direction_num,
    }
}

pub fn num_to_motion(num: Vector2<i32>, motion_per_tnt: MotionPerTnt) -> Vector3<f64> {
    let total_count = num.x.abs() + num.y.abs();
    let tnt_count = matrix![
        1,1;
        0,0;
        1,-1
    ] * num
        + vector![0, total_count, 0];

    vector![
        motion_per_tnt.x_z * tnt_count.x as f64,
        motion_per_tnt.y * tnt_count.y as f64,
        motion_per_tnt.x_z * tnt_count.z as f64,
    ]
}

#[cfg(test)]
mod tests {
    use approx::abs_diff_eq;
    use itertools::iproduct;
    use nalgebra::vector;

    use super::*;

    #[test]
    fn num_rb_test() {
        let directions = [
            Matrix2::from_columns(&[Vector2::new(1, 0), Vector2::new(0, 1)]),
            Matrix2::from_columns(&[Vector2::new(-1, 0), Vector2::new(0, 1)]),
            Matrix2::from_columns(&[Vector2::new(0, -1), Vector2::new(-1, 0)]),
            Matrix2::from_columns(&[Vector2::new(0, -1), Vector2::new(1, 0)]),
        ];

        for (x, y) in iproduct!(-100..=100, -100..=100) {
            let num = vector![x, y];
            let rb = num_to_rb(num, directions);
            assert!(rb.count[0] >= 0);
            assert!(rb.count[1] >= 0);
            assert_eq!(rb_to_num(rb, directions), num);
        }
    }

    struct ToMotionTestCase {
        input: Vector2<i32>,
        output: Vector3<f64>,
    }

    #[test]
    fn to_motion_test() {
        let motion_per_tnt = MotionPerTnt {
            x_z: 0.6406475114548377,
            y: 0.0000041762421424,
        };

        let cases = vec![
            ToMotionTestCase {
                input: vector![0, 0],
                output: vector![0.0, 0.0, 0.0],
            },
            ToMotionTestCase {
                input: vector![1, 0],
                output: vector![0.640648, 0.000004, 0.640648],
            },
            ToMotionTestCase {
                input: vector![0, 1],
                output: vector![0.640648, 0.000004, -0.640648],
            },
            ToMotionTestCase {
                input: vector![-1, 0],
                output: vector![-0.640648, 0.000004, -0.640648],
            },
            ToMotionTestCase {
                input: vector![0, -1],
                output: vector![-0.640648, 0.000004, 0.640648],
            },
            ToMotionTestCase {
                input: vector![1, -1],
                output: vector![0.0, 0.000008, 1.281295],
            },
            ToMotionTestCase {
                input: vector![-1, 1],
                output: vector![0.0, 0.000008, -1.281295],
            },
            ToMotionTestCase {
                input: vector![2, 3],
                output: vector![3.203238, 0.000021, -0.640648],
            },
            ToMotionTestCase {
                input: vector![-2, -3],
                output: vector![-3.203238, 0.000021, 0.640648],
            },
        ];

        for case in cases {
            assert!(abs_diff_eq!(
                num_to_motion(case.input, motion_per_tnt),
                case.output,
                epsilon = 1e-5
            ));
        }
    }
}
