#!/usr/bin/env bash

# Generate obelisk extension packages (gen/) of the shared WIT interfaces.

set -exuo pipefail
cd "$(dirname "$0")/../wit"

for dir in db github llm; do
  (cd "$dir" && obelisk generate wit-extensions activity . gen)
done
(cd workflow && obelisk generate wit-extensions workflow . gen)
