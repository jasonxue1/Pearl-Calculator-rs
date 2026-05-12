use eframe::egui;

use crate::constants::LEFT_PANEL_WIDTH;
use crate::i18n::{Language, Translator};
use crate::models::{AppTab, PearlGuiApp, StatusKind};
use crate::settings;

const MIT_LICENSE_URL: &str = "https://opensource.org/licenses/MIT";
const APACHE_LICENSE_URL: &str = "https://www.apache.org/licenses/LICENSE-2.0";
const GITHUB_URL: &str = "https://github.com/jasonxue1/Pearl-Calculator-rs";
const HOMEPAGE_URL: &str = "https://pearl.jasonxue.dev";

pub(crate) fn run() -> Result<(), eframe::Error> {
    let mut app_icon = eframe::icon_data::from_png_bytes(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/icons/icon-256.png"
    )))
    .expect("failed to load app icon from assets/icon.png");
    premultiply_icon_alpha(&mut app_icon);

    let options = eframe::NativeOptions {
        run_and_return: false,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([980.0, 640.0])
            .with_icon(app_icon),
        ..Default::default()
    };
    eframe::run_native(
        "Pearl Calculator",
        options,
        Box::new(|cc| {
            apply_fonts(&cc.egui_ctx);
            let mut app = PearlGuiApp::default();
            if let Some(language) = settings::load_language() {
                app.language = language;
            }
            app.initialize_config_store();
            Ok(Box::new(app))
        }),
    )
}

fn apply_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),
        egui::FontData::from_static(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/NotoSansSC-Regular.ttf"
        )))
        .into(),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans_sc".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "noto_sans_sc".to_owned());
    ctx.set_fonts(fonts);
}

fn premultiply_icon_alpha(icon: &mut egui::IconData) {
    for pixel in icon.rgba.chunks_exact_mut(4) {
        let alpha = pixel[3] as u16;
        if alpha == 0 {
            pixel[0] = 0;
            pixel[1] = 0;
            pixel[2] = 0;
            continue;
        }

        // Some native window-icon paths expect premultiplied alpha.
        pixel[0] = ((pixel[0] as u16 * alpha + 127) / 255) as u8;
        pixel[1] = ((pixel[1] as u16 * alpha + 127) / 255) as u8;
        pixel[2] = ((pixel[2] as u16 * alpha + 127) / 255) as u8;
    }
}

