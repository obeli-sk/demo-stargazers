#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

MOCK_OPENAI_PORT=18080

# Start mock OpenAI server
python3 ./scripts/mock-openai-server.py $MOCK_OPENAI_PORT &
MOCK_PID=$!
echo "Started mock OpenAI server with PID $MOCK_PID"

cleanup() {
    echo "Stopping mock OpenAI server (PID $MOCK_PID)..."
    kill $MOCK_PID 2>/dev/null || true
}
trap cleanup EXIT

# Wait for mock server to be ready
SECONDS=0
while ! curl -s http://127.0.0.1:$MOCK_OPENAI_PORT/v1/chat/completions -X POST -d '{}' > /dev/null 2>&1; do
    if [[ $SECONDS -ge 5 ]]; then
        echo "Mock OpenAI server failed to start"
        exit 1
    fi
    sleep 0.5
done
echo "Mock OpenAI server is ready"

# Set environment variables for mock OpenAI
export TEST_OPENAI_API_KEY="mock-api-key-for-testing"
export TEST_OPENAI_API_BASE_URL="http://127.0.0.1:$MOCK_OPENAI_PORT"

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
