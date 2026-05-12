use super::solver::FtlConfig;
use crate::*;
use minecraft_mth as mth;
use nalgebra::{Vector2, matrix, vector};

const END_SPAWN_POSTION: Array = Array(vector![100.5, 50.0, 0.5]);
const END_SPAWN_YAW: Angle = Angle(90.0);

enum Teleport {
    None,
    #[allow(unused)]
    NetherPortal,
    EndPortal,
}

#[derive(Clone, Copy)]
enum CalculationMode {
    Direct,
    NetherToEnd,
}

impl Pearl {
    fn calculation_mode(self, target_dimension: Dimension) -> Result<CalculationMode, PearlError> {
        match (self.dimension, target_dimension) {
            (Dimension::Nether, Dimension::Nether) | (Dimension::End, Dimension::End) => {
                Ok(CalculationMode::Direct)
            }
            (Dimension::Nether, Dimension::End) => Ok(CalculationMode::NetherToEnd),
            (start_dimension, target_dimension) => {
                Err(PearlError::UnsupportedCalculationDimensionTransition {
                    start_dimension,
                    target_dimension,
                })
            }
        }
    }

    fn move_motion(&mut self) {
        self.position += self.motion
    }

    fn lerp_rotation(&mut self) {
        self.yaw.lerp_rotation(self.motion);
    }

    fn tick(&mut self, teleport: Teleport) -> Result<(), PearlError> {
        self.motion.tick_motion();
        self.lerp_rotation();

        match teleport {
            Teleport::None => self.move_motion(),
            Teleport::EndPortal => match self.dimension {
                Dimension::End => return Err(PearlError::EndPortalTeleportFromEnd),
                _ => {
                    self.rotate_motion(END_SPAWN_YAW);
                    self.dimension = Dimension::End;
                    self.position = END_SPAWN_POSTION;
                }
            },
            Teleport::NetherPortal => {
                return Err(PearlError::Unimplemented {
                    feature: "nether-portal teleport in Pearl::tick",
                });
            }
        }
        Ok(())
    }

    fn rotate_motion(&mut self, new_yaw: Angle) {
        let old_yaw = self.yaw;
        self.yaw = new_yaw;
        let rad = (old_yaw.0 - self.yaw.0) * mth::DEG_TO_RAD;
        let s = mth::sin(rad as f64) as f64;
        let c = mth::cos(rad as f64) as f64;
        let r = matrix![
            c, 0.0, s;
            0.0, 1.0, 0.0;
            -s,  0.0,c
        ];
        self.motion.0 = r * self.motion.0;
    }

    pub(crate) fn simulation(
        &mut self,
        tnt_motion: Array,
        Time(time): Time,
        to_end_time: Option<Time>,
    ) -> Result<SimulationReport, PearlError> {
        let Time(to_end_time) = to_end_time.unwrap_or(Time(0));
        if to_end_time > time {
            return Err(PearlError::ToEndTimeAfterEnd { to_end_time, time });
        }
        if time == 0 {
            return Err(PearlError::SimulationTimeZero);
        }
        self.motion += tnt_motion;
        let mut history: Vec<Pearl> = Vec::new();
        history.push(*self);
        let mut to_end_pos = None;
        for tick in 0..time {
            if tick + 1 == to_end_time {
                let mut self_clone = *self;
                self_clone.tick(Teleport::None)?;
                to_end_pos = Some(self_clone.position);
                self.tick(Teleport::EndPortal)?
            } else {
                self.tick(Teleport::None)?
            };
            history.push(*self);
        }
        Ok(SimulationReport {
            history,
            final_pos: self.position,
            end_portal_pos: to_end_pos,
        })
    }

    pub(crate) fn calculation(
        self,
        target_point: Vector2<i64>,
        motion_per_tnt: MotionPerTnt,
        min_time: Time,
        max_time: Time,
        dimension: Dimension,
    ) -> Result<Vec<FtlConfig>, PearlError> {
        let mode = self.calculation_mode(dimension)?;

        let start_pos = match mode {
            CalculationMode::Direct => self.position,
            CalculationMode::NetherToEnd => END_SPAWN_POSTION,
        };
        let target_distance = start_pos.array_to(target_point.into());

        let mut result: Vec<FtlConfig> = Vec::new();

        let start_time_iter = match mode {
            CalculationMode::Direct => Time::range(Time(0), Time(1)),
            CalculationMode::NetherToEnd => Time::range(Time(0), max_time),
        };

        for start_time in start_time_iter {
            let first_end_time = match mode {
                CalculationMode::Direct => start_time + 1,
                CalculationMode::NetherToEnd => start_time + 2,
            }
            .max(min_time);

            for end_time in Time::range(first_end_time, max_time + 1) {
                let rotate = matches!(mode, CalculationMode::NetherToEnd);
                let nums = Array::to_nums(
                    target_distance,
                    motion_per_tnt,
                    self.motion,
                    start_time,
                    end_time,
                    rotate,
                );
                let to_end_time = match mode {
                    CalculationMode::NetherToEnd => Some(start_time + 1),
                    CalculationMode::Direct => None,
                };
                result.extend(nums.iter().map(|&num| FtlConfig {
                    tnt_num: num,
                    end_time,
                    to_end_time,
                }));
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::vector;

    fn pearl_in(dimension: Dimension) -> Pearl {
        Pearl {
            position: Array(vector![0.0, 0.0, 0.0]),
            motion: Array(vector![0.0, 0.0, 0.0]),
            yaw: Angle(0.0),
            dimension,
        }
    }

    #[test]
    fn calculation_mode_supports_expected_transitions() {
        assert!(matches!(
            pearl_in(Dimension::Nether).calculation_mode(Dimension::Nether),
            Ok(CalculationMode::Direct)
        ));
        assert!(matches!(
            pearl_in(Dimension::Nether).calculation_mode(Dimension::End),
            Ok(CalculationMode::NetherToEnd)
        ));
        assert!(matches!(
            pearl_in(Dimension::End).calculation_mode(Dimension::End),
            Ok(CalculationMode::Direct)
        ));
    }

    #[test]
    fn calculation_mode_rejects_unsupported_transitions() {
        assert!(matches!(
            pearl_in(Dimension::End).calculation_mode(Dimension::Nether),
            Err(PearlError::UnsupportedCalculationDimensionTransition {
                start_dimension: Dimension::End,
                target_dimension: Dimension::Nether,
            })
        ));
        assert!(matches!(
            pearl_in(Dimension::Overworld).calculation_mode(Dimension::Nether),
            Err(PearlError::UnsupportedCalculationDimensionTransition {
                start_dimension: Dimension::Overworld,
                target_dimension: Dimension::Nether,
            })
        ));
    }
}
