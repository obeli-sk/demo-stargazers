#!/usr/bin/env bash

# Collect versions of binaries installed by `nix develop` producing file `dev-deps.txt`.
# This script should be executed after every `nix flake update`.

set -exuo pipefail
cd "$(dirname "$0")/.."

rm -f dev-deps.txt
cargo upgrade --version >> dev-deps.txt
cargo nextest --version | sed -n '1p' >> dev-deps.txt
just --version >> dev-deps.txt
obelisk --version >> dev-deps.txt
rustc --version >> dev-deps.txt
wasmtime --version >> dev-deps.txt
# libc
ldd --version | sed -n '1p' >> dev-deps.txt
