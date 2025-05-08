#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

# Build the openai-js activity
(
cd activity/llm/openai-js
npm install
npm run build
)
(
cd workflow-js
npm install
npm run build
)
