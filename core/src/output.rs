use std::io::{self, IsTerminal};

use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentArrangement, Table,
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL,
};
use pearl_calculator::pearl::{Dimension, Pearl, SimulationReport};

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_GREEN: &str = "\x1b[1;32m";
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
