#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

find . -path '*legacy-componentizejs/package.json' -not -path '*/node_modules/*' | while read modfile; do
  dir=$(dirname "$modfile")
  echo "Updating deps in $dir"
  (
    cd "$dir"
    npm update
  )
done
