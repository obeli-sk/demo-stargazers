#!/usr/bin/env bash

# Collect versions of binaries installed by `nix develop` producing file `dev-deps.txt`.
# This script should be executed after every `nix flake update`.

set -exuo pipefail
cd "$(dirname "$0")/.."

rm -f dev-deps.txt
echo "cargo-binstall $(cargo-binstall -V)" >> dev-deps.txt
cargo upgrade --version >> dev-deps.txt
cargo-expand --version >> dev-deps.txt
cargo-generate --version >> dev-deps.txt
cargo nextest --version | head -n 1 >> dev-deps.txt
just --version >> dev-deps.txt
obelisk --version >> dev-deps.txt
echo "pkg-config $(pkg-config --version)" >> dev-deps.txt
rustc --version >> dev-deps.txt
wasm-tools --version >> dev-deps.txt
wasmtime --version >> dev-deps.txt
# JavaScript
echo "node.js $(node --version)" >> dev-deps.txt
wizer --version >> dev-deps.txt
# Go
tinygo version >> dev-deps.txt
echo "wit-bindgen-go-cli 0.7.0" >> dev-deps.txt
# libc
ldd --version | head -n 1 >> dev-deps.txt
