#!/bin/bash
set -e
RUSTFLAGS="-C link-args=-s" cargo build --target wasm32-unknown-unknown --release
cargo build --all --target wasm32-unknown-unknown --release

cp target/wasm32-unknown-unknown/release/ft_contract.wasm ./res/