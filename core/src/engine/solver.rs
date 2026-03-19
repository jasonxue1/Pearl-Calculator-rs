use nalgebra::{Matrix, Matrix2, Vector2, vector};
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct FtlConfig {
    pub tnt_num: TNTNum,
    pub end_time: Time,
    pub to_end_time: Option<Time>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub(crate) struct TNTNum(pub Vector2<i64>);

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

impl RB {
    fn from_num(num: TNTNum, directions: Directions) -> Result<Self, PearlError> {
        let direction_type = if num.0.x > 0 {
            if num.0.y > 0 { 0 } else { 1 }
        } else if num.0.y > 0 {
            2
        } else {
            3
        };
        let direction_num = directions.resolve()?[direction_type];
        let direction = directions.0[direction_num].match_direction()?;
        Ok(RB {
            num: TNTNumRB::from(direction.transpose() * num.0),
            direction: direction_num,
        })
    }

    pub(crate) fn to_num(self, directions: Directions) -> Result<TNTNum, PearlError> {
        if self.direction > 3 {
            return Err(PearlError::DirectionOutOfRange {
                value: self.direction as u64,
            });
        }
        let base = directions.0[self.direction].match_direction()?;
        Ok(TNTNum(base * Vector2::from(self.num)))
    }

    pub(crate) fn is_available(self, max: TNTNumRB) -> bool {
        self.num.is_available(max)
    }
}

fn match_direction(input: [i8; 2]) -> Result<Vector2<i64>, PearlError> {
    match input {
        [1, 1] => Ok(vector![1, 0]),
        [-1, -1] => Ok(vector![-1, 0]),
        [1, -1] => Ok(vector![0, 1]),
        [-1, 1] => Ok(vector![0, -1]),
        _ => Err(PearlError::InvalidDirectionVector(input)),
    }
}

impl Direction {
    fn match_direction(self) -> Result<Matrix2<i64>, PearlError> {
        let red = match_direction(self.red)?;
        let blue = match_direction(self.blue)?;
        Ok(Matrix::from_columns(&[red, blue]))
    }
}

impl Directions {
    pub(crate) fn resolve(self) -> Result<[usize; 4], PearlError> {
        let mut indices = [0; 4];
        let mut seen = [false; 4];

        for (i, direction) in self.0.iter().enumerate() {
            let matrix = direction.match_direction()?;
            let sum = matrix.column(0) + matrix.column(1);
            let idx = match (sum.x, sum.y) {
                (1, 1) => 0,
                (1, -1) => 1,
                (-1, 1) => 2,
                (-1, -1) => 3,
                _ => return Err(PearlError::InvalidDirectionCombination { x: sum.x, y: sum.y }),
            };

            if seen[idx] {
                return Err(PearlError::DuplicateDirectionQuadrant { quadrant: idx });
            }
            seen[idx] = true;
            indices[idx] = i;
        }

        Ok(indices)
    }
}

impl FtlConfig {
    pub(crate) fn generate(
        self,
        config: &Config,
        target_point: Vector2<i64>,
    ) -> Result<CalculationReport, PearlError> {
        let rb = RB::from_num(self.tnt_num, config.directions)?;
        let simulation_report = simulation(config, rb, Some(self.end_time), self.to_end_time)?;

        let final_pos = simulation_report.final_pos;

        Ok(CalculationReport {
            rb,
            end_time: self.end_time,
            error: final_pos.distance_xz(target_point.into()),
            final_pos,
            to_end_time: self.to_end_time,
            end_portal_pos: simulation_report.end_portal_pos,
        })
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

impl TNTNumRB {
    fn is_available(self, max: Self) -> bool {
        self.red <= max.red && self.blue <= max.blue
    }
}
