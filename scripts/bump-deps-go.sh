#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

find . -name 'go.mod' | while read modfile; do
  dir=$(dirname "$modfile")
  echo "Updating deps in $dir"
  (
    cd "$dir"
    go get -u ./...
    go mod tidy
  )
done
