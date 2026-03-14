use std::array::from_fn;

use nalgebra::{Matrix2, Vector2};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{pearl::Pearl, util::MaxTnt};

// direction
// red1 red2
// blue1 blue2
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub pearl: Pearl,
    #[serde(
        deserialize_with = "deserialize_directions",
        serialize_with = "serialize_directions"
    )]
    pub directions: [Matrix2<i64>; 4],
    code: Code,
    pub motion_per_tnt: MotionPerTnt,
    pub max_tnt: MaxTnt,
}

pub fn resolve_directions(directions: [Matrix2<i64>; 4]) -> [usize; 4] {
    let mut indices = [0; 4];
    let mut seen = [false; 4];

    for (i, m) in directions.iter().enumerate() {
        let sum = m.column(0) + m.column(1);
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

fn deserialize_directions<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<[Matrix2<i64>; 4], D::Error> {
    let defs = <[DirectionDef; 4]>::deserialize(deserializer)?;
    let result = defs.map(|d| Matrix2::from_columns(&[d.red, d.blue]));
    resolve_directions(result);
    Ok(result)
}

fn serialize_directions<S: Serializer>(
    directions: &[Matrix2<i64>; 4],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let defs: [DirectionDef; 4] = from_fn(|i| {
        let m = &directions[i];
        DirectionDef {
            red: m.column(0).into_owned(),
            blue: m.column(1).into_owned(),
        }
    });
    defs.serialize(serializer)
}

#[derive(Serialize, Deserialize, Debug)]
struct DirectionDef {
    red: Vector2<i64>,
    blue: Vector2<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
enum CodeItem {
    Red { count: u64 },
    Blue { count: u64 },
    Direction { count: u64 },
    Space,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct MotionPerTnt {
    pub x_z: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct CodeExtra {
    caps: Vec<CodeCaps>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CodeCaps {
    bits: Vec<usize>,
    cap: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Code {
    default: Vec<CodeItem>,
    extra: CodeExtra,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    version: u64,
    config: Config,
}

impl From<Root> for Config {
    fn from(value: Root) -> Self {
        match value.version {
            1 => value.config,
            _ => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::config::Root;

    #[test]
    fn parse_json_test() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let file = root.join("../test/config.json");
        let content = fs::read_to_string(file).unwrap();
        let value: Root = serde_json::from_str(&content).unwrap();
        dbg!(value);
    }
}
