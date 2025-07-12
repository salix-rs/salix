//! Salix proto build script compiling Cap'n proto schemas

fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(&["proto/salix.proto"], &["proto/"])?;
    Ok(())
}
