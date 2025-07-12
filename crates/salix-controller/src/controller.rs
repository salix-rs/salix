use prost::Message;
use quinn::Endpoint;
use std::{collections::HashMap, io::Cursor, sync::Arc};

use anyhow::anyhow;
use tokio::sync::Mutex;
use uuid::Uuid;

use anyhow::{Context, Result};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use salix_config::Config;
use salix_proto::ALPN_QUIC_SALIX;

#[derive(Debug)]
struct Agent {
    hostname: String,
    version: String,
}

type AgentMap = Arc<Mutex<HashMap<Uuid, Agent>>>;

#[derive(Debug, Clone)]
pub(crate) struct Controller {
    agents: AgentMap,
}

impl Controller {
    pub(crate) async fn make_endpoint(config: &Config) -> Result<Endpoint> {
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
            let cert_chain =
                std::fs::read(&cert_path).context("failed to read certificate chain")?;
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

        Ok(endpoint)
    }

    pub(crate) fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn handle_connection(&self, conn: quinn::Connection) -> Result<()> {
        loop {
            match conn.accept_bi().await {
                Ok((send, recv)) => match self.handle_stream(send, recv).await {
                    Ok(should_continue) => {
                        if !should_continue {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Stream error: {e}");
                        break;
                    }
                },
                Err(e) => {
                    eprintln!("Connection error {e}");
                    break;
                }
            }
        }

        eprintln!("Connection closed");
        Ok(())
    }

    async fn handle_stream(
        &self,
        mut send: quinn::SendStream,
        mut recv: quinn::RecvStream,
    ) -> Result<bool> {
        let request_message = self.read_protobuf_message(&mut recv).await?;

        let response_message: Option<salix_proto::Message> = match request_message.message {
            Some(salix_proto::message::Message::RegistrationRequest(_req)) => None,
            _ => None,
        };

        if response_message.is_some() {
            self.write_protobuf_message(&mut send, response_message.unwrap())
                .await?;
        }
        send.finish()?;
        Ok(true)
    }

    async fn read_protobuf_message(
        &self,
        stream: &mut quinn::RecvStream,
    ) -> Result<salix_proto::Message> {
        let mut len_bytes = [0_u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        if len > 10_000_000 {
            return Err(anyhow!("Message too large"));
        }

        let mut buf = vec![0_u8; len];
        stream.read_exact(&mut buf).await?;

        let message = salix_proto::Message::decode(&mut Cursor::new(buf))?;
        Ok(message)
    }

    async fn write_protobuf_message(
        &self,
        stream: &mut quinn::SendStream,
        message: salix_proto::Message,
    ) -> Result<()> {
        let mut buf = Vec::new();
        message.encode(&mut buf)?;

        let len = buf.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&buf).await?;
        Ok(())
    }

    pub(crate) async fn run(&self, endpoint: Endpoint) -> Result<()> {
        while let Some(conn) = endpoint.accept().await {
            let server = self.clone();
            tokio::spawn(async move {
                match conn.await {
                    Ok(connection) => {
                        if let Err(e) = server.handle_connection(connection).await {
                            eprintln!("Connection error: {e}")
                        }
                    }
                    Err(e) => eprintln!("Connection failed: {e}"),
                }
            });
        }
        Ok(())
    }
}
