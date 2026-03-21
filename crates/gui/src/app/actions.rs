use std::{fs, path::Path};

use nalgebra::Vector2;
use pearl_calculator::{
    Config, RB, Root, TNTNumRB, Time, calculation as core_calculation,
    simulation as core_simulation,
};

use crate::models::{
    PearlGuiApp, SimulationRowView, SimulationView, StatusMessage, build_calculation_view,
};
use crate::parsing::{
    parse_optional_f64, parse_optional_u64, parse_optional_usize, parse_required_i64,
    parse_required_u64, parse_required_usize,
};

impl PearlGuiApp {
    pub(super) fn set_error(&mut self, message: impl Into<String>) {
        self.status = Some(StatusMessage::error(message.into()));
    }

    pub(super) fn set_success(&mut self, message: impl Into<String>) {
        let _ = message.into();
        self.status = Some(StatusMessage::success("success"));
    }

    fn load_config(&self) -> Result<Config, String> {
        let path = Path::new(&self.config_path);
        let text = fs::read_to_string(path)
            .map_err(|e| format!("failed to read config '{}': {}", path.display(), e))?;
        let root: Root = serde_json::from_str(&text)
            .map_err(|e| format!("failed to parse config json '{}': {}", path.display(), e))?;
        let config = Config::try_from(root).map_err(|e| e.to_string())?;
        config.check().map_err(|e| e.to_string())?;
        Ok(config)
    }

    pub(super) fn run_calculation(&mut self) {
        let config = match self.load_config() {
            Ok(config) => config,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let x = match parse_required_i64(&self.calc_target_x, "target x") {
            Ok(v) => v,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };
        let z = match parse_required_i64(&self.calc_target_z, "target z") {
            Ok(v) => v,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let max_red = match parse_optional_u64(&self.calc_max_red, "max red") {
            Ok(v) => v,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };
        let max_blue = match parse_optional_u64(&self.calc_max_blue, "max blue") {
            Ok(v) => v,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };
        let max_tnt = match (max_red, max_blue) {
            (None, None) => None,
            (Some(red), Some(blue)) => Some(TNTNumRB { red, blue }),
            _ => {
                self.set_error("max red / max blue must be both empty or both provided");
                return;
            }
        };

        let max_error = match parse_optional_f64(&self.calc_max_error, "max error") {
            Ok(v) => v,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let max_time = match parse_optional_u64(&self.calc_max_time, "max time") {
            Ok(v) => v.map(Time),
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let show_first = match parse_optional_usize(&self.calc_show_first, "show first") {
            Ok(v) => v,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        match core_calculation(
            &config,
            max_tnt,
            Vector2::new(x, z),
            max_error,
            max_time,
            Some(self.calc_dimension.to_dimension()),
            show_first,
        ) {
            Ok(reports) => {
                self.set_success("success");
                self.calc_view = Some(build_calculation_view(&reports));
            }
            Err(err) => {
                self.set_error(err.to_string());
            }
        }
    }

    pub(super) fn run_simulation(&mut self) {
        let config = match self.load_config() {
            Ok(config) => config,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let direction = match parse_required_usize(&self.sim_direction, "direction") {
            Ok(v) => v,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };
        if direction > 3 {
            self.set_error("direction must be in range 0..=3");
            return;
        }

        let red = match parse_required_u64(&self.sim_red, "red") {
            Ok(v) => v,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let blue = match parse_required_u64(&self.sim_blue, "blue") {
            Ok(v) => v,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let time = match parse_optional_u64(&self.sim_time, "time") {
            Ok(v) => v.map(Time),
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let to_end_time = match parse_optional_u64(&self.sim_to_end_time, "to_end_time") {
            Ok(v) => v.map(Time),
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let rb = RB {
            num: TNTNumRB { red, blue },
            direction,
        };

        match core_simulation(&config, rb, time, to_end_time) {
            Ok(report) => {
                self.set_success("success");
                self.sim_view = Some(SimulationView {
                    rows: report
                        .history
                        .iter()
                        .enumerate()
                        .map(|(tick, pearl)| SimulationRowView {
                            tick,
                            pos_x: pearl.position.0.x,
                            pos_y: pearl.position.0.y,
                            pos_z: pearl.position.0.z,
                            vel_x: pearl.motion.0.x,
                            vel_y: pearl.motion.0.y,
                            vel_z: pearl.motion.0.z,
                            yaw: pearl.yaw.0 as f64,
                            dim: pearl.dimension.to_string(),
                        })
                        .collect(),
                    final_pos: [
                        report.final_pos.0.x,
                        report.final_pos.0.y,
                        report.final_pos.0.z,
                    ],
                    end_portal_pos: report.end_portal_pos.map(|p| [p.0.x, p.0.y, p.0.z]),
                });
            }
            Err(err) => {
                self.set_error(err.to_string());
            }
        }
    }
}
