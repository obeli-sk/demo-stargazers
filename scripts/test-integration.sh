#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

MOCK_OPENAI_PORT=18080
MOCK_TURSO_PORT=18081
MOCK_GITHUB_PORT=18082
MOCK_OPENAI_PID=""
MOCK_TURSO_PID=""
MOCK_GITHUB_PID=""

cleanup() {
    for mock_pid in "$MOCK_OPENAI_PID" "$MOCK_TURSO_PID" "$MOCK_GITHUB_PID"; do
        if [[ -n "$mock_pid" ]]; then
            kill "$mock_pid" 2>/dev/null || true
        fi
    done
}
trap cleanup EXIT

export TEST_OPENAI_API_KEY="mock-api-key-for-testing"
export TEST_OPENAI_API_BASE_URL="http://127.0.0.1:$MOCK_OPENAI_PORT"
export TEST_TURSO_TOKEN="mock-token-for-testing"
export TEST_TURSO_LOCATION="http://127.0.0.1:$MOCK_TURSO_PORT"
export TEST_GITHUB_TOKEN_STARGAZERS="mock-token-for-testing"
export TEST_GITHUB_API_BASE_URL="http://127.0.0.1:$MOCK_GITHUB_PORT"
export TEST_GITHUB_LOGIN="test-stargazer"
export TEST_GITHUB_REPO="someghaccount/someghrepo"

python3 ./scripts/mock-openai-server.py "$MOCK_OPENAI_PORT" &
MOCK_OPENAI_PID=$!
python3 ./scripts/mock-turso-server.py "$MOCK_TURSO_PORT" &
MOCK_TURSO_PID=$!
python3 ./scripts/mock-github-server.py "$MOCK_GITHUB_PORT" &
MOCK_GITHUB_PID=$!

SECONDS=0
until curl -sf -X POST "$TEST_OPENAI_API_BASE_URL/v1/chat/completions" -d '{}' >/dev/null \
    && curl -sf -X POST "$TEST_TURSO_LOCATION/v2/pipeline" \
        -H 'Content-Type: application/json' -d '{"requests":[{"type":"close"}]}' >/dev/null \
    && curl -sf -X POST "$TEST_GITHUB_API_BASE_URL/graphql" \
        -H 'Content-Type: application/json' -d '{}' >/dev/null; do
    if [[ $SECONDS -ge 5 ]]; then
        echo "Mock services failed to start"
        exit 1
    fi
    sleep 0.5
done

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
