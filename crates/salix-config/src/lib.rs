//! Salix config

use std::net::SocketAddr;
use std::{env::current_dir, path::PathBuf};

use anyhow::Result;
use serde::Deserialize;

mod default_values;

//TODO: Add enum + implement in conf structs

#[derive(Deserialize, Clone, Debug)]
//Controller Config
pub struct ControllerConfig {
    listen: SocketAddr,
    cert_path: PathBuf,
    key_path: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
//Agent Config
pub struct AgentConfig {
    controller_address: String,
    #[serde(default = "default_values::default_agent_cert_path")]
    cert_path: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
// General Config
pub struct Config {
    controller: ControllerConfig,
    agent: AgentConfig,
}

// Get configuration from sources
pub fn get_config(config_path: Option<PathBuf>) -> Result<Config> {
    let config_path = if let Some(config_path) = config_path {
        config_path
    } else {
        let mut config_path = current_dir()?;
        config_path.push("salix.toml");
        config_path
    };

    // TODO: Add env source
    let settings = config::Config::builder()
        .add_source(config::File::from(config_path))
        .add_source(config::Environment::with_prefix("SALIX"))
        .build()?;
    Ok(settings.try_deserialize()?)
}
