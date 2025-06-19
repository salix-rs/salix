// Defining default values for the config file

use std::path::PathBuf;

pub fn default_agent_cert_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push("salix.pem");
    return path;
}