use std::{fs, path::PathBuf};

use nalgebra::Vector2;
use pearl_calculator::{
    CodeItem, Config, PearlError, RB, Root, TNTNumCode, TNTNumRB, Time,
    calculation as core_calculation, code_to_rb as core_code_to_rb, rb_to_code as core_rb_to_code,
    simulation as core_simulation,
};

use crate::i18n::{Language, Translator};
use crate::models::{
    PearlGuiApp, SimulationRowView, SimulationView, StatusMessage, build_calculation_view,
};
use crate::parsing::{
    ParseError, parse_optional_f64, parse_optional_u64, parse_optional_usize, parse_required_i64,
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
        load_config_local(&self.config_path)
            .map_err(|e| localize_config_load_error(self.language, &e))
    }

    pub(super) fn run_calculation(&mut self) {
        normalize_trim(&mut self.config_path);
        normalize_compact(&mut self.calc_target_x);
        normalize_compact(&mut self.calc_target_z);
        normalize_compact(&mut self.calc_max_red);
        normalize_compact(&mut self.calc_max_blue);
        normalize_compact(&mut self.calc_max_error);
        normalize_compact(&mut self.calc_max_time);
        normalize_compact(&mut self.calc_show_first);

        let tr = Translator::new(self.language);
        let config = match self.load_config() {
            Ok(config) => config,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let x = match parse_required_i64(&self.calc_target_x, &tr.t("target-x")) {
            Ok(v) => v,
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };
        let z = match parse_required_i64(&self.calc_target_z, &tr.t("target-z")) {
            Ok(v) => v,
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };

        let max_red = match parse_optional_u64(&self.calc_max_red, &tr.t("max-tnt-red-optional")) {
            Ok(v) => v,
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };
        let max_blue = match parse_optional_u64(&self.calc_max_blue, &tr.t("max-tnt-blue-optional"))
        {
            Ok(v) => v,
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };
        let max_tnt = match (max_red, max_blue) {
            (None, None) => None,
            (Some(red), Some(blue)) => Some(TNTNumRB { red, blue }),
            _ => {
                self.set_error(tr.t("error-max-red-blue-pair"));
                return;
            }
        };

        let max_error = match parse_optional_f64(&self.calc_max_error, &tr.t("max-error-optional"))
        {
            Ok(v) => v,
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };

        let max_time = match parse_optional_u64(&self.calc_max_time, &tr.t("max-time-optional")) {
            Ok(v) => v.map(Time),
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };

        let show_first =
            match parse_optional_usize(&self.calc_show_first, &tr.t("show-first-optional")) {
                Ok(v) => v,
                Err(err) => {
                    self.set_error(localize_parse_error(self.language, &err));
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
                self.set_error(localize_core_error(self.language, &err));
            }
        }
    }

    pub(super) fn run_simulation(&mut self) {
        normalize_trim(&mut self.config_path);
        normalize_compact(&mut self.sim_direction);
        normalize_compact(&mut self.sim_red);
        normalize_compact(&mut self.sim_blue);
        normalize_compact(&mut self.sim_time);
        normalize_compact(&mut self.sim_to_end_time);

        let tr = Translator::new(self.language);
        let config = match self.load_config() {
            Ok(config) => config,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let direction = match parse_required_usize(&self.sim_direction, &tr.t("direction-range")) {
            Ok(v) => v,
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };
        if direction > 3 {
            self.set_error(tr.t("error-direction-range"));
            return;
        }

        let red = match parse_required_u64(&self.sim_red, &tr.t("header-red")) {
            Ok(v) => v,
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };

        let blue = match parse_required_u64(&self.sim_blue, &tr.t("header-blue")) {
            Ok(v) => v,
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };

        let time = match parse_optional_u64(&self.sim_time, &tr.t("time-optional")) {
            Ok(v) => v.map(Time),
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };

        let to_end_time =
            match parse_optional_u64(&self.sim_to_end_time, &tr.t("to-end-time-optional")) {
                Ok(v) => v.map(Time),
                Err(err) => {
                    self.set_error(localize_parse_error(self.language, &err));
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
                self.set_error(localize_core_error(self.language, &err));
            }
        }
    }

    pub(super) fn run_convert_rb_to_code(&mut self) {
        normalize_trim(&mut self.config_path);
        normalize_compact(&mut self.conv_direction);
        normalize_compact(&mut self.conv_red);
        normalize_compact(&mut self.conv_blue);

        let tr = Translator::new(self.language);
        let config = match self.load_config() {
            Ok(config) => config,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let direction = match parse_required_usize(&self.conv_direction, &tr.t("direction-range")) {
            Ok(v) => v,
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };
        if direction > 3 {
            self.set_error(tr.t("error-direction-range"));
            return;
        }

        let red = match parse_required_u64(&self.conv_red, &tr.t("header-red")) {
            Ok(v) => v,
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };

        let blue = match parse_required_u64(&self.conv_blue, &tr.t("header-blue")) {
            Ok(v) => v,
            Err(err) => {
                self.set_error(localize_parse_error(self.language, &err));
                return;
            }
        };

        match core_rb_to_code(
            &config.code,
            RB {
                num: TNTNumRB { red, blue },
                direction,
            },
        ) {
            Ok(code) => {
                self.set_success("success");
                self.conv_code = format_code_bits_with_rule(&config.code.default, &code);
            }
            Err(err) => self.set_error(localize_core_error(self.language, &err)),
        }
    }

    pub(super) fn run_convert_code_to_rb(&mut self) {
        normalize_trim(&mut self.config_path);
        normalize_compact(&mut self.conv_code);

        let config = match self.load_config() {
            Ok(config) => config,
            Err(err) => {
                self.set_error(err);
                return;
            }
        };

        let code = match parse_code_input(&self.conv_code) {
            Ok(code) => code,
            Err(err) => {
                self.set_error(localize_code_input_error(self.language, &err));
                return;
            }
        };

        match core_code_to_rb(&config.code, code) {
            Ok(rb) => {
                self.set_success("success");
                self.conv_direction = rb.direction.to_string();
                self.conv_red = rb.num.red.to_string();
                self.conv_blue = rb.num.blue.to_string();
                if let Ok(code) = core_rb_to_code(&config.code, rb) {
                    self.conv_code = format_code_bits_with_rule(&config.code.default, &code);
                }
            }
            Err(err) => self.set_error(localize_core_error(self.language, &err)),
        }
    }
}

fn parse_code_input(input: &str) -> Result<TNTNumCode, String> {
    let compact: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.is_empty() {
        return Err("code-empty".to_string());
    }

    let mut bits = Vec::with_capacity(compact.len());
    for (idx, ch) in compact.chars().enumerate() {
        match ch {
            '0' => bits.push(false),
            '1' => bits.push(true),
            _ => {
                return Err(format!("code-invalid-char:{}:{}", idx + 1, ch));
            }
        }
    }

    Ok(TNTNumCode(bits))
}

fn format_code_bits_with_rule(rule: &[CodeItem], code: &TNTNumCode) -> String {
    let bits = &code.0;
    let mut bit_idx = 0usize;
    let mut out = String::new();

    for item in rule {
        match item {
            CodeItem::Space => out.push(' '),
            CodeItem::Red { .. } | CodeItem::Blue { .. } | CodeItem::Direction { .. } => {
                match bits.get(bit_idx) {
                    Some(true) => out.push('1'),
                    Some(false) => out.push('0'),
                    None => return "rb-to-code produced fewer bits than code rule requires".into(),
                }
                bit_idx += 1;
            }
        }
    }

    if bit_idx != bits.len() {
        return "rb-to-code produced more bits than code rule requires".into();
    }

    out
}

fn localize_core_error(language: Language, err: &PearlError) -> String {
    let tr = Translator::new(language);
    match err {
        PearlError::UnsupportedConfigVersion(version) => tr.t_args(
            "core-error-unsupported-config-version",
            &[("version", version.to_string())],
        ),
        PearlError::InvalidDirectionVector(vector) => tr.t_args(
            "core-error-invalid-direction-vector",
            &[("x", vector[0].to_string()), ("y", vector[1].to_string())],
        ),
        PearlError::InvalidDirectionCombination { x, y } => tr.t_args(
            "core-error-invalid-direction-combination",
            &[("x", x.to_string()), ("y", y.to_string())],
        ),
        PearlError::DuplicateDirectionQuadrant { quadrant } => tr.t_args(
            "core-error-duplicate-direction-quadrant",
            &[("quadrant", quadrant.to_string())],
        ),
        PearlError::SimulationTimeZero => tr.t("core-error-simulation-time-zero"),
        PearlError::ToEndTimeAfterEnd { to_end_time, time } => tr.t_args(
            "core-error-to-end-time-after-end",
            &[
                ("to_end_time", to_end_time.to_string()),
                ("time", time.to_string()),
            ],
        ),
        PearlError::EndPortalTeleportFromEnd => tr.t("core-error-end-portal-teleport-from-end"),
        PearlError::Unimplemented { feature } => tr.t_args(
            "core-error-unimplemented",
            &[("feature", (*feature).to_string())],
        ),
        PearlError::UnsupportedDimension { dimension, context } => tr.t_args(
            "core-error-unsupported-dimension",
            &[
                ("dimension", dimension.to_string()),
                ("context", (*context).to_string()),
            ],
        ),
        PearlError::InvalidMaxTntArgCount(count) => tr.t_args(
            "core-error-invalid-max-tnt-arg-count",
            &[("count", count.to_string())],
        ),
        PearlError::InvalidCapBit { bit, max } => tr.t_args(
            "core-error-invalid-cap-bit",
            &[("bit", bit.to_string()), ("max", max.to_string())],
        ),
        PearlError::DuplicateCapBit { bit } => {
            tr.t_args("core-error-duplicate-cap-bit", &[("bit", bit.to_string())])
        }
        PearlError::OverlappingCapBit { bit } => tr.t_args(
            "core-error-overlapping-cap-bit",
            &[("bit", bit.to_string())],
        ),
        PearlError::CodeLengthMismatch { expected, actual } => tr.t_args(
            "core-error-code-length-mismatch",
            &[
                ("expected", expected.to_string()),
                ("actual", actual.to_string()),
            ],
        ),
        PearlError::MixedCapKinds => tr.t("core-error-mixed-cap-kinds"),
        PearlError::DirectionOutOfRange { value } => tr.t_args(
            "core-error-direction-out-of-range",
            &[("value", value.to_string())],
        ),
        PearlError::ValueOverflow => tr.t("core-error-value-overflow"),
        PearlError::NoExactEncoding { rb } => tr.t_args(
            "core-error-no-exact-encoding",
            &[
                ("direction", rb.direction.to_string()),
                ("red", rb.num.red.to_string()),
                ("blue", rb.num.blue.to_string()),
            ],
        ),
    }
}

fn localize_parse_error(language: Language, err: &ParseError) -> String {
    let tr = Translator::new(language);
    let expected = match err.expected.as_str() {
        "integer" => tr.t("parse-type-integer"),
        "number" => tr.t("parse-type-number"),
        other => other.to_string(),
    };
    tr.t_args(
        "parse-error-must-be",
        &[("field", err.name.clone()), ("expected", expected)],
    )
}

fn localize_config_load_error(language: Language, err: &GuiConfigLoadError) -> String {
    let tr = Translator::new(language);
    match err {
        GuiConfigLoadError::ReadConfig { path, source } => tr.t_args(
            "config-error-read-failed",
            &[
                ("path", path.display().to_string()),
                ("source", source.to_string()),
            ],
        ),
        GuiConfigLoadError::ParseConfigJson { path, source } => tr.t_args(
            "config-error-parse-json-failed",
            &[
                ("path", path.display().to_string()),
                ("source", source.to_string()),
            ],
        ),
        GuiConfigLoadError::Core { source } => localize_core_error(language, source),
    }
}

fn localize_code_input_error(language: Language, err: &str) -> String {
    let tr = Translator::new(language);
    if err == "code-empty" {
        return tr.t("error-code-empty");
    }

    if let Some(rest) = err.strip_prefix("code-invalid-char:") {
        let mut parts = rest.splitn(2, ':');
        if let (Some(position), Some(ch)) = (parts.next(), parts.next()) {
            return tr.t_args(
                "error-code-invalid-char",
                &[("position", position.to_string()), ("char", ch.to_string())],
            );
        }
    }

    err.to_string()
}

enum GuiConfigLoadError {
    ReadConfig {
        path: PathBuf,
        source: std::io::Error,
    },
    ParseConfigJson {
        path: PathBuf,
        source: serde_json::Error,
    },
    Core {
        source: PearlError,
    },
}

fn load_config_local(path: &str) -> Result<Config, GuiConfigLoadError> {
    let path = PathBuf::from(path);
    let text = fs::read_to_string(&path).map_err(|source| GuiConfigLoadError::ReadConfig {
        path: path.clone(),
        source,
    })?;
    let root: Root =
        serde_json::from_str(&text).map_err(|source| GuiConfigLoadError::ParseConfigJson {
            path: path.clone(),
            source,
        })?;
    let config = Config::try_from(root).map_err(|source| GuiConfigLoadError::Core { source })?;
    config
        .check()
        .map_err(|source| GuiConfigLoadError::Core { source })?;
    Ok(config)
}

fn normalize_trim(value: &mut String) {
    *value = value.trim().to_string();
}

fn normalize_compact(value: &mut String) {
    value.retain(|c| !c.is_whitespace());
}
