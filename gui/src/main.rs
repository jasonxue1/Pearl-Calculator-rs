mod app;
mod constants;
mod models;
mod parsing;

fn main() -> Result<(), eframe::Error> {
    app::run()
}
