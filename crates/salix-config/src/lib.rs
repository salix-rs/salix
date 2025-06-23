//! Salix config

use log::{self, Level};
use std::net::SocketAddr;
use std::{env::current_dir, path::PathBuf};

use anyhow::Result;
use serde::Deserialize;

mod default_values;
mod utils;

fn deserialize_log_level<'de, D>(deserializer: D) -> Result<Level, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    match s.to_ascii_lowercase().as_str() {
        "error" => Ok(Level::Error),
        "warn" => Ok(Level::Warn),
        "info" => Ok(Level::Info),
        "debug" => Ok(Level::Debug),
        "trace" => Ok(Level::Trace),
        _ => Err(serde::de::Error::custom(format!(
            "invalid log level: {}",
            s
        ))),
    }
}

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
    #[serde(
        default = "default_values::default_config_log_level",
        deserialize_with = "deserialize_log_level"
    )]
    log_level: Level,
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
