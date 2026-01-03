#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

generate() {
  local path="$1"
  local component_type="$2"

  if [ ! -d "$path" ]; then
    return 0
  fi
  find "$path" -maxdepth 1 -type d -exec test -d "{}/wit" \; -print | while read -r dir; do
    echo "Updating $dir"
    (
      cd "$dir/wit"
      obelisk generate extensions "$component_type" . gen
    )
  done
}

generate "activity" "activity_wasm"
generate "activity-stub" "activity_stub"
generate "workflow" "workflow"
