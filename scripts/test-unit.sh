#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

cargo nextest run --workspace "$@"
