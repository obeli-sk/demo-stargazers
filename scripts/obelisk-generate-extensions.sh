#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

generate() {
  local path="$1"
  local component_type="$2"

  find "$path" -maxdepth 1 -type d -exec test -d "{}/wit" \; -print | while read -r dir; do
    echo "Updating $dir"
    (
      cd "$dir" || exit
      obelisk generate extensions "$component_type" wit
    )
  done
}

generate "activity" "activity_wasm"
generate "workflow" "workflow"
