#!/usr/bin/env bash
set -exuo pipefail
cd "$(dirname "$0")"

rm -rf gen
# Regenerate bindings after modifying `wit` folder
wit-bindgen-go generate --world root --out gen wit/
go mod tidy
