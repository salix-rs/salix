//! Salix config

use std::{env::current_dir, path::PathBuf};

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct ControllerConfig {
    listen: String,
    cert_path: PathBuf,
    key_path: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AgentConfig {
    controller_address: String,
    cert_path: PathBuf,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    controller: ControllerConfig,
    agent: AgentConfig,
}

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
        .build()?;
    Ok(settings.try_deserialize()?)
}
