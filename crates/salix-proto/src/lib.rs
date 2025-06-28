//! Salix protocol definition

pub const ALPN_QUIC_SALIX: &[&[u8]] = &[b"salix"];

tonic::include_proto!("salix");
