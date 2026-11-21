#!/usr/bin/env bash

# Generate obelisk_* folders inside wit/deps of the Rust components and the shared workflow interface.

set -exuo pipefail
cd "$(dirname "$0")/../.."

generate() {
  local dir="$1"
  local component_type="$2"
  (
    cd "$dir"
    rm -rf deps/obelisk_*
    obelisk generate wit-support "$component_type" deps
  )
}

# The activities are blocked by the process api dependency on wasi:io.
generate rust/workflow/wit "workflow"
generate rust/webhook/wit "webhook_endpoint"
generate rust/wit/workflow "workflow"
