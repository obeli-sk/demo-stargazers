#!/usr/bin/env bash

# Generate obelisk_* folders inside impls' wit/deps

set -exuo pipefail
cd "$(dirname "$0")/.."

generate() {
  local path="$1"
  local component_type="$2"

  if [ ! -d "$path" ]; then
    return 0
  fi
  find "$path" -maxdepth 2 -type d -exec test -d "{}/wit" \; -print | while read -r dir; do
    echo "Updating $dir"
    (
      cd "$dir/wit"
      rm -rf deps/obelisk_*
      obelisk generate wit-support "$component_type" deps
    )
  done
}

generate_interface() {
  local path="$1"
  local component_type="$2"

  if [ ! -d "$path" ]; then
    return 0
  fi
  (
    cd "$path"
    rm -rf deps/obelisk_*
    obelisk generate wit-support "$component_type" deps
  )
}

# generate "activity" "activity_wasm" # blocked by process api dependency on wasi:io
generate "activity-stub" "activity_stub"
generate "workflow" "workflow"
generate_interface "workflow/stargazers/wit-interface" "workflow"
generate "webhook" "webhook_endpoint"
