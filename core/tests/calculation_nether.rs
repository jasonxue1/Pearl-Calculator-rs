use std::{fs, path::PathBuf};

use nalgebra::vector;
use pearl_calculator::{
    calculation,
    config::{Config, Root},
    pearl::Dimension,
};

#[test]
fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file = root.join("../test/config.json");
    let content = fs::read_to_string(file).unwrap();
    let value: Root = serde_json::from_str(&content).unwrap();
    let config = Config::from(value);

    let max_tnt_num = vector![1088, 1088];
    let target_point = vector![3000, 2000];
    let error = 10;
    let max_time = 20;
    let dimension = Dimension::Nether;
    let res = calculation(
        &config,
        max_tnt_num,
        target_point,
        error,
        max_time,
        dimension,
    );
    dbg!(&res);
    // for FtlConfig::Nether(config_nether) in res.iter() {
    //     let final_pos = simulation(&config, config_nether.num, config_nether.time, 0).final_pos;
    //     assert!(check_close(target_point, final_pos, error + 2));
    // }
    // panic!()
}
