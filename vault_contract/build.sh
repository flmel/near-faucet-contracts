rm -rf out/
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
mkdir out
cp target/wasm32-unknown-unknown/release/near_testnet_faucet_vault.wasm out/main.wasm