use pearl_calculator::Dimension;

use crate::i18n::Language;

pub(crate) struct PearlGuiApp {
    pub(crate) config_path: String,
    pub(crate) status: Option<StatusMessage>,
    pub(crate) active_tab: AppTab,
    pub(crate) language: Language,

    pub(crate) calc_target_x: String,
    pub(crate) calc_target_z: String,
    pub(crate) calc_max_red: String,
    pub(crate) calc_max_blue: String,
    pub(crate) calc_max_error: String,
    pub(crate) calc_max_time: String,
    pub(crate) calc_show_first: String,
    pub(crate) calc_dimension: DimensionOption,
    pub(crate) calc_view: Option<CalculationView>,

    pub(crate) sim_direction: String,
    pub(crate) sim_red: String,
    pub(crate) sim_blue: String,
    pub(crate) sim_time: String,
    pub(crate) sim_to_end_time: String,
    pub(crate) sim_view: Option<SimulationView>,

    pub(crate) conv_direction: String,
    pub(crate) conv_red: String,
    pub(crate) conv_blue: String,
    pub(crate) conv_code: String,
}

impl Default for PearlGuiApp {
    fn default() -> Self {
        Self {
            config_path: "test-config/config.json".to_string(),
            status: None,
            active_tab: AppTab::Calculation,
            language: Language::default(),
            calc_target_x: "0".to_string(),
            calc_target_z: "0".to_string(),
            calc_max_red: String::new(),
            calc_max_blue: String::new(),
            calc_max_error: String::new(),
            calc_max_time: String::new(),
            calc_show_first: String::new(),
            calc_dimension: DimensionOption::Nether,
            calc_view: None,
            sim_direction: "0".to_string(),
            sim_red: "0".to_string(),
            sim_blue: "0".to_string(),
            sim_time: String::new(),
            sim_to_end_time: String::new(),
            sim_view: None,
            conv_direction: "0".to_string(),
            conv_red: "0".to_string(),
            conv_blue: "0".to_string(),
            conv_code: String::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusKind {
    Success,
    Error,
}

#[derive(Clone)]
pub(crate) struct StatusMessage {
    pub(crate) kind: StatusKind,
    pub(crate) text: String,
}

impl StatusMessage {
    pub(crate) fn success(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Success,
            text: text.into(),
        }
    }

    pub(crate) fn error(text: impl Into<String>) -> Self {
        Self {
            kind: StatusKind::Error,
            text: text.into(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct SimulationRowView {
    pub(crate) tick: usize,
    pub(crate) pos_x: f64,
    pub(crate) pos_y: f64,
    pub(crate) pos_z: f64,
    pub(crate) vel_x: f64,
    pub(crate) vel_y: f64,
    pub(crate) vel_z: f64,
    pub(crate) yaw: f64,
    pub(crate) dim: String,
}

#[derive(Clone)]
pub(crate) struct SimulationView {
    pub(crate) rows: Vec<SimulationRowView>,
    pub(crate) final_pos: [f64; 3],
    pub(crate) end_portal_pos: Option<[f64; 3]>,
}

#[derive(Clone)]
pub(crate) struct CalculationRowView {
    pub(crate) time: u64,
    pub(crate) dir: usize,
    pub(crate) red: u64,
    pub(crate) blue: u64,
    pub(crate) error: f64,
    pub(crate) pos_x: f64,
    pub(crate) pos_y: f64,
    pub(crate) pos_z: f64,
    pub(crate) to_end_time: Option<u64>,
    pub(crate) portal_pos: Option<[f64; 3]>,
}

#[derive(Clone)]
pub(crate) struct CalculationView {
    pub(crate) rows: Vec<CalculationRowView>,
    pub(crate) show_to_end_time: bool,
    pub(crate) show_end_portal_pos: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppTab {
    Calculation,
    Simulation,
    Convert,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DimensionOption {
    Overworld,
    #[default]
    Nether,
    End,
}

impl DimensionOption {
    pub(crate) fn to_dimension(self) -> Dimension {
        match self {
            Self::Overworld => Dimension::Overworld,
            Self::Nether => Dimension::Nether,
            Self::End => Dimension::End,
        }
    }
}

pub(crate) fn build_calculation_view(
    reports: &[pearl_calculator::CalculationReport],
) -> CalculationView {
    let show_to_end_time = reports.iter().any(|r| r.to_end_time.is_some());
    let show_end_portal_pos = reports.iter().any(|r| r.end_portal_pos.is_some());

    let rows = reports
        .iter()
        .map(|r| CalculationRowView {
            time: r.end_time.0,
            dir: r.rb.direction,
            red: r.rb.num.red,
            blue: r.rb.num.blue,
            error: r.error,
            pos_x: r.final_pos.0.x,
            pos_y: r.final_pos.0.y,
            pos_z: r.final_pos.0.z,
            to_end_time: r.to_end_time.map(|v| v.0),
            portal_pos: r.end_portal_pos.map(|p| [p.0.x, p.0.y, p.0.z]),
        })
        .collect();

    CalculationView {
        rows,
        show_to_end_time,
        show_end_portal_pos,
    }
}
