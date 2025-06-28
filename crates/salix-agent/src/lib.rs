//! Salix agent

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use gethostname::gethostname;
use salix_config::get_config;
use salix_proto::{RegistrationRequest, controller_service_client::ControllerServiceClient};
use uuid::Uuid;

/// CLI arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

/// Main function
#[tokio::main]
pub async fn run(cli: Cli) -> Result<()> {
    let config = get_config(cli.config)?;

    let agent_id = Uuid::now_v7();

    let mut controller_client =
        ControllerServiceClient::connect(config.agent.controller_address).await?;

    let request = tonic::Request::new(RegistrationRequest {
        agent_id: agent_id.to_string().into(),
        hostname: gethostname().into_string().unwrap_or_default().into(),
        version: env!("CARGO_PKG_VERSION").to_owned().into(),
        timestamp: 0.into(),
    });
    let response = controller_client.register(request).await?;

    println!("{:?}", &response);

    Ok(())
}
