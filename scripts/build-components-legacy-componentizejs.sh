#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

find . -path '*legacy-componentizejs/package.json' -not -path '*/node_modules/*' | while read modfile; do
  dir=$(dirname "$modfile")
  echo "Building $dir"
  (
    cd "$dir"
    npm install
    npm run build
  )
done
