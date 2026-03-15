use crate::util::FtlConfig;
use crate::*;
use minecraft_mth as mth;
use nalgebra::{Vector2, matrix, vector};

const END_SPAWN_POSTION: Array = Array(vector![100.5, 50.0, 0.5]);
const END_SPAWN_YAW: Yaw = Yaw(90.0);

enum Teleport {
    None,
    // NetherPortal,
    EndPortal,
}

impl Pearl {
    fn move_motion(&mut self) {
        self.position += self.motion
    }

    fn lerp_rotation(&mut self) {
        self.yaw.lerp_rotation(self.motion);
    }

    fn tick(&mut self, teleport: Teleport) {
        self.motion.tick();
        self.lerp_rotation();

        match teleport {
            Teleport::None => self.move_motion(),
            Teleport::EndPortal => match self.dimension {
                Dimension::End => todo!(),
                _ => {
                    self.rotate_motion(END_SPAWN_YAW);
                    self.dimension = Dimension::End;
                    self.position = END_SPAWN_POSTION;
                }
            },
        }
    }

    fn rotate_motion(&mut self, new_yaw: Yaw) {
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
    ) -> SimulationReport {
        let Time(to_end_time) = to_end_time.unwrap_or(Time(0));
        if to_end_time > time {
            panic!()
        };
        if time == 0 {
            panic!()
        };
        self.motion += tnt_motion;
        let mut history: Vec<Pearl> = Vec::new();
        history.push(*self);
        let mut to_end_pos = None;
        for _ in 0..time {
            if time == to_end_time {
                let mut self_clone = *self;
                self_clone.tick(Teleport::None);
                to_end_pos = Some(self_clone.position);
                self.tick(Teleport::EndPortal)
            } else {
                self.tick(Teleport::None)
            };
            history.push(*self);
        }
        SimulationReport {
            history,
            final_pos: self.position,
            end_portal_pos: to_end_pos,
        }
    }

    pub(crate) fn calculation(
        self,
        target_point: Vector2<i64>,
        motion_per_tnt: MotionPerTnt,
        max_time: Time,
        dimension: Dimension,
    ) -> Vec<FtlConfig> {
        let start_pos = match dimension {
            Dimension::Nether => self.position,
            Dimension::End => END_SPAWN_POSTION,
            Dimension::Overworld => todo!(),
        };
        let target_distance = start_pos.array_to(target_point.into());

        let mut result: Vec<FtlConfig> = Vec::new();

        let start_time_iter = match dimension {
            Dimension::Nether => Time::range(Time(0), Time(1)),
            Dimension::End => Time::range(Time(1), max_time),
            Dimension::Overworld => todo!(),
        };

        for start_time in start_time_iter {
            for end_time in Time::range(start_time + 1, max_time + 1) {
                let nums = target_distance.to_nums(motion_per_tnt, start_time, end_time);
                let to_end_time = match dimension {
                    Dimension::End => Some(start_time),
                    _ => None,
                };
                result.extend(nums.iter().map(|&num| FtlConfig {
                    tnt_num: num,
                    end_time,
                    to_end_time,
                }));
            }
        }

        result
    }
}
