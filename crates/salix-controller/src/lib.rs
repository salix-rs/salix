//! Salix controller
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use clap::Parser;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use salix_config::get_config;
use salix_proto::{
    MessageRequest, RegistrationRequest,
    controller_service_server::{ControllerService, ControllerServiceServer},
};
use tokio::{sync::Mutex, task::JoinSet, time::sleep};
use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;

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
    hostname: String,
    version: String,
}

type AgentMap = Arc<Mutex<HashMap<Uuid, Agent>>>;

#[derive(Debug, Default)]
struct State {
    agents: AgentMap,
}

impl State {
    fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn handle_connection(&self, conn: quinn::Connection) -> Result<()> {
        loop {
            let (send, recv) = conn.accept_bi().await?;
            self.handle_stream(send, recv).await?;
        }
    }

    async fn handle_stream(&self, mut send: quinn::SendStream, mut recv: RecvStream) -> Result<()> {
        Ok(())
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

async fn print_state(state_m: Arc<Mutex<ControllerState>>) -> Result<()> {
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

    let (certs, key) = {
        let key_path = config.controller.key_path.clone();
        let key =
            std::fs::read(&config.controller.key_path).context("Failed to read private key")?;
        let key = if key_path.extension().is_some_and(|x| x == "der") {
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key))
        } else {
            rustls_pemfile::private_key(&mut &*key)
                .context("malformed PKCS #1 private key")?
                .ok_or_else(|| anyhow::Error::msg("no private keys found"))?
        };

        let cert_path = config.controller.cert_path.clone();
        let cert_chain = std::fs::read(&cert_path).context("failed to read certificate chain")?;
        let cert_chain = if cert_path.extension().is_some_and(|x| x == "der") {
            vec![CertificateDer::from(cert_chain)]
        } else {
            rustls_pemfile::certs(&mut &*cert_chain)
                .collect::<Result<_, _>>()
                .context("invalid PEM-encoded certificate")?
        };

        (cert_chain, key)
    };

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    server_crypto.alpn_protocols = ALPN_QUIC_SALIX.iter().map(|&x| x.into()).collect();

    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(
        quinn_proto::crypto::rustls::QuicServerConfig::try_from(server_crypto)?,
    ));
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let endpoint = quinn::Endpoint::server(server_config, config.controller.listen)?;
    let server = handle_new_connection(state.clone(), endpoint);

    let state_printer = print_state(state.clone());

    let mut set = JoinSet::new();
    set.spawn(server);
    set.spawn(state_printer);
    set.join_all().await;

    Ok(())
}

async fn handle_new_connection(
    state: Arc<Mutex<ControllerState>>,
    endpoint: quinn::Endpoint,
) -> Result<()> {
    while let Some(conn) = endpoint.accept().await {
        let state_clone = state.clone();
        tokio::spawn(handle_connection(state_clone, conn));
    }
    Ok(())
}

async fn handle_connection(
    state: Arc<Mutex<ControllerState>>,
    conn: quinn::Incoming,
) -> Result<()> {
    let connection = conn.await?;
    loop {
        let stream = connection.accept_bi().await;
        let stream = match stream {
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                return Ok(());
            }
            Err(e) => {
                return Err(e)?;
            }
            Ok(s) => s,
        };

        let fut = handle_request(state.clone(), stream);
        tokio::spawn(fut);
    }
    Ok(())
}

async fn handle_request(
    state: Arc<Mutex<ControllerState>>,
    (mut send, mut recv): (quinn::SendStream, quinn::RecvStream),
) -> Result<()> {
    Ok(())
}
