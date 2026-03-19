mod output;
use std::{
    error::Error,
    fs,
    io::{self, Error as IoError, ErrorKind, IsTerminal},
    path::PathBuf,
};

use crate::output::{print_calculation_report, print_simulation_report};
use clap::{
    Args, Parser, Subcommand,
    builder::styling::{AnsiColor, Effects, Styles},
};
use pearl_calculator::{
    CodeItem, CodeRule, Config, Dimension, PearlError, RB, Root, TNTNumCode, TNTNumRB, Time,
    calculation, code_to_rb, rb_to_code, simulation,
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
    Check(CheckArgs),
    Convert(ConvertArgs),
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

#[derive(Args)]
struct CheckArgs {
    #[arg(
        short = 'c',
        long = "config",
        help = "\u{1b}[33m(Required)\u{1b}[0m Config file path"
    )]
    config_file_path: PathBuf,
}

#[derive(Args)]
struct ConvertArgs {
    #[command(subcommand)]
    command: ConvertCommand,
}

#[derive(Subcommand)]
enum ConvertCommand {
    #[command(name = "rb-to-code")]
    Rb2Code(Rb2CodeArgs),
    #[command(name = "code-to-rb")]
    Code2Rb(Code2RbArgs),
}

#[derive(Args)]
struct Rb2CodeArgs {
    #[arg(
        short = 'c',
        long = "config",
        help = "\u{1b}[33m(Required)\u{1b}[0m Config file path"
    )]
    config_file_path: PathBuf,
    direction: usize,
    red: u64,
    blue: u64,
}

#[derive(Args)]
struct Code2RbArgs {
    #[arg(
        short = 'c',
        long = "config",
        help = "\u{1b}[33m(Required)\u{1b}[0m Config file path"
    )]
    config_file_path: PathBuf,
    code: String,
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

fn parse_code_input(input: &str) -> Result<TNTNumCode, Box<dyn Error>> {
    let trimmed: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    if trimmed.is_empty() {
        return Err(IoError::new(ErrorKind::InvalidInput, "code cannot be empty").into());
    }

    let mut bits = Vec::with_capacity(trimmed.len());
    for (idx, ch) in trimmed.chars().enumerate() {
        match ch {
            '0' => bits.push(false),
            '1' => bits.push(true),
            _ => {
                return Err(IoError::new(
                    ErrorKind::InvalidInput,
                    format!("invalid code char at position {}: '{ch}'", idx + 1),
                )
                .into());
            }
        }
    }

    Ok(TNTNumCode(bits))
}

fn format_code_with_rule(rule: &CodeRule, code: TNTNumCode) -> Result<String, Box<dyn Error>> {
    let bits = code.0;
    let mut bit_idx = 0usize;
    let mut out = String::new();
    let use_color = io::stdout().is_terminal();
    let reset = "\x1b[0m";
    let red = "\x1b[1;31m";
    let blue = "\x1b[1;34m";
    let green = "\x1b[1;32m";

    for item in &rule.default {
        match item {
            CodeItem::Space => out.push(' '),
            CodeItem::Red { .. } | CodeItem::Blue { .. } | CodeItem::Direction { .. } => {
                let bit = bits.get(bit_idx).ok_or_else(|| {
                    IoError::new(
                        ErrorKind::InvalidData,
                        "rb-to-code produced fewer bits than code rule requires",
                    )
                })?;
                let ch = if *bit { '1' } else { '0' };
                if use_color {
                    let color = match item {
                        CodeItem::Red { .. } => red,
                        CodeItem::Blue { .. } => blue,
                        CodeItem::Direction { .. } => green,
                        CodeItem::Space => unreachable!(),
                    };
                    out.push_str(color);
                    out.push(ch);
                    out.push_str(reset);
                } else {
                    out.push(ch);
                }
                bit_idx += 1;
            }
        }
    }

    if bit_idx != bits.len() {
        return Err(IoError::new(
            ErrorKind::InvalidData,
            "rb-to-code produced more bits than code rule requires",
        )
        .into());
    }

    Ok(out)
}

fn format_code_to_rb_output(rb: RB) -> String {
    if !io::stdout().is_terminal() {
        return format!(
            "direction={} red={} blue={}",
            rb.direction, rb.num.red, rb.num.blue
        );
    }

    let reset = "\x1b[0m";
    let magenta = "\x1b[1;35m";
    let yellow = "\x1b[1;33m";

    format!(
        "{}direction={}{} {}red={}{} {}blue={}{}",
        magenta, rb.direction, reset, yellow, rb.num.red, reset, yellow, rb.num.blue, reset
    )
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
        Command::Check(args) => {
            let config = load_config(args.config_file_path)?;
            config.check()?;
            println!("Config check passed.");
        }
        Command::Convert(args) => match args.command {
            ConvertCommand::Rb2Code(rb2code_args) => {
                let config = load_config(rb2code_args.config_file_path)?;
                let code = rb_to_code(
                    &config.code,
                    RB {
                        num: TNTNumRB {
                            red: rb2code_args.red,
                            blue: rb2code_args.blue,
                        },
                        direction: rb2code_args.direction,
                    },
                )?;
                println!("{}", format_code_with_rule(&config.code, code)?);
            }
            ConvertCommand::Code2Rb(code2rb_args) => {
                let config = load_config(code2rb_args.config_file_path)?;
                let code = parse_code_input(&code2rb_args.code)?;
                let rb = code_to_rb(&config.code, code)?;
                println!("{}", format_code_to_rb_output(rb));
            }
        },
    }
    Ok(())
}
