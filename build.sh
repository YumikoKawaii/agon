#!/bin/sh
set -e
cd "$(dirname "$0")"
cargo build --release --target wasm32-unknown-unknown
rm -rf dist
mkdir -p dist
cp web/* dist/
cp target/wasm32-unknown-unknown/release/agon.wasm dist/
cp -R assets dist/assets
