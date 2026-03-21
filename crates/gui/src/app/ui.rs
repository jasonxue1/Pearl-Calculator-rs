use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::constants::{
    FORM_FIELD_WIDTH, TABLE_COMPACT_COL_WIDTH, TABLE_DIM_COL_WIDTH, TABLE_ERROR_COL_WIDTH,
    TABLE_MAX_HEIGHT, TABLE_VEC3_COL_WIDTH, TABLE_YAW_COL_WIDTH,
};
use crate::i18n::Translator;
use crate::models::{CalculationRowView, DimensionOption, PearlGuiApp, SimulationRowView};

impl PearlGuiApp {
    fn form_label(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) {
        ui.add_sized([170.0, 0.0], egui::Label::new(text));
    }

    fn form_input(ui: &mut egui::Ui, value: &mut String) {
        ui.add_sized([FORM_FIELD_WIDTH, 0.0], egui::TextEdit::singleline(value));
    }

    fn form_row(ui: &mut egui::Ui, label: impl Into<egui::WidgetText>, value: &mut String) {
        ui.horizontal(|ui| {
            Self::form_label(ui, label);
            Self::form_input(ui, value);
        });
    }

    pub(super) fn render_calculation_input_panel(&mut self, ui: &mut egui::Ui) {
        let tr = Translator::new(self.language);
        ui.label(tr.t("calculation-parameters"));
        ui.add_space(4.0);

        Self::form_row(ui, tr.t("target-x"), &mut self.calc_target_x);
        Self::form_row(ui, tr.t("target-z"), &mut self.calc_target_z);
        Self::form_row(ui, tr.t("max-tnt-red-optional"), &mut self.calc_max_red);
        Self::form_row(ui, tr.t("max-tnt-blue-optional"), &mut self.calc_max_blue);
        Self::form_row(ui, tr.t("max-error-optional"), &mut self.calc_max_error);
        Self::form_row(ui, tr.t("max-time-optional"), &mut self.calc_max_time);
        Self::form_row(ui, tr.t("show-first-optional"), &mut self.calc_show_first);

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(tr.t("dimension"));
            ui.selectable_value(
                &mut self.calc_dimension,
                DimensionOption::Overworld,
                tr.t("dimension-overworld"),
            );
            ui.selectable_value(
                &mut self.calc_dimension,
                DimensionOption::Nether,
                tr.t("dimension-nether"),
            );
            ui.selectable_value(
                &mut self.calc_dimension,
                DimensionOption::End,
                tr.t("dimension-end"),
            );
        });

        ui.add_space(2.0);
        if ui.button(tr.t("run-calculation")).clicked() {
            self.run_calculation();
        }
    }

    pub(super) fn render_calculation_output_panel(&mut self, ui: &mut egui::Ui) {
        let tr = Translator::new(self.language);
        ui.spacing_mut().item_spacing.x = 10.0;
        let view = self.calc_view.as_ref();
        let rows: &[CalculationRowView] = view.map_or(&[], |v| v.rows.as_slice());
        let show_to_end_time = view.is_some_and(|v| v.show_to_end_time);
        let show_end_portal_pos = view.is_some_and(|v| v.show_end_portal_pos);

        if let Some(view) = view {
            if view.rows.is_empty() {
                ui.label(tr.t("no-calculation-results"));
            } else {
                ui.label(tr.t_count("calculation-finished", view.rows.len()));
            }
            ui.add_space(4.0);
        }
        let table_id = format!(
            "calculation_table_te{}_ep{}",
            show_to_end_time as u8, show_end_portal_pos as u8
        );

        let mut table = TableBuilder::new(ui)
            .id_salt(table_id)
            .striped(true)
            .resizable(true)
            .vscroll(true)
            .max_scroll_height(TABLE_MAX_HEIGHT)
            .column(Column::exact(TABLE_COMPACT_COL_WIDTH))
            .column(Column::exact(TABLE_COMPACT_COL_WIDTH))
            .column(Column::exact(TABLE_COMPACT_COL_WIDTH))
            .column(Column::exact(TABLE_COMPACT_COL_WIDTH))
            .column(Column::exact(TABLE_ERROR_COL_WIDTH))
            .column(Column::exact(TABLE_VEC3_COL_WIDTH));

        if show_to_end_time {
            table = table.column(Column::exact(TABLE_COMPACT_COL_WIDTH + 8.0));
        }
        if show_end_portal_pos {
            table = table.column(Column::exact(TABLE_VEC3_COL_WIDTH));
        }

        table
            .header(24.0, |mut header| {
                header.col(|ui| {
                    ui.strong(tr.t("header-time"));
                });
                header.col(|ui| {
                    ui.strong(tr.t("header-dir"));
                });
                header.col(|ui| {
                    ui.strong(tr.t("header-red"));
                });
                header.col(|ui| {
                    ui.strong(tr.t("header-blue"));
                });
                header.col(|ui| {
                    ui.strong(tr.t("header-error"));
                });
                header.col(|ui| {
                    ui.strong(tr.t("header-pos"));
                });
                if show_to_end_time {
                    header.col(|ui| {
                        ui.strong(tr.t("header-to-end"));
                    });
                }
                if show_end_portal_pos {
                    header.col(|ui| {
                        ui.strong(tr.t("header-portal"));
                    });
                }
            })
            .body(|body| {
                body.rows(24.0, rows.len(), |mut row| {
                    let item = &rows[row.index()];
                    row.col(|ui| {
                        ui.monospace(item.time.to_string());
                    });
                    row.col(|ui| {
                        ui.monospace(item.dir.to_string());
                    });
                    row.col(|ui| {
                        ui.monospace(item.red.to_string());
                    });
                    row.col(|ui| {
                        ui.monospace(item.blue.to_string());
                    });
                    row.col(|ui| {
                        ui.monospace(format!("{:.5}", item.error));
                    });
                    row.col(|ui| {
                        ui.monospace(format!(
                            "({:.2}, {:.2}, {:.2})",
                            item.pos_x, item.pos_y, item.pos_z
                        ));
                    });
                    if show_to_end_time {
                        row.col(|ui| {
                            match item.to_end_time {
                                Some(v) => ui.monospace(v.to_string()),
                                None => ui.monospace(""),
                            };
                        });
                    }
                    if show_end_portal_pos {
                        row.col(|ui| {
                            match item.portal_pos {
                                Some(p) => {
                                    ui.monospace(format!("({:.2}, {:.2}, {:.2})", p[0], p[1], p[2]))
                                }
                                None => ui.monospace(""),
                            };
                        });
                    }
                });
            });
    }

    pub(super) fn render_simulation_input_panel(&mut self, ui: &mut egui::Ui) {
        let tr = Translator::new(self.language);
        ui.label(tr.t("simulation-parameters"));
        ui.add_space(4.0);

        Self::form_row(ui, tr.t("direction-range"), &mut self.sim_direction);
        Self::form_row(ui, tr.t("header-red"), &mut self.sim_red);
        Self::form_row(ui, tr.t("header-blue"), &mut self.sim_blue);
        Self::form_row(ui, tr.t("time-optional"), &mut self.sim_time);
        Self::form_row(ui, tr.t("to-end-time-optional"), &mut self.sim_to_end_time);

        ui.add_space(2.0);
        if ui.button(tr.t("run-simulation")).clicked() {
            self.run_simulation();
        }
    }

    pub(super) fn render_simulation_output_panel(&mut self, ui: &mut egui::Ui) {
        let tr = Translator::new(self.language);
        ui.spacing_mut().item_spacing.x = 6.0;
        let rows: &[SimulationRowView] = self.sim_view.as_ref().map_or(&[], |v| v.rows.as_slice());
        if let Some(view) = &self.sim_view {
            if let Some(end_portal_pos) = view.end_portal_pos {
                ui.label(format!(
                    "{}: ({:.2}, {:.2}, {:.2})",
                    tr.t("end-portal-position"),
                    end_portal_pos[0],
                    end_portal_pos[1],
                    end_portal_pos[2]
                ));
            }
            ui.label(format!(
                "{}: ({:.2}, {:.2}, {:.2})",
                tr.t("final-position"),
                view.final_pos[0],
                view.final_pos[1],
                view.final_pos[2]
            ));
            ui.add_space(4.0);
        }

        TableBuilder::new(ui)
            .id_salt("simulation_table")
            .striped(true)
            .resizable(true)
            .vscroll(true)
            .max_scroll_height(TABLE_MAX_HEIGHT)
            .column(Column::exact(TABLE_COMPACT_COL_WIDTH))
            .column(Column::remainder())
            .column(Column::remainder())
            .column(Column::exact(TABLE_YAW_COL_WIDTH))
            .column(Column::exact(TABLE_DIM_COL_WIDTH))
            .header(24.0, |mut header| {
                header.col(|ui| {
                    ui.strong(tr.t("header-gt"));
                });
                header.col(|ui| {
                    ui.strong(tr.t("header-pos"));
                });
                header.col(|ui| {
                    ui.strong(tr.t("header-vel"));
                });
                header.col(|ui| {
                    ui.strong(tr.t("header-yaw"));
                });
                header.col(|ui| {
                    ui.strong(tr.t("header-dim"));
                });
            })
            .body(|body| {
                body.rows(24.0, rows.len(), |mut row| {
                    let item = &rows[row.index()];
                    row.col(|ui| {
                        ui.monospace(item.tick.to_string());
                    });
                    row.col(|ui| {
                        ui.monospace(format!(
                            "({:.2}, {:.2}, {:.2})",
                            item.pos_x, item.pos_y, item.pos_z
                        ));
                    });
                    row.col(|ui| {
                        ui.monospace(format!(
                            "({:.5}, {:.5}, {:.5})",
                            item.vel_x, item.vel_y, item.vel_z
                        ));
                    });
                    row.col(|ui| {
                        ui.monospace(format!("{:.5}", item.yaw));
                    });
                    row.col(|ui| {
                        ui.monospace(&item.dim);
                    });
                });
            });
    }
}
