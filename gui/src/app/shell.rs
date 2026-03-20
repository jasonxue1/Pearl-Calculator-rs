use eframe::egui;

use crate::constants::LEFT_PANEL_WIDTH;
use crate::models::{AppTab, PearlGuiApp, StatusKind};

pub(crate) fn run() -> Result<(), eframe::Error> {
    let mut app_icon = eframe::icon_data::from_png_bytes(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../assets/icon-256.png"
    )))
    .expect("failed to load app icon from assets/icon.png");
    premultiply_icon_alpha(&mut app_icon);

    let options = eframe::NativeOptions {
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
            Ok(Box::new(PearlGuiApp::default()))
        }),
    )
}

fn apply_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),
        egui::FontData::from_static(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/NotoSansSC-Regular.ttf"
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::apply_style(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pearl Calculator");

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, AppTab::Calculation, "calculation");
                ui.selectable_value(&mut self.active_tab, AppTab::Simulation, "simulation");
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(LEFT_PANEL_WIDTH, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |left| {
                        left.group(|ui| {
                            ui.heading("Input");
                            ui.add_space(4.0);
                            ui.label("Config Path");
                            ui.add_sized(
                                [ui.available_width(), 0.0],
                                egui::TextEdit::singleline(&mut self.config_path),
                            );
                            ui.separator();

                            match self.active_tab {
                                AppTab::Calculation => self.render_calculation_input_panel(ui),
                                AppTab::Simulation => self.render_simulation_input_panel(ui),
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
                            ui.label(egui::RichText::new("Output").heading());
                            ui.add_space(2.0);
                            if let Some(status) = &self.status {
                                let color = match status.kind {
                                    StatusKind::Error => egui::Color32::from_rgb(176, 0, 32),
                                    StatusKind::Success => egui::Color32::from_rgb(20, 120, 70),
                                };
                                let text = match status.kind {
                                    StatusKind::Error if !status.text.starts_with("error: ") => {
                                        format!("error: {}", status.text)
                                    }
                                    StatusKind::Success => "success".to_string(),
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
                            }
                        });
                    },
                );
            });
        });
    }
}

impl PearlGuiApp {
    fn apply_style(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.interact_size.y = 26.0;
        style.spacing.text_edit_width = crate::constants::FORM_FIELD_WIDTH;
        ctx.set_style(style);
    }
}
