#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

cargo nextest run --workspace "$@"
node --test activity/db/turso-js/*.test.mjs
