#!/usr/bin/env bash
set -euo pipefail

sudo apt-get update
sudo apt-get install -y --no-install-recommends pkg-config libasound2-dev libpcsclite-dev

rustup toolchain install stable
rustup default stable
rustup component add rustfmt clippy

cargo build
cargo test --no-run
cargo build --release --bins --all-features
