#!/usr/bin/env bash
set -e
wasm-pack build --target web
cargo run --manifest-path ../runlicense-sdk-rust/Cargo.toml --bin generate_manifest -- "$(pwd)/pkg/gameboy_bg.wasm"
echo "Done — pkg/ is ready"
