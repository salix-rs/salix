//! Salix controller
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use controller::Controller;
use salix_config::get_config;

mod controller;

/// CLI arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

/// Main function
#[tokio::main]
pub async fn run(cli: Cli) -> Result<()> {
    let config = get_config(cli.config)?;

    let endpoint = Controller::make_endpoint(&config).await?;
    let controller = Controller::new();
    controller.run(endpoint).await?;

    Ok(())
}
