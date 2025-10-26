#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

find . -name 'package.json' -not -path '*/node_modules/*' -not -path './.gopath/*' | while read modfile; do
  dir=$(dirname "$modfile")
  echo "Building $dir"
  (
    cd "$dir"
    npm install
    npm run build
  )
done
