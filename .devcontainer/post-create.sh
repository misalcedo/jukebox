#!/usr/bin/env bash
set -euo pipefail

sudo apt-get update
sudo apt-get install -y --no-install-recommends pkg-config libasound2-dev libpcsclite-dev

RUST_VERSION="$(sed -nE 's/^rust-version = "([^"]+)"/\1/p' Cargo.toml)"
rustup toolchain install "$RUST_VERSION"
rustup default "$RUST_VERSION"
rustup component add rustfmt clippy

cargo build
cargo test --no-run
cargo build --release --bins --all-features
