//! Salix controller
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use controller::Controller;
use salix_config::get_config;
use web::Web;

mod controller;
mod web;

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
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let endpoint = Controller::make_endpoint(&config).await?;
    let controller = Controller::new();

    let web = Web::new();

    // TODO: abort when one of those fails
    let (controller_result, web_result) = tokio::join!(controller.run(endpoint), web.run());
    controller_result?;
    web_result?;

    Ok(())
}
