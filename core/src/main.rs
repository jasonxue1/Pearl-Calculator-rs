use color_eyre::{config::HookBuilder, eyre::Result};

mod cli;

fn main() -> Result<()> {
    HookBuilder::default()
        .display_env_section(false)
        .display_location_section(false)
        .install()?;
    cli::run()
}
