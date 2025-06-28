//! Salix
use anyhow::Result;
use clap::Parser;
use salix_controller::{Cli, run};

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli)
}
