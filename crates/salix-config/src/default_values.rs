// Defining default values for the config file

use std::{fmt::Debug, path::PathBuf};

pub fn default_agent_cert_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push("salix.pem");
    return path;
}

pub fn default_config_log_level() -> log::Level {
    return log::Level::Info;
}
