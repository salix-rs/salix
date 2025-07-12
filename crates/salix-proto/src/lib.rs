//! Salix protocol definition

pub const ALPN_QUIC_SALIX: &[&[u8]] = &[b"salix"];

include!(concat!(env!("OUT_DIR"), "/salix.rs"));