impl eframe::App for PearlGuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let should_close_by_shortcut =
            ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::W));
        if should_close_by_shortcut {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        Self::apply_style(&ctx);
        self.refresh_available_configs();
        let tr = Translator::new(self.language);

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(tr.t("app-title"));
                let remaining = ui.available_width();
                ui.allocate_ui_with_layout(
                    egui::vec2(remaining, 0.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.menu_button(tr.t("settings-button"), |ui| {
                            let previous_language = self.language;
                            ui.label(tr.t("language-label"));
                            ui.separator();
                            ui.selectable_value(
                                &mut self.language,
                                Language::English,
                                tr.t("language-option-english"),
                            );
                            ui.selectable_value(
                                &mut self.language,
                                Language::SimplifiedChinese,
                                tr.t("language-option-zh-cn"),
                            );
                            if previous_language != self.language {
                                let _ = settings::save_language(self.language);
                            }
                        });

                        if ui.button(tr.t("settings-import-config")).clicked() {
                            self.import_config_from_system_picker();
                        }

                        let selected_text = self
                            .selected_config
                            .clone()
                            .unwrap_or_else(|| tr.t("settings-config-none"));
                        let mut selected_to_apply: Option<String> = None;
                        egui::ComboBox::from_id_salt("topbar_config_select")
                            .selected_text(selected_text)
                            .width(220.0)
                            .show_ui(ui, |ui| {
                                for file_name in &self.available_configs {
                                    let is_selected =
                                        self.selected_config.as_deref() == Some(file_name);
                                    if ui.selectable_label(is_selected, file_name).clicked() {
                                        selected_to_apply = Some(file_name.clone());
                                    }
                                }
                            });
                        if let Some(file_name) = selected_to_apply {
                            self.select_config_from_settings(&file_name);
                        }
                    },
                );
            });

            ui.add_space(2.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(format!("{}:", tr.t("meta-license"))).weak());
                ui.hyperlink_to("MIT License", MIT_LICENSE_URL);
                ui.label("|");
                ui.hyperlink_to("Apache License Version 2.0", APACHE_LICENSE_URL);
                ui.separator();
                ui.hyperlink_to(tr.t("meta-github"), GITHUB_URL);
                ui.separator();
                ui.hyperlink_to(tr.t("meta-homepage"), HOMEPAGE_URL);
            });

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.active_tab,
                    AppTab::Calculation,
                    tr.t("tab-calculation"),
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    AppTab::Simulation,
                    tr.t("tab-simulation"),
                );
                ui.selectable_value(&mut self.active_tab, AppTab::Convert, tr.t("tab-convert"));
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(LEFT_PANEL_WIDTH, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |left| {
                        left.group(|ui| {
                            ui.heading(tr.t("input"));
                            ui.separator();

                            match self.active_tab {
                                AppTab::Calculation => self.render_calculation_input_panel(ui),
                                AppTab::Simulation => self.render_simulation_input_panel(ui),
                                AppTab::Convert => self.render_convert_input_panel(ui),
                            }
                        });
                    },
                );

                let right_width = ui.available_width();
                ui.allocate_ui_with_layout(
                    egui::vec2(right_width, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |right| {
                        right.group(|ui| {
                            ui.label(egui::RichText::new(tr.t("output")).heading());
                            ui.add_space(2.0);
                            if let Some(status) = &self.status {
                                let color = match status.kind {
                                    StatusKind::Error => egui::Color32::from_rgb(176, 0, 32),
                                    StatusKind::Success => egui::Color32::from_rgb(20, 120, 70),
                                };
                                let error_prefix = tr.t("error-prefix");
                                let text = match status.kind {
                                    StatusKind::Error
                                        if !status.text.starts_with(&error_prefix) =>
                                    {
                                        format!("{error_prefix}{}", status.text)
                                    }
                                    StatusKind::Success => tr.t("status-success"),
                                    _ => status.text.clone(),
                                };
                                ui.label(egui::RichText::new(text).color(color).strong());
                                ui.add_space(2.0);
                            }
                            ui.separator();
                            ui.add_space(2.0);

                            match self.active_tab {
                                AppTab::Calculation => self.render_calculation_output_panel(ui),
                                AppTab::Simulation => self.render_simulation_output_panel(ui),
                                AppTab::Convert => self.render_convert_output_panel(ui),
                            }
                        });
                    },
                );
            });
        });

        self.render_import_conflict_dialog(&ctx);
    }
}

impl PearlGuiApp {
    fn apply_style(ctx: &egui::Context) {
        let mut style = ctx.global_style().as_ref().clone();
        style.visuals = egui::Visuals::light();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.interact_size.y = 26.0;
        style.spacing.text_edit_width = crate::constants::FORM_FIELD_WIDTH;
        ctx.set_global_style(style);
    }

    fn render_import_conflict_dialog(&mut self, ctx: &egui::Context) {
        if self.import_conflict_source.is_none() {
            return;
        }

        let tr = Translator::new(self.language);
        egui::Window::new(tr.t("settings-import-conflict-title"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(tr.t_args(
                    "settings-import-conflict-message",
                    &[("name", self.import_conflict_name.clone())],
                ));
                ui.label(tr.t("settings-import-conflict-rename-label"));
                ui.add_sized(
                    [360.0, 0.0],
                    egui::TextEdit::singleline(&mut self.import_rename_name),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(tr.t("settings-import-cancel")).clicked() {
                        self.cancel_import_conflict();
                    }
                    if ui.button(tr.t("settings-import-rename")).clicked() {
                        self.import_conflict_rename();
                    }
                    if ui.button(tr.t("settings-import-overwrite")).clicked() {
                        self.import_conflict_overwrite();
                    }
                });
            });
    }
}
