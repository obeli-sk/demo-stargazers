#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

(
cd activity/llm/openai-go
./build.sh
)
