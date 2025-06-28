//! Salix
use anyhow::Result;
use clap::Parser;
use salix_agent::{Cli, run};

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli)
}
