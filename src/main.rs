use std::{
    io::{self, IsTerminal},
    process::ExitCode,
};

mod cli;

fn print_error(message: &str) {
    if io::stderr().is_terminal() {
        eprintln!("\x1b[1;31merror:\x1b[0m {message}");
    } else {
        eprintln!("error: {message}");
    }
}

fn main() -> ExitCode {
    match cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            print_error(&err.to_string());
            ExitCode::FAILURE
        }
    }
}
