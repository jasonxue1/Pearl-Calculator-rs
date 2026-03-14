use std::io::{self, IsTerminal};

use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentArrangement, Table,
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL,
};
use pearl_calculator::{
    pearl::{Dimension, Pearl, SimulationReport},
    util::{ConfigOutputNether, FtlConfigOutput},
};

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_GREEN: &str = "\x1b[1;32m";
const TIME_COLOR: Color = Color::Green;
const RB_COLOR: Color = Color::Yellow;
const DIRECTION_COLOR: Color = Color::Magenta;
const ERROR_COLOR: Color = Color::Red;
const POS_COLOR: Color = Color::Cyan;
const VEL_COLOR: Color = Color::Yellow;
const YAW_COLOR: Color = Color::Magenta;
const DIM_COLOR: Color = Color::Blue;

fn format_scalar(value: f64) -> String {
    format!("{value:>10.5}")
}

fn new_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Disabled);
    table
}

fn header_cell(label: &str, color: Color) -> Cell {
    Cell::new(label)
        .fg(color)
        .add_attribute(Attribute::Bold)
        .set_alignment(CellAlignment::Center)
}

fn number_cell(value: f64, color: Color) -> Cell {
    Cell::new(format_scalar(value))
        .set_alignment(CellAlignment::Right)
        .fg(color)
        .add_attribute(Attribute::Bold)
}

fn integer_cell<T: ToString>(value: T, color: Color) -> Cell {
    Cell::new(value.to_string())
        .set_alignment(CellAlignment::Right)
        .fg(color)
        .add_attribute(Attribute::Bold)
}

fn dimension_cell(dimension: Dimension) -> Cell {
    Cell::new(dimension.to_string())
        .fg(DIM_COLOR)
        .add_attribute(Attribute::Bold)
        .set_alignment(CellAlignment::Center)
}

fn tick_cell(tick: usize) -> Cell {
    Cell::new(tick.to_string())
        .set_alignment(CellAlignment::Right)
        .fg(Color::DarkGrey)
}

fn history_row(tick: usize, pearl: Pearl) -> Vec<Cell> {
    vec![
        tick_cell(tick),
        number_cell(pearl.position.x, POS_COLOR),
        number_cell(pearl.position.y, POS_COLOR),
        number_cell(pearl.position.z, POS_COLOR),
        number_cell(pearl.motion.x, VEL_COLOR),
        number_cell(pearl.motion.y, VEL_COLOR),
        number_cell(pearl.motion.z, VEL_COLOR),
        number_cell(pearl.yaw as f64, YAW_COLOR),
        dimension_cell(pearl.dimension),
    ]
}

fn format_final_position(report: &SimulationReport) -> String {
    format!(
        "Final position: ({:.5}, {:.5}, {:.5})",
        report.final_pos.x, report.final_pos.y, report.final_pos.z
    )
}

fn green_text(text: &str) -> String {
    if io::stdout().is_terminal() {
        format!("{ANSI_GREEN}{text}{ANSI_RESET}")
    } else {
        text.to_string()
    }
}

pub fn print_simulation_report(simulation_report: SimulationReport) {
    let final_line = format_final_position(&simulation_report);
    let mut history = new_table();
    history.set_header(vec![
        header_cell("GT", Color::White),
        header_cell("Pos X", POS_COLOR),
        header_cell("Pos Y", POS_COLOR),
        header_cell("Pos Z", POS_COLOR),
        header_cell("Vel X", VEL_COLOR),
        header_cell("Vel Y", VEL_COLOR),
        header_cell("Vel Z", VEL_COLOR),
        header_cell("Yaw", YAW_COLOR),
        header_cell("Dim", DIM_COLOR),
    ]);

    for (tick, pearl) in simulation_report.history.into_iter().enumerate() {
        history.add_row(history_row(tick, pearl));
    }
    println!("{history}");
    println!("{}", green_text(&final_line));
}

fn calculation_row(result: ConfigOutputNether) -> Vec<Cell> {
    vec![
        integer_cell(result.time, TIME_COLOR),
        integer_cell(result.rb.direction, DIRECTION_COLOR),
        integer_cell(result.rb.count.x, RB_COLOR),
        integer_cell(result.rb.count.y, RB_COLOR),
        number_cell(result.error, ERROR_COLOR),
        number_cell(result.final_pos.x, POS_COLOR),
        number_cell(result.final_pos.y, POS_COLOR),
        number_cell(result.final_pos.z, POS_COLOR),
    ]
}

pub fn print_calculation_report(results: Vec<FtlConfigOutput>, max_output_count: usize) {
    if results.is_empty() {
        println!("{}", green_text("No calculation results."));
        return;
    }

    let result_count = results.len();

    let mut table = new_table();
    table.set_header(vec![
        header_cell("Time", TIME_COLOR),
        header_cell("Dir", DIRECTION_COLOR),
        header_cell("Red", RB_COLOR),
        header_cell("Blue", RB_COLOR),
        header_cell("Error", ERROR_COLOR),
        header_cell("Pos X", POS_COLOR),
        header_cell("Pos Y", POS_COLOR),
        header_cell("Pos Z", POS_COLOR),
    ]);

    for result in results.into_iter().take(max_output_count) {
        match result {
            FtlConfigOutput::Nether(config) => table.add_row(calculation_row(config)),
        };
    }

    println!("{table}");
    println!(
        "{}",
        green_text(&format!(
            "Calculation finished. Showing up to {max_output_count} of {result_count} result(s)."
        ))
    );
}
