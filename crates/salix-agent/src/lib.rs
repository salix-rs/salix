//! Salix

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

/// CLI arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

/// Main function
#[tokio::main]
pub async fn run(_cli: Cli) -> Result<()> {
    Ok(())
}
