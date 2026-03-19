mod output;
use std::{error::Error, fs, path::PathBuf};

use crate::output::{print_calculation_report, print_simulation_report};
use clap::{
    Args, Parser, Subcommand,
    builder::styling::{AnsiColor, Effects, Styles},
};
use pearl_calculator::{
    Config, Dimension, PearlError, RB, Root, TNTNumRB, Time, calculation, simulation,
};

#[derive(Parser)]
#[command(styles = cli_styles())]
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
struct SimulationArgs {
    #[arg(
        short = 'c',
        long = "config",
        help = "\u{1b}[33m(Required)\u{1b}[0m Config file path"
    )]
    config_file_path: PathBuf,
    #[arg(short = 't', long = "time", help = "Simulation time")]
    time: Option<u64>,
    #[arg(short = 'e', long = "to-end-time")]
    to_end_time: Option<u64>,
    direction: usize,
    red: u64,
    blue: u64,
}

#[derive(Args)]
struct CalculationArgs {
    #[arg(
        short = 'c',
        long = "config",
        help = "\u{1b}[33m(Required)\u{1b}[0m Config file path"
    )]
    config_file_path: PathBuf,
    #[arg(
        long = "max-tnt",
        num_args = 0..=2,
        value_names = ["RED", "BLUE"],
        help = "Zero values uses config max_tnt, one value uses the same red/blue limit"
    )]
    max_tnt: Option<Vec<u64>>,
    #[arg(allow_hyphen_values = true)]
    x: i64,
    #[arg(allow_hyphen_values = true)]
    z: i64,
    #[arg(long = "max-error")]
    max_error: Option<f64>,
    #[arg(short = 't', long = "max-time")]
    max_time: Option<u64>,
    #[arg(short = 'd', long = "dimension", value_parser = parse_dimension)]
    dimension: Option<Dimension>,
    #[arg(long = "first")]
    first: Option<usize>,
}

fn cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Green.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Red.on_default() | Effects::BOLD)
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
}

fn parse_max_tnt_num(values: Option<Vec<u64>>) -> Result<Option<TNTNumRB>, PearlError> {
    match values.as_deref() {
        None | Some([]) => Ok(None),
        Some([value]) => Ok(Some(TNTNumRB {
            red: *value,
            blue: *value,
        })),
        Some([red, blue]) => Ok(Some(TNTNumRB {
            red: *red,
            blue: *blue,
        })),
        Some(values) => Err(PearlError::InvalidMaxTntArgCount(values.len())),
    }
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
    Ok(Config::try_from(root)?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Simulation(args) => {
            let config = load_config(args.config_file_path)?;

            let rb = RB {
                num: TNTNumRB {
                    red: args.red,
                    blue: args.blue,
                },
                direction: args.direction,
            };
            let simulation_report =
                simulation(&config, rb, args.time.map(Time), args.to_end_time.map(Time))?;
            print_simulation_report(simulation_report);
        }
        Command::Calculation(args) => {
            let config = load_config(args.config_file_path)?;
            let max_tnt = parse_max_tnt_num(args.max_tnt)?;
            let calculation_report = calculation(
                &config,
                max_tnt,
                nalgebra::vector![args.x, args.z],
                args.max_error,
                args.max_time.map(Time),
                args.dimension,
                args.first,
            )?;
            print_calculation_report(calculation_report);
        }
    }
    Ok(())
}
