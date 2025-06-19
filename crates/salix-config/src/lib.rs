//! Salix config

use std::os::unix::net::SocketAddr;
use std::{env::current_dir, path::PathBuf};

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

//TODO: Add enum + implement in conf structs

#[derive(Deserialize, Clone, Debug)]
//Controller Config
pub struct ControllerConfig {
    listen: String,
    cert_path: PathBuf,
    key_path: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
//Agent Config
pub struct AgentConfig {
    #[serde(default = "default_controller_address")]
    controller_address: String,
    cert_path: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
// General Config
pub struct Config {
    controller: ControllerConfig,
    agent: AgentConfig,
}

pub fn default_controller_address() -> String {
    return "127.0.0.1".to_string();
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
