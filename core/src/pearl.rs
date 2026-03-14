use std::fmt;

use minecraft_mth as mth;
use nalgebra::{Vector3, matrix, vector};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

const G: f64 = 0.03;
const D: f64 = 0.99_f32 as f64;
const END_SPAWN_POS: Vector3<f64> = vector![100.5, 50.0, 0.5];

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Pearl {
    pub position: Vector3<f64>,
    pub motion: Vector3<f64>,
    #[serde(default)]
    pub yaw: f32,
    #[serde(default)]
    pub dimension: Dimension,
}

#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Clone, Copy)]
#[repr(i8)]
#[derive(Default)]
pub enum Dimension {
    Overworld = 0,
    #[default]
    Nether = -1,
    End = 1,
}

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

pub enum Teleport {
    None,
    NetherPortal,
    EndPortal,
}

#[derive(Debug)]
pub struct SimulationReport {
    pub history: Vec<Pearl>,
    pub final_pos: Vector3<f64>,
}

impl Pearl {
    pub fn tick(&mut self, teleport: Teleport) {
        self.motion.y -= G;
        self.motion *= D;
        self.lerp_rotation();

        match teleport {
            Teleport::None => self.position += self.motion,
            Teleport::EndPortal if self.dimension != Dimension::End => {
                self.rotate_motion(90.0);
                self.dimension = Dimension::End;
                self.position = END_SPAWN_POS;
            }
            _ => todo!(),
        }
    }

    fn lerp_rotation(&mut self) {
        let target_yaw: f32 = mth::atan2(self.motion.x, self.motion.z) as f32 * mth::RAD_TO_DEG;
        self.yaw = mth::lerp(
            0.2,
            target_yaw - mth::wrap_degrees(target_yaw - self.yaw),
            target_yaw,
        );
    }

    fn rotate_motion(&mut self, new_yaw: f32) {
        let old_yaw = self.yaw;
        self.yaw = new_yaw;
        let rad = (old_yaw - self.yaw) * mth::DEG_TO_RAD;
        let s = mth::sin(rad as f64) as f64;
        let c = mth::cos(rad as f64) as f64;
        let r = matrix![
            c, 0.0, s;
            0.0, 1.0, 0.0;
            -s,  0.0,c
        ];
        self.motion = r * self.motion;
    }

    pub fn simulation(
        &mut self,
        tnt_motion: Vector3<f64>,
        time: u32,
        to_end_time: u32,
    ) -> SimulationReport {
        if to_end_time > time {
            panic!()
        };
        if time == 0 {
            panic!()
        };
        self.motion += tnt_motion;
        let mut history: Vec<Pearl> = Vec::new();
        history.push(*self);
        for _ in 0..time {
            if time == to_end_time {
                self.tick(Teleport::EndPortal)
            } else {
                self.tick(Teleport::None)
            };
            history.push(*self);
        }
        SimulationReport {
            history,
            final_pos: self.position,
        }
    }
}

