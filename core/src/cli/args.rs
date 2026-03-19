use std::path::PathBuf;

use clap::{
    Args, Parser, Subcommand,
    builder::styling::{AnsiColor, Effects, Styles},
};
use pearl_calculator::Dimension;

#[derive(Parser)]
#[command(styles = cli_styles())]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Simulation(SimulationArgs),
    Calculation(CalculationArgs),
    Check(CheckArgs),
    Convert(ConvertArgs),
}

#[derive(Args)]
pub(crate) struct SimulationArgs {
    #[arg(
        short = 'c',
        long = "config",
        help = "\u{1b}[33m(Required)\u{1b}[0m Config file path"
    )]
    pub config_file_path: PathBuf,
    #[arg(short = 't', long = "time", help = "Simulation time")]
    pub time: Option<u64>,
    #[arg(short = 'e', long = "to-end-time")]
    pub to_end_time: Option<u64>,
    pub direction: usize,
    pub red: u64,
    pub blue: u64,
}

#[derive(Args)]
pub(crate) struct CalculationArgs {
    #[arg(
        short = 'c',
        long = "config",
        help = "\u{1b}[33m(Required)\u{1b}[0m Config file path"
    )]
    pub config_file_path: PathBuf,
    #[arg(
        long = "max-tnt",
        num_args = 0..=2,
        value_names = ["RED", "BLUE"],
        help = "Zero values uses config max_tnt, one value uses the same red/blue limit"
    )]
    pub max_tnt: Option<Vec<u64>>,
    #[arg(allow_hyphen_values = true)]
    pub x: i64,
    #[arg(allow_hyphen_values = true)]
    pub z: i64,
    #[arg(long = "max-error")]
    pub max_error: Option<f64>,
    #[arg(short = 't', long = "max-time")]
    pub max_time: Option<u64>,
    #[arg(short = 'd', long = "dimension", value_parser = parse_dimension)]
    pub dimension: Option<Dimension>,
    #[arg(long = "first")]
    pub first: Option<usize>,
}

#[derive(Args)]
pub(crate) struct CheckArgs {
    #[arg(
        short = 'c',
        long = "config",
        help = "\u{1b}[33m(Required)\u{1b}[0m Config file path"
    )]
    pub config_file_path: PathBuf,
}

#[derive(Args)]
pub(crate) struct ConvertArgs {
    #[command(subcommand)]
    pub command: ConvertCommand,
}

#[derive(Subcommand)]
pub(crate) enum ConvertCommand {
    #[command(name = "rb-to-code")]
    Rb2Code(Rb2CodeArgs),
    #[command(name = "code-to-rb")]
    Code2Rb(Code2RbArgs),
}

#[derive(Args)]
pub(crate) struct Rb2CodeArgs {
    #[arg(
        short = 'c',
        long = "config",
        help = "\u{1b}[33m(Required)\u{1b}[0m Config file path"
    )]
    pub config_file_path: PathBuf,
    pub direction: usize,
    pub red: u64,
    pub blue: u64,
}

#[derive(Args)]
pub(crate) struct Code2RbArgs {
    #[arg(
        short = 'c',
        long = "config",
        help = "\u{1b}[33m(Required)\u{1b}[0m Config file path"
    )]
    pub config_file_path: PathBuf,
    pub code: String,
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

fn parse_dimension(value: &str) -> Result<Dimension, String> {
    match value {
        "overworld" => Ok(Dimension::Overworld),
        "nether" => Ok(Dimension::Nether),
        "end" => Ok(Dimension::End),
        _ => Err("dimension must be one of: overworld, nether, end".to_string()),
    }
}
