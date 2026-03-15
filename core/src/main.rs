mod output;
use std::{error::Error, fs, io, path::PathBuf};

use crate::output::{print_calculation_report, print_simulation_report};
use clap::{ArgGroup, Args, Parser, Subcommand};
use pearl_calculator::{
    calculation,
    config::{Config, Root},
    pearl::Dimension,
    simulation,
    util::{RB, TNTNum, TNTNumRB, Time},
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
        long = "max-tnt",
        num_args = 0..=2,
        value_names = ["RED", "BLUE"],
        help = "Zero values uses config max_tnt, one value uses the same red/blue limit"
    )]
    max_tnt: Option<Vec<u64>>,
    #[arg(
        long = "target-point",
        num_args = 2,
        value_names = ["X", "Z"],
        required = true,
        allow_hyphen_values = true
    )]
    target_point: Vec<i64>,
    #[arg(long = "max-error")]
    max_error: Option<f64>,
    #[arg(long = "max-time")]
    max_time: Option<u64>,
    #[arg(long = "dimension", value_parser = parse_dimension)]
    dimension: Option<Dimension>,
    #[arg(long = "first")]
    first: Option<usize>,
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
        num: TNTNumRB {
            red: u64::try_from(values[1]).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "--rb <RED> must be non-negative",
                )
            })?,
            blue: u64::try_from(values[2]).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "--rb <BLUE> must be non-negative",
                )
            })?,
        },
        direction,
    })
}

fn parse_max_tnt_num(values: Option<Vec<u64>>) -> Option<TNTNumRB> {
    match values.as_deref() {
        None | Some([]) => None,
        Some([value]) => Some(TNTNumRB {
            red: *value,
            blue: *value,
        }),
        Some([red, blue]) => Some(TNTNumRB {
            red: *red,
            blue: *blue,
        }),
        _ => unreachable!("clap enforces zero, one, or two --max-tnt-num values"),
    }
}

fn parse_target_point(values: Vec<i64>) -> [i64; 2] {
    [values[0], values[1]]
}

fn parse_dimension(value: &str) -> Result<Dimension, String> {
    match value {
        "overworld" => Ok(Dimension::Overworld),
        "nether" => Ok(Dimension::Nether),
        "end" => Ok(Dimension::End),
        _ => Err("dimension must be one of: overworld, nether, end".to_string()),
    }
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

            let num: TNTNum = match (args.num, args.rb) {
                (Some(num), None) => {
                    let [se, ne] = parse_num(num);
                    TNTNum(nalgebra::vector![se, ne])
                }
                (None, Some(rb)) => TNTNum::from_rb(parse_rb(rb)?, config.directions),
                _ => unreachable!("clap enforces exactly one input mode"),
            };
            let simulation_report = simulation(
                &config,
                num,
                Time(args.time),
                Some(args.to_end_time).filter(|&x| x != 0).map(Time),
            );
            print_simulation_report(simulation_report);
        }
        Command::Calculation(args) => {
            let config = load_config(args.config_file_path)?;
            let max_tnt = parse_max_tnt_num(args.max_tnt);
            let [x, z] = parse_target_point(args.target_point);
            let calculation_report = calculation(
                &config,
                max_tnt,
                nalgebra::vector![x, z],
                args.max_error,
                args.max_time.map(Time),
                args.dimension,
                args.first,
            );
            print_calculation_report(calculation_report);
        }
    }
    Ok(())
}