#[cfg(test)]
impl Pearl {
    fn approx_eq(&self, other: Self) -> bool {
        const EPS: f64 = 1e-5;
        use approx::abs_diff_eq;
        abs_diff_eq!(self.position, other.position, epsilon = EPS)
            && abs_diff_eq!(self.motion, other.motion, epsilon = EPS)
            && abs_diff_eq!(self.yaw as f64, other.yaw as f64, epsilon = EPS)
            && self.dimension == other.dimension
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    struct TickTestCase {
        input: Pearl,
        output: Pearl,
    }

    #[test]
    fn tick_test_no_teleport() {
        let cases = vec![
            TickTestCase {
                input: Pearl {
                    position: vector![10.0, 10.0, 10.0],
                    motion: vector![5.0, 15.0, 4.0],
                    yaw: 50.0,
                    dimension: Dimension::Nether,
                },
                output: Pearl {
                    position: vector![14.95, 24.8203, 13.96],
                    motion: vector![4.95, 14.8203, 3.96],
                    yaw: 50.268036,
                    dimension: Dimension::Nether,
                },
            },
            TickTestCase {
                input: Pearl {
                    position: vector![-3.5, 64.0, 8.25],
                    motion: vector![0.0, 0.0, 0.0],
                    yaw: 0.0,
                    dimension: Dimension::Nether,
                },
                output: Pearl {
                    position: vector![-3.5, 63.9703, 8.25],
                    motion: vector![0.0, -0.0297, 0.0],
                    yaw: 0.0,
                    dimension: Dimension::Nether,
                },
            },
            TickTestCase {
                input: Pearl {
                    position: vector![123.456, 78.9, -45.67],
                    motion: vector![-1.25, 0.5, 2.75],
                    yaw: -170.0,
                    dimension: Dimension::Nether,
                },
                output: Pearl {
                    position: vector![122.2185, 79.3653, -42.9475],
                    motion: vector![-1.2375, 0.4653, 2.7225],
                    yaw: -140.8888,
                    dimension: Dimension::Nether,
                },
            },
            TickTestCase {
                input: Pearl {
                    position: vector![0.0, 100.0, 0.0],
                    motion: vector![10.0, -2.0, -10.0],
                    yaw: 179.0,
                    dimension: Dimension::Nether,
                },
                output: Pearl {
                    position: vector![9.9, 97.9903, -9.9],
                    motion: vector![9.9, -2.0097, -9.9],
                    yaw: 170.2,
                    dimension: Dimension::Nether,
                },
            },
        ];

        for case in cases {
            let mut pearl = case.input;
            pearl.tick(Teleport::None);
            assert!(pearl.approx_eq(case.output));
        }
    }

    #[test]
    fn tick_test_from_nether_to_end() {
        let cases = vec![
            TickTestCase {
                input: Pearl {
                    position: vector![10.0, 10.0, 10.0],
                    motion: vector![5.0, 15.0, 4.0],
                    yaw: 50.0,
                    dimension: Dimension::Nether,
                },
                output: Pearl {
                    position: vector![100.5, 50.0, 0.5],
                    motion: vector![1.275825, 14.8203, 6.209073],
                    yaw: 90.0,
                    dimension: Dimension::End,
                },
            },
            TickTestCase {
                input: Pearl {
                    position: vector![-3.5, 64.0, 8.25],
                    motion: vector![0.0, 0.0, 0.0],
                    yaw: 0.0,
                    dimension: Dimension::Nether,
                },
                output: Pearl {
                    position: vector![100.5, 50.0, 0.5],
                    motion: vector![0.0, -0.0297, 0.0],
                    yaw: 90.0,
                    dimension: Dimension::End,
                },
            },
            TickTestCase {
                input: Pearl {
                    position: vector![123.456, 78.9, -45.67],
                    motion: vector![-1.25, 0.5, 2.75],
                    yaw: -170.0,
                    dimension: Dimension::Nether,
                },
                output: Pearl {
                    position: vector![100.5, 50.0, 0.5],
                    motion: vector![2.893098, 0.4653, -0.757229],
                    yaw: 90.0,
                    dimension: Dimension::End,
                },
            },
            TickTestCase {
                input: Pearl {
                    position: vector![0.0, 100.0, 0.0],
                    motion: vector![10.0, -2.0, -10.0],
                    yaw: 179.0,
                    dimension: Dimension::Nether,
                },
                output: Pearl {
                    position: vector![100.5, 50.0, 0.5],
                    motion: vector![-8.069406, -2.0097, -11.441359],
                    yaw: 90.0,
                    dimension: Dimension::End,
                },
            },
        ];

        for case in cases {
            let mut pearl = case.input;
            pearl.tick(Teleport::EndPortal);
            assert!(pearl.approx_eq(case.output));
        }
    }
}
