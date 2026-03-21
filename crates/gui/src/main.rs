#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod constants;
mod i18n;
mod models;
mod parsing;
mod settings;

fn main() -> Result<(), eframe::Error> {
    app::run()
}
