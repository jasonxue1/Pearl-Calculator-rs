mod app;
mod constants;
mod i18n;
mod models;
mod parsing;

fn main() -> Result<(), eframe::Error> {
    app::run()
}
