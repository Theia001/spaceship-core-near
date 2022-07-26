#/bin/bash
VER=2021-11-01
rustup toolchain install stable-$VER
rustup default stable-$VER
rustup target add wasm32-unknown-unknown
cargo build -p token-tia --target wasm32-unknown-unknown --release
cargo build -p boxmall --target wasm32-unknown-unknown --release
cargo build -p riskerpool --target wasm32-unknown-unknown --release
cargo build -p rankpool --target wasm32-unknown-unknown --release
cargo build -p magicbox --target wasm32-unknown-unknown --release
cargo build -p shippool --target wasm32-unknown-unknown --release
cargo build -p spaceship --target wasm32-unknown-unknown --release