#!/bin/sh

echo ">> Building contract"

rustup target add wasm32-unknown-unknown
cargo build --all --target wasm32-unknown-unknown --release
mkdir -p ./out
cp ./target/wasm32-unknown-unknown/release/*.wasm ./out
