#!/usr/bin/env bash
set -exuo pipefail
cd "$(dirname "$0")"

module_name=$(grep '^module ' go.mod | awk '{print $2}')
component_name=$(basename "$module_name")
tinygo build -target=wasip2 -o dist/$component_name.wasm --wit-package wit/ --wit-world root main.go
