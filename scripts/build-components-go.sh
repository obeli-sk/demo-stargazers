#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."


find . -name 'go.mod' | while read modfile; do
  dir=$(dirname "$modfile")
  echo "Building $dir"
  (
    cd "$dir"
    ./build.sh
  )
done
