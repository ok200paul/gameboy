#!/usr/bin/env bash
set -e
wasm-pack build --target web
cargo run --manifest-path ../runlicense/sdk-webassembly-rust/Cargo.toml --features tools --bin generate_manifest -- "$(pwd)/pkg/gameboy_bg.wasm" --src "$(pwd)/src" --package "gameboy:emulator"
echo "Done — pkg/ is ready"
