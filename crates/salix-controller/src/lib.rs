//! Salix controller
use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::Result;
use clap::Parser;
use salix_config::get_config;
use salix_proto::{
    MessageRequest, RegistrationRequest,
    controller_service_server::{ControllerService, ControllerServiceServer},
};
use tokio::{sync::Mutex, task::JoinSet, time::sleep};
use tonic::{Request, Response, Status, transport::Server};

/// Our own lovely constant
pub const ALPN_QUIC_SALIX: &[&[u8]] = &[b"salix"];

/// CLI arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(Debug)]
struct Agent {
    agent_id: String,
    hostname: String,
    version: String,
}

#[derive(Debug, Default)]
struct ControllerState {
    agents: Vec<Agent>,
}

#[derive(Debug, Default)]
struct Controller {
    state: Arc<Mutex<ControllerState>>,
}

impl Controller {
    fn new(state: Arc<Mutex<ControllerState>>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl ControllerService for Controller {
    async fn register(
        &self,
        request: Request<RegistrationRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        if request.agent_id.is_none() {
            return Err(Status::invalid_argument("agent_id is required"));
        }

        let mut state = self.state.lock().await;
        state.agents.push(Agent {
            agent_id: request.agent_id().to_owned(),
            hostname: request.hostname().to_owned(),
            version: request.version().to_owned(),
        });

        Ok(Response::new(()))
    }

    async fn message(&self, _request: Request<MessageRequest>) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}

async fn print_state(state_m: Arc<Mutex<ControllerState>>) -> Result<(), tonic::transport::Error> {
    loop {
        let state = state_m.lock().await;
        println!("Currently registered agents:");
        for agent in &state.agents {
            println!(
                "{agent_id} {hostname} {version}",
                agent_id = agent.agent_id,
                hostname = agent.hostname,
                version = agent.version
            );
        }
        drop(state);
        sleep(Duration::from_secs(10)).await;
    }
}

/// Main function
#[tokio::main]
pub async fn run(cli: Cli) -> Result<()> {
    let config = get_config(cli.config)?;

    let state = Arc::new(Mutex::new(ControllerState::default()));

    let controller = Controller::new(state.clone());

    let server = Server::builder()
        .add_service(ControllerServiceServer::new(controller))
        .serve(config.controller.listen);
    let state_printer = print_state(state.clone());

    let mut set = JoinSet::new();
    set.spawn(server);
    set.spawn(state_printer);
    set.join_all().await;

    Ok(())
}
