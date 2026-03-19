use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::constants::{
    FORM_FIELD_WIDTH, TABLE_COMPACT_COL_WIDTH, TABLE_DIM_COL_WIDTH, TABLE_ERROR_COL_WIDTH,
    TABLE_MAX_HEIGHT, TABLE_VEC3_COL_WIDTH, TABLE_YAW_COL_WIDTH,
};
use crate::models::{CalculationRowView, DimensionOption, PearlGuiApp, SimulationRowView};

impl PearlGuiApp {
    fn form_label(ui: &mut egui::Ui, text: &str) {
        ui.add_sized([170.0, 0.0], egui::Label::new(text));
    }

    fn form_input(ui: &mut egui::Ui, value: &mut String) {
        ui.add_sized([FORM_FIELD_WIDTH, 0.0], egui::TextEdit::singleline(value));
    }

    fn form_row(ui: &mut egui::Ui, label: &str, value: &mut String) {
        ui.horizontal(|ui| {
            Self::form_label(ui, label);
            Self::form_input(ui, value);
        });
    }

    pub(super) fn render_calculation_input_panel(&mut self, ui: &mut egui::Ui) {
        ui.label("Calculation Parameters");
        ui.add_space(4.0);

        Self::form_row(ui, "Target X", &mut self.calc_target_x);
        Self::form_row(ui, "Target Z", &mut self.calc_target_z);
        Self::form_row(ui, "Max TNT Red (optional)", &mut self.calc_max_red);
        Self::form_row(ui, "Max TNT Blue (optional)", &mut self.calc_max_blue);
        Self::form_row(ui, "Max Error (optional)", &mut self.calc_max_error);
        Self::form_row(ui, "Max Time (optional)", &mut self.calc_max_time);
        Self::form_row(ui, "Show First (optional)", &mut self.calc_show_first);

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Dimension");
            ui.selectable_value(
                &mut self.calc_dimension,
                DimensionOption::Overworld,
                "Overworld",
            );
            ui.selectable_value(&mut self.calc_dimension, DimensionOption::Nether, "Nether");
            ui.selectable_value(&mut self.calc_dimension, DimensionOption::End, "End");
        });

        ui.add_space(2.0);
        if ui.button("Run Calculation").clicked() {
            self.run_calculation();
        }
    }

    pub(super) fn render_calculation_output_panel(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.x = 10.0;
        let view = self.calc_view.as_ref();
        let rows: &[CalculationRowView] = view.map_or(&[], |v| v.rows.as_slice());
        let show_to_end_time = view.is_some_and(|v| v.show_to_end_time);
        let show_end_portal_pos = view.is_some_and(|v| v.show_end_portal_pos);

        if let Some(view) = view {
            if view.rows.is_empty() {
                ui.label("No calculation results.");
            } else {
                ui.label(format!(
                    "Calculation finished. {} result(s).",
                    view.rows.len()
                ));
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
                    ui.strong("Time");
                });
                header.col(|ui| {
                    ui.strong("Dir");
                });
                header.col(|ui| {
                    ui.strong("Red");
                });
                header.col(|ui| {
                    ui.strong("Blue");
                });
                header.col(|ui| {
                    ui.strong("Error");
                });
                header.col(|ui| {
                    ui.strong("Pos (x, y, z)");
                });
                if show_to_end_time {
                    header.col(|ui| {
                        ui.strong("To End");
                    });
                }
                if show_end_portal_pos {
                    header.col(|ui| {
                        ui.strong("Portal (x, y, z)");
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
        ui.label("Simulation Parameters");
        ui.add_space(4.0);

        Self::form_row(ui, "Direction (0..=3)", &mut self.sim_direction);
        Self::form_row(ui, "Red", &mut self.sim_red);
        Self::form_row(ui, "Blue", &mut self.sim_blue);
        Self::form_row(ui, "Time (optional)", &mut self.sim_time);
        Self::form_row(ui, "To End Time (optional)", &mut self.sim_to_end_time);

        ui.add_space(2.0);
        if ui.button("Run Simulation").clicked() {
            self.run_simulation();
        }
    }

    pub(super) fn render_simulation_output_panel(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.x = 6.0;
        let rows: &[SimulationRowView] = self.sim_view.as_ref().map_or(&[], |v| v.rows.as_slice());
        if let Some(view) = &self.sim_view {
            if let Some(end_portal_pos) = view.end_portal_pos {
                ui.label(format!(
                    "End portal position: ({:.2}, {:.2}, {:.2})",
                    end_portal_pos[0], end_portal_pos[1], end_portal_pos[2]
                ));
            }
            ui.label(format!(
                "Final position: ({:.2}, {:.2}, {:.2})",
                view.final_pos[0], view.final_pos[1], view.final_pos[2]
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
                    ui.strong("GT");
                });
                header.col(|ui| {
                    ui.strong("Pos (x, y, z)");
                });
                header.col(|ui| {
                    ui.strong("Vel (x, y, z)");
                });
                header.col(|ui| {
                    ui.strong("Yaw");
                });
                header.col(|ui| {
                    ui.strong("Dim");
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
