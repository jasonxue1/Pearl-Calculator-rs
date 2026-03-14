mod output;
use std::{error::Error, fs, io, path::PathBuf};

use crate::output::{print_calculation_report, print_simulation_report};
use clap::{ArgGroup, Args, Parser, Subcommand};
use nalgebra::vector;
use pearl_calculator::{
    calculation,
    config::{Config, Root},
    convert::{RB, rb_to_num},
    pearl::Dimension,
    simulation,
    util::MaxTnt,
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Simulation(SimulationArgs),
    Calculation(CalculationArgs),
}

#[derive(Args)]
#[command(group(
    ArgGroup::new("input_mode")
        .required(true)
        .multiple(false)
        .args(["num", "rb"])
))]
struct SimulationArgs {
    #[arg(long = "config")]
    config_file_path: PathBuf,
    #[arg(long)]
    time: u64,
    #[arg(long = "to-end-time", default_value_t = 0)]
    to_end_time: u64,
    #[arg(
        long,
        num_args = 2,
        value_names = ["SE", "NE"],
        allow_hyphen_values = true,
        group = "input_mode"
    )]
    num: Option<Vec<i64>>,
    #[arg(
        long,
        num_args = 3,
        value_names = ["DIRECTION", "RED", "BLUE"],
        allow_hyphen_values = true,
        group = "input_mode"
    )]
    rb: Option<Vec<i64>>,
}

#[derive(Args)]
struct CalculationArgs {
    #[arg(long = "config")]
    config_file_path: PathBuf,
    #[arg(
        long = "max-tnt-num",
        num_args = 0..=2,
        value_names = ["RED", "BLUE"],
        help = "Zero values uses config max_tnt, one value uses the same red/blue limit"
    )]
    max_tnt_num: Option<Vec<u64>>,
    #[arg(
        long = "target-point",
        num_args = 2,
        value_names = ["X", "Z"],
        required = true,
        allow_hyphen_values = true
    )]
    target_point: Vec<i64>,
    #[arg(long)]
    error: u64,
    #[arg(long = "max-time")]
    max_time: u64,
}

fn parse_num(values: Vec<i64>) -> [i64; 2] {
    [values[0], values[1]]
}

fn parse_rb(values: Vec<i64>) -> Result<RB, io::Error> {
    let direction = usize::try_from(values[0]).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "--rb <DIRECTION> must be a non-negative integer",
        )
    })?;
    Ok(RB {
        count: vector![values[1], values[2]],
        direction,
    })
}

fn parse_max_tnt_num(values: Option<Vec<u64>>) -> Option<MaxTnt> {
    match values.as_deref() {
        None | Some([]) => None,
        Some([value]) => Some(MaxTnt {
            red: *value,
            blue: *value,
        }),
        Some([red, blue]) => Some(MaxTnt {
            red: *red,
            blue: *blue,
        }),
        _ => unreachable!("clap enforces zero, one, or two --max-tnt-num values"),
    }
}

fn parse_target_point(values: Vec<i64>) -> [i64; 2] {
    [values[0], values[1]]
}

fn load_config(path: PathBuf) -> Result<Config, Box<dyn Error>> {
    let config_file = fs::read_to_string(path)?;
    let root: Root = serde_json::from_str(&config_file)?;
    Ok(Config::from(root))
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Simulation(args) => {
            let config = load_config(args.config_file_path)?;

            let num = match (args.num, args.rb) {
                (Some(num), None) => {
                    let [se, ne] = parse_num(num);
                    vector![se, ne]
                }
                (None, Some(rb)) => rb_to_num(parse_rb(rb)?, &config.directions),
                _ => unreachable!("clap enforces exactly one input mode"),
            };
            let simulation_report = simulation(&config, num, args.time, args.to_end_time);
            print_simulation_report(simulation_report);
        }
        Command::Calculation(args) => {
            let config = load_config(args.config_file_path)?;
            let max_tnt = parse_max_tnt_num(args.max_tnt_num);
            let [x, z] = parse_target_point(args.target_point);
            let calculation_report = calculation(
                &config,
                max_tnt,
                vector![x, z],
                args.error,
                args.max_time,
                Dimension::Nether,
            );
            print_calculation_report(calculation_report, config.directions);
        }
    }
    Ok(())
}
