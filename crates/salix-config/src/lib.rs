//! Salix config

use log::{self, Level};
use std::{env::current_dir, net::SocketAddr, path::PathBuf};

use anyhow::Result;
use serde::Deserialize;

pub fn default_agent_cert_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push("salix.pem");
    return path;
}

pub fn default_config_log_level() -> log::Level {
    return log::Level::Info;
}

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

/// Controller Config
#[derive(Deserialize, Clone, Debug)]
pub struct ControllerConfig {
    pub listen: SocketAddr,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

/// Agent Config
#[derive(Deserialize, Clone, Debug)]
pub struct AgentConfig {
    pub controller_address: String,
    #[serde(default = "default_agent_cert_path")]
    pub cert_path: PathBuf,
}

/// Main Config
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub controller: ControllerConfig,
    pub agent: AgentConfig,
    #[serde(
        default = "default_config_log_level",
        deserialize_with = "deserialize_log_level"
    )]
    pub log_level: Level,
}

/// Get configuration from sources
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
