#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

(
cd activity/account/github
cargo nextest run -- --ignored
)
(
cd activity/db/turso
cargo nextest run --test-threads=1 -- --ignored
)
(
cd activity/llm/chatgpt
cargo nextest run -- --ignored
)
