#!/bin/bash
set -e
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
cargo build --all --target wasm32-unknown-unknown --release