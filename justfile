# List available commands
default:
  just --list

# Auto format code
fmt:
  cargo fmt
[private]
ci-lint-rustfmt:
  cargo fmt --check

# Lint code
lint:
  for crate in crates/*; do pushd ${crate} && cargo clippy && popd; done
[private]
ci-lint-clippy:
  RUSTFLAGS="-Dwarnings" just lint

# Lint and auto format
l: fmt lint

alias b := build
# Build all rust crates
build:
  for crate in crates/*; do pushd ${crate} && cargo build && popd; done
[private]
ci-build:
  RUSTFLAGS="-Dwarnings" just build

# Test salix crate
test-salix:
  cd crates/salix && cargo test

alias t := test
# Test all rust crates
test: test-salix
[private]
ci-test:
  RUSTFLAGS="-Dwarnings" just test

# Cleanup rust build directory
clean:
  rm -rf target
