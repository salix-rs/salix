//! Salix

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use salix_config::get_config;

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
    let _config = get_config(cli.config)?;
    dbg!(_config);
    Ok(())
}
