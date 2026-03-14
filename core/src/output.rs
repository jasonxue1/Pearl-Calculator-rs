use comfy_table::Table;
use nalgebra::Vector3;
use pearl_calculator::pearl::SimulationReport;

fn format_vec3(v: Vector3<f64>) -> String {
    format!("[{:.5}, {:.5}, {:.5}]", v.x, v.y, v.z)
}

pub fn print_simulation_report(simulation_report: SimulationReport) {
    let mut table = Table::new();

    table.set_header(vec!["Tick", "Position", "Motion", "Yaw", "Dimension"]);

    for (i, pearl) in simulation_report.history.iter().enumerate() {
        table.add_row(vec![
            i.to_string(),
            format_vec3(pearl.position),
            format_vec3(pearl.motion),
            format!("{:.5}", pearl.yaw),
            pearl.dimension.to_string(),
        ]);
    }
    println!("{table}")
}
