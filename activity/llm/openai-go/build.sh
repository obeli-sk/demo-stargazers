#!/usr/bin/env bash
set -exuo pipefail
cd "$(dirname "$0")"

tinygo build -target=wasip2 -o dist/openai-go.wasm --wit-package wit/ --wit-world root main.go
