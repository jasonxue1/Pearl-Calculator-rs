mod output;
use std::{error::Error, fs, io, path::PathBuf};

use crate::output::print_simulation_report;
use clap::{ArgGroup, Args, Parser, Subcommand};
use nalgebra::vector;
use pearl_calculator::{
    config::{Config, Root},
    convert::{RB, rb_to_num},
    simulation,
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Simulation(SimulationArgs),
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

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Simulation(args) => {
            let config_file = fs::read_to_string(args.config_file_path)?;
            let root: Root = serde_json::from_str(&config_file)?;
            let config = Config::from(root);

            let num = match (args.num, args.rb) {
                (Some(num), None) => {
                    let [se, ne] = parse_num(num);
                    vector![se, ne]
                }
                (None, Some(rb)) => rb_to_num(parse_rb(rb)?, config.directions),
                _ => unreachable!("clap enforces exactly one input mode"),
            };
            let simulation_report = simulation(&config, num, args.time, args.to_end_time);
            print_simulation_report(simulation_report);
        }
    }
    Ok(())
}
