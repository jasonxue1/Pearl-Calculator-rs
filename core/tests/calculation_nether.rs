use std::{fs, path::PathBuf};

use nalgebra::vector;
use pearl_calculator::{
    calculation,
    config::{Config, Root},
    pearl::Dimension,
    util::MaxTnt,
};

#[test]
fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("../test/config.json");
    let content = fs::read_to_string(file).unwrap();
    let value: Root = serde_json::from_str(&content).unwrap();
    let config = Config::from(value);

    let max_tnt = MaxTnt {
        red: 1000,
        blue: 1000,
    };
    let target_point = vector![3000, 2000];
    let error = 10;
    let max_time = 5;
    let dimension = Dimension::Nether;
    let res = calculation(
        &config,
        Some(max_tnt),
        target_point,
        error,
        max_time,
        dimension,
    );
    dbg!(&res);
    // for pearl_calculator::util::FtlConfig::Nether(config_nether) in res.iter() {
    //     let final_pos =
    //         pearl_calculator::simulation(&config, config_nether.num, config_nether.time, 0)
    //             .final_pos;
    //     assert!(pearl_calculator::util::check_close(
    //         target_point,
    //         final_pos,
    //         error + 2
    //     ));
    // }
}
