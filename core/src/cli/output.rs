use std::io::{self, IsTerminal};

use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ContentArrangement, Table,
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL,
};
use pearl_calculator::{CalculationReport, Dimension, Pearl, SimulationReport};

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

#[derive(Clone, Copy)]
struct SimulationRow {
    tick: usize,
    pos_x: f64,
    pos_y: f64,
    pos_z: f64,
    vel_x: f64,
    vel_y: f64,
    vel_z: f64,
    yaw: f64,
    dim: Dimension,
}

#[derive(Clone, Copy)]
struct CalculationLayout {
    show_to_end_time: bool,
    show_end_portal_pos: bool,
}

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

fn empty_cell() -> Cell {
    Cell::new("")
}

fn green_text(text: &str) -> String {
    if io::stdout().is_terminal() {
        format!("{ANSI_GREEN}{text}{ANSI_RESET}")
    } else {
        text.to_string()
    }
}

fn format_final_position(report: &SimulationReport) -> String {
    format!(
        "Final position: ({:.5}, {:.5}, {:.5})",
        report.final_pos.0.x, report.final_pos.0.y, report.final_pos.0.z
    )
}

fn build_simulation_rows(history: Vec<Pearl>) -> Vec<SimulationRow> {
    history
        .into_iter()
        .enumerate()
        .map(|(tick, pearl)| SimulationRow {
            tick,
            pos_x: pearl.position.0.x,
            pos_y: pearl.position.0.y,
            pos_z: pearl.position.0.z,
            vel_x: pearl.motion.0.x,
            vel_y: pearl.motion.0.y,
            vel_z: pearl.motion.0.z,
            yaw: pearl.yaw.0 as f64,
            dim: pearl.dimension,
        })
        .collect()
}

fn simulation_row_cells(row: SimulationRow) -> Vec<Cell> {
    vec![
        tick_cell(row.tick),
        number_cell(row.pos_x, POS_COLOR),
        number_cell(row.pos_y, POS_COLOR),
        number_cell(row.pos_z, POS_COLOR),
        number_cell(row.vel_x, VEL_COLOR),
        number_cell(row.vel_y, VEL_COLOR),
        number_cell(row.vel_z, VEL_COLOR),
        number_cell(row.yaw, YAW_COLOR),
        dimension_cell(row.dim),
    ]
}

fn simulation_row_csv(row: SimulationRow) -> String {
    format!(
        "{},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{:.5},{}",
        row.tick,
        row.pos_x,
        row.pos_y,
        row.pos_z,
        row.vel_x,
        row.vel_y,
        row.vel_z,
        row.yaw,
        row.dim
    )
}

fn calculation_layout(results: &[CalculationReport]) -> CalculationLayout {
    CalculationLayout {
        show_to_end_time: results.iter().any(|result| result.to_end_time.is_some()),
        show_end_portal_pos: results.iter().any(|result| result.end_portal_pos.is_some()),
    }
}

fn calculation_row_cells(result: &CalculationReport, layout: CalculationLayout) -> Vec<Cell> {
    let mut row = vec![
        integer_cell(result.end_time.0, TIME_COLOR),
        integer_cell(result.rb.direction, DIRECTION_COLOR),
        integer_cell(result.rb.num.red, RB_COLOR),
        integer_cell(result.rb.num.blue, RB_COLOR),
        number_cell(result.error, ERROR_COLOR),
        number_cell(result.final_pos.0.x, POS_COLOR),
        number_cell(result.final_pos.0.y, POS_COLOR),
        number_cell(result.final_pos.0.z, POS_COLOR),
    ];

    if layout.show_to_end_time {
        row.push(
            result
                .to_end_time
                .map(|value| integer_cell(value.0, TIME_COLOR))
                .unwrap_or_else(empty_cell),
        );
    }

    if layout.show_end_portal_pos {
        match result.end_portal_pos {
            Some(pos) => {
                row.push(number_cell(pos.0.x, POS_COLOR));
                row.push(number_cell(pos.0.y, POS_COLOR));
                row.push(number_cell(pos.0.z, POS_COLOR));
            }
            None => {
                row.push(empty_cell());
                row.push(empty_cell());
                row.push(empty_cell());
            }
        }
    }

    row
}

