use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct CalculationReport {
    pub rb: RB,
    pub end_time: Time,
    pub error: f64,
    pub final_pos: Array,
    pub to_end_time: Option<Time>,
    pub end_portal_pos: Option<Array>,
}

#[derive(Debug)]
pub struct SimulationReport {
    pub history: Vec<Pearl>,
    pub final_pos: Array,
    pub end_portal_pos: Option<Array>,
}
