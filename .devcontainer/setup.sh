#!/bin/bash
set -e
apt update
apt install -y cmake ninja-build
rustup install nightly # Required for wasm/c interop
rustup +nightly target add wasm32-unknown-unknown

curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
cargo binstall -y wasm-bindgen-cli wasm-pack cargo-make cargo-nextest simple-http-server

cargo make setup-wasm