fn calculation_row_csv(result: &CalculationReport, layout: CalculationLayout) -> Vec<String> {
    let mut row = vec![
        result.end_time.0.to_string(),
        result.rb.direction.to_string(),
        result.rb.num.red.to_string(),
        result.rb.num.blue.to_string(),
        format!("{:.5}", result.error),
        format!("{:.5}", result.final_pos.0.x),
        format!("{:.5}", result.final_pos.0.y),
        format!("{:.5}", result.final_pos.0.z),
    ];

    if layout.show_to_end_time {
        row.push(
            result
                .to_end_time
                .map(|value| value.0.to_string())
                .unwrap_or_default(),
        );
    }

    if layout.show_end_portal_pos {
        if let Some(pos) = result.end_portal_pos {
            row.push(format!("{:.5}", pos.0.x));
            row.push(format!("{:.5}", pos.0.y));
            row.push(format!("{:.5}", pos.0.z));
        } else {
            row.push(String::new());
            row.push(String::new());
            row.push(String::new());
        }
    }

    row
}

pub(crate) fn print_simulation_report(simulation_report: SimulationReport) {
    let final_line = format_final_position(&simulation_report);
    let rows = build_simulation_rows(simulation_report.history);

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

    for row in rows {
        history.add_row(simulation_row_cells(row));
    }

    println!("{history}");
    if let Some(end_portal_pos) = simulation_report.end_portal_pos {
        println!(
            "{}",
            green_text(&format!(
                "End portal position: ({:.5}, {:.5}, {:.5})",
                end_portal_pos.0.x, end_portal_pos.0.y, end_portal_pos.0.z
            ))
        );
    }
    println!("{}", green_text(&final_line));
}

pub(crate) fn print_simulation_report_csv(simulation_report: SimulationReport) {
    let rows = build_simulation_rows(simulation_report.history);
    println!("gt,pos_x,pos_y,pos_z,vel_x,vel_y,vel_z,yaw,dim");
    for row in rows {
        println!("{}", simulation_row_csv(row));
    }
}

pub(crate) fn print_calculation_report(results: Vec<CalculationReport>) {
    if results.is_empty() {
        println!("{}", green_text("No calculation results."));
        return;
    }

    let result_count = results.len();
    let layout = calculation_layout(&results);

    let mut table = new_table();
    let mut header = vec![
        header_cell("Time", TIME_COLOR),
        header_cell("Dir", DIRECTION_COLOR),
        header_cell("Red", RB_COLOR),
        header_cell("Blue", RB_COLOR),
        header_cell("Error", ERROR_COLOR),
        header_cell("Pos X", POS_COLOR),
        header_cell("Pos Y", POS_COLOR),
        header_cell("Pos Z", POS_COLOR),
    ];

    if layout.show_to_end_time {
        header.push(header_cell("To End", TIME_COLOR));
    }

    if layout.show_end_portal_pos {
        header.push(header_cell("Portal X", POS_COLOR));
        header.push(header_cell("Portal Y", POS_COLOR));
        header.push(header_cell("Portal Z", POS_COLOR));
    }

    table.set_header(header);

    for result in &results {
        table.add_row(calculation_row_cells(result, layout));
    }

    println!("{table}");
    println!(
        "{}",
        green_text(&format!("Calculation finished. {result_count} result(s)."))
    );
}

pub(crate) fn print_calculation_report_csv(results: Vec<CalculationReport>) {
    let layout = calculation_layout(&results);

    let mut header = vec![
        "time".to_string(),
        "dir".to_string(),
        "red".to_string(),
        "blue".to_string(),
        "error".to_string(),
        "pos_x".to_string(),
        "pos_y".to_string(),
        "pos_z".to_string(),
    ];

    if layout.show_to_end_time {
        header.push("to_end_time".to_string());
    }

    if layout.show_end_portal_pos {
        header.push("portal_x".to_string());
        header.push("portal_y".to_string());
        header.push("portal_z".to_string());
    }

    println!("{}", header.join(","));

    for result in &results {
        println!("{}", calculation_row_csv(result, layout).join(","));
    }
}
