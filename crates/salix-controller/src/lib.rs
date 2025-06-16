//! Salix
use std::{path::PathBuf, sync::Arc};

use anyhow::{Result, anyhow};
use clap::Parser;
use quinn_proto::crypto::rustls::QuicServerConfig;
use rustls::pki_types::PrivatePkcs8KeyDer;
use tracing::{error, info};

/// Our own lovely constant
pub const ALPN_QUIC_SALIX: &[&[u8]] = &[b"salix"];

/// CLI arguments
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

/// Main function
#[tokio::main]
pub async fn run(_cli: Cli) -> Result<()> {
    // let pwd = std::env::current_dir()?;
    // let cert_path = pwd.join("cert.der");
    // let key_path = pwd.join("key.der");
    let (certs, key) = {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
        let cert = cert.cert.into();
        (vec![cert], key.into())
    };

    rustls::crypto::CryptoProvider::install_default(rustls::crypto::aws_lc_rs::default_provider())
        .unwrap();

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    server_crypto.alpn_protocols = ALPN_QUIC_SALIX.iter().map(|&x| x.into()).collect();

    let mut server_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let endpoint = quinn::Endpoint::server(server_config, "[::1]:4433".parse()?)?;
    eprintln!("listening on {}", endpoint.local_addr()?);

    while let Some(conn) = endpoint.accept().await {
        let fut = handle_connection(conn);
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                error!("connection failed: {reason}", reason = e.to_string());
            }
        });
    }
    Ok(())
}

async fn handle_connection(conn: quinn::Incoming) -> Result<()> {
    let connection = conn.await?;
    loop {
        let stream = connection.accept_bi().await;
        let stream = match stream {
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => {
                return Ok(());
            }
            Err(e) => {
                return Err(e.into());
            }
            Ok(s) => s,
        };
        let fut = handle_request(stream);
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                eprintln!("failed: {reason}", reason = e.to_string());
            }
        });
    }
}

async fn handle_request(
    (mut send, mut recv): (quinn::SendStream, quinn::RecvStream),
) -> Result<()> {
    let req = recv
        .read_to_end(64 * 1024)
        .await
        .map_err(|e| anyhow!("faield request request: {}", e))?;

    let mut escaped = String::new();
    for &x in &req[..] {
        let part = std::ascii::escape_default(x).collect::<Vec<_>>();
        escaped.push_str(str::from_utf8(&part).unwrap());
    }

    info!(content = %escaped);

    send.write_all(escaped.as_bytes())
        .await
        .map_err(|e| anyhow!("failed to send response: {}", e))?;

    send.finish().unwrap();
    info!("complete");

    Ok(())
}
