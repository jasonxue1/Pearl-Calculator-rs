use std::io;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use miette::Result;
use pearl_calculator::{RB, TNTNumRB, Time, calculation, code_to_rb, rb_to_code, simulation};

mod args;
mod helpers;
mod output;

use args::{Cli, Command, ConvertCommand};
use helpers::{
    format_code_to_rb_output, format_code_with_rule, load_config, parse_code_input,
    parse_max_tnt_num,
};
use output::{
    print_calculation_report, print_calculation_report_csv, print_simulation_report,
    print_simulation_report_csv,
};

pub(crate) fn run() -> Result<()> {
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
            if args.csv {
                print_simulation_report_csv(simulation_report);
            } else {
                print_simulation_report(simulation_report);
            }
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
            if args.csv {
                print_calculation_report_csv(calculation_report);
            } else {
                print_calculation_report(calculation_report);
            }
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
        Command::Complete(args) => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            generate(args.shell, &mut cmd, bin_name, &mut io::stdout());
        }
    }

    Ok(())
}
