//! Salix
use std::{env::current_dir, fs::File, io::Write, path::PathBuf};

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use rcgen::{CertifiedKey, generate_simple_self_signed};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Controller(salix_controller::Cli),
    Agent(salix_agent::Cli),
    GenConfig {
        #[arg(short, long, value_name = "DIR")]
        directory: Option<PathBuf>,
    },
}

const DEFAULT_CONFIG: &str = r#"
log_level = "debug"

[controller]
listen = "[::1]:1312"
cert_path = "salix.pem"
key_path = "salix.key"

[agent]
controller_address = "http://localhost:1312"
cert_path = "salix.pem"
"#;

fn gen_config(directory: Option<PathBuf>) -> Result<()> {
    let directory = directory.unwrap_or(current_dir()?);
    let dir_path = directory.as_path();
    if !directory.is_dir() {
        return Err(anyhow!("{} must be a directory.", dir_path.display()));
    }

    let CertifiedKey { cert, signing_key } =
        generate_simple_self_signed(vec!["localhost".to_owned()])?;

    let mut cert_path = directory.clone();
    cert_path.push("salix.pem");
    let mut cert_file = File::create(cert_path.as_path())?;
    cert_file.write_all(cert.pem().as_bytes())?;

    let mut key_path = directory.clone();
    key_path.push("salix.key");
    let mut key_file = File::create(key_path.as_path())?;
    key_file.write_all(signing_key.serialize_pem().as_bytes())?;

    let mut config_path = directory.clone();
    config_path.push("salix.toml");
    let mut config_file = File::create(config_path.as_path())?;
    config_file.write_all(DEFAULT_CONFIG.as_bytes())?;

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Controller(args) => salix_controller::run(args),
        Commands::Agent(args) => salix_agent::run(args),
        Commands::GenConfig { directory } => gen_config(directory),
    }
}
