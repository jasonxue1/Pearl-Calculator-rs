mod output;
use std::{fs, io, path::PathBuf};

use crate::output::print_simulation_report;
use clap::{Parser, Subcommand};
use nalgebra::vector;
use pearl_calculator::{
    config::{Config, Root},
    convert::{RB, rb_to_num},
    simulation,
};

#[derive(Parser)]
struct Cli {
    config_file_path: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Simulation {
        time: u32,
        to_end_time: u32,
        #[command(subcommand)]
        mode: Mode,
    },
}

#[derive(Subcommand)]
enum Mode {
    Rb {
        direction: usize,
        red: i32,
        blue: i32,
    },
    Num {
        se: i32,
        ne: i32,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let config_file = fs::read_to_string(cli.config_file_path)?;
    let root: Root = serde_json::from_str(&config_file)?;
    let config = Config::from(root);

    match cli.command {
        Command::Simulation {
            mode,
            time,
            to_end_time,
        } => {
            let num = match mode {
                Mode::Num { se, ne } => vector![se, ne],
                Mode::Rb {
                    direction,
                    red,
                    blue,
                } => {
                    let rb = RB {
                        count: vector![red, blue],
                        direction,
                    };
                    rb_to_num(rb, config.directions)
                }
            };
            let simulation_report = simulation(config, num, time, to_end_time);
            print_simulation_report(simulation_report);
        }
    }
    Ok(())
}
