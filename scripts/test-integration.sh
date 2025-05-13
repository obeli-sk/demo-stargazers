#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

(
cd activity/github/impl
cargo nextest run -- --ignored
)
(
cd activity/db/turso
cargo nextest run --test-threads=1 -- --ignored
)
(
cd activity/llm/openai
cargo nextest run -- --ignored
)
