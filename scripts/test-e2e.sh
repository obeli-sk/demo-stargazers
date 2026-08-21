#!/usr/bin/env bash

# Usage: test-e2e.sh path-to-obelisk.toml
# This script starts Obelisk,
# sends a "star-added" HTTP request to the webhook endpoint,
# waits for the scheduled execution to complete,
# and verifies that the user is stored in the database along with the generated description.

set -exo pipefail
cd "$(dirname "$0")/.."

OBELISK_TOML="$1"
STAR_ACCOUNT="someghaccount"
STAR_REPO="someghrepo"
MOCK_OPENAI_PORT=18080
MOCK_TURSO_PORT=18081
MOCK_GITHUB_PORT=18082
MOCK_OPENAI_PID=""
MOCK_TURSO_PID=""
MOCK_GITHUB_PID=""
PID=""
TEST_DEPLOYMENT=""
MOCK_OPENAI_API_BASE_URL="http://127.0.0.1:$MOCK_OPENAI_PORT"
MOCK_TURSO_LOCATION="http://127.0.0.1:$MOCK_TURSO_PORT"
MOCK_GITHUB_API_BASE_URL="http://127.0.0.1:$MOCK_GITHUB_PORT"

export OBELISK__API__TOKEN=$(obelisk generate token --json | python3 -c 'import json, sys; print(json.load(sys.stdin)["token"])')
export GITHUB_WEBHOOK_SECRET="It's a Secret to Everybody"
USE_MOCK_OPENAI=false
USE_MOCK_TURSO=false
USE_MOCK_GITHUB=false

if [[ -z "${OPENAI_API_KEY:-}" ]]; then
    USE_MOCK_OPENAI=true
    export OPENAI_API_KEY="mock-api-key-for-testing"
    export OPENAI_API_BASE_URL="$MOCK_OPENAI_API_BASE_URL"
fi
if [[ -z "${TURSO_TOKEN:-}" || -z "${TURSO_LOCATION:-}" ]]; then
    USE_MOCK_TURSO=true
    export TURSO_TOKEN="mock-token-for-testing"
    export TURSO_LOCATION="$MOCK_TURSO_LOCATION"
fi
if [[ -z "${GITHUB_TOKEN_STARGAZERS:-}" || -z "${TEST_GITHUB_LOGIN:-}" ]]; then
    USE_MOCK_GITHUB=true
    export GITHUB_TOKEN_STARGAZERS="mock-token-for-testing"
    export GITHUB_API_BASE_URL="$MOCK_GITHUB_API_BASE_URL"
    export TEST_GITHUB_LOGIN="test-stargazer"
fi

cleanup() {
    if [[ -n "$PID" ]]; then
        echo "Sending SIGINT to obelisk process $PID..."
        kill -SIGINT "$PID" 2>/dev/null || true
    fi

    # Wait up to 5 seconds for the process to exit
    SECONDS=0
    while [[ -n "$PID" ]] && kill -0 "$PID" 2>/dev/null; do
        if [[ $SECONDS -ge 5 ]]; then
            echo "Cleanup timeout reached. Sending SIGKILL to process $PID..."
            kill -SIGKILL "$PID" 2>/dev/null || true
            break
        fi
        sleep 1
    done

    for mock_pid in "$MOCK_OPENAI_PID" "$MOCK_TURSO_PID" "$MOCK_GITHUB_PID"; do
        if [[ -n "$mock_pid" ]]; then
            kill "$mock_pid" 2>/dev/null || true
        fi
    done
    if [[ -n "$TEST_DEPLOYMENT" ]]; then
        rm -f "$TEST_DEPLOYMENT"
    fi
}

trap cleanup EXIT

if $USE_MOCK_OPENAI; then
    python3 ./scripts/mock-openai-server.py "$MOCK_OPENAI_PORT" &
    MOCK_OPENAI_PID=$!
fi
if $USE_MOCK_TURSO; then
    python3 ./scripts/mock-turso-server.py "$MOCK_TURSO_PORT" &
    MOCK_TURSO_PID=$!
fi
if $USE_MOCK_GITHUB; then
    python3 ./scripts/mock-github-server.py "$MOCK_GITHUB_PORT" &
    MOCK_GITHUB_PID=$!
fi

SECONDS=0
until { ! $USE_MOCK_OPENAI || curl -sf -X POST "$MOCK_OPENAI_API_BASE_URL/v1/chat/completions" -d '{}' >/dev/null; } \
    && { ! $USE_MOCK_TURSO || curl -sf -X POST "$MOCK_TURSO_LOCATION/v2/pipeline" \
        -H 'Content-Type: application/json' -d '{"requests":[{"type":"close"}]}' >/dev/null; } \
    && { ! $USE_MOCK_GITHUB || curl -sf -X POST "$MOCK_GITHUB_API_BASE_URL/graphql" \
        -H 'Content-Type: application/json' -d '{}' >/dev/null; }; do
    if [[ $SECONDS -ge 5 ]]; then
        echo "Mock services failed to start"
        exit 1
    fi
    sleep 0.5
done

if { $USE_MOCK_TURSO || $USE_MOCK_GITHUB; } \
    && [[ "$OBELISK_TOML" == "obelisk-oci.toml" || "$OBELISK_TOML" == "./obelisk-oci.toml" ]]; then
    TEST_DEPLOYMENT=$(mktemp ./obelisk-oci-e2e-XXXXXX.toml)
    sed \
        -e 's|oci://docker.io/getobelisk/demo_stargazers_activity_github_impl:[^" ]*|target/wasm32-wasip2/release/activity_github_impl.wasm|' \
        -e 's|oci://docker.io/getobelisk/demo_stargazers_activity_db_turso:[^" ]*|target/wasm32-wasip2/release/activity_db_turso.wasm|' \
        "$OBELISK_TOML" > "$TEST_DEPLOYMENT"
    OBELISK_TOML="$TEST_DEPLOYMENT"
fi

if ! $USE_MOCK_TURSO; then
    for table in stars users repos; do
        curl --fail -X POST "https://${TURSO_LOCATION}/v2/pipeline" \
            -H "Authorization: Bearer ${TURSO_TOKEN}" \
            -H "Content-Type: application/json" \
            -d '{"requests":[{"type":"execute","stmt":{"sql":"DELETE FROM '${table}'"}},{"type":"close"}]}'
    done
fi

obelisk deployment verify --server-config ./server.toml --deployment "$OBELISK_TOML"
obelisk server run --server-config ./server.toml --deployment "$OBELISK_TOML" &
PID=$!

# Wait for obelisk to start responding
SECONDS=0
while ! obelisk component list 2>/dev/null; do
    if [[ $SECONDS -ge 10 ]]; then
        echo "Timeout reached"
        exit 1
    fi
    sleep 1
done

dump_executions() {
    while read -r execution_id _; do
        obelisk execution get "$execution_id" || true
        obelisk execution logs --level trace "$execution_id" || true
    done < <(obelisk execution list --show-derived)
}

# Make sure this repo has no stars
if ! JSON=$(curl --fail "http://127.0.0.1:9090?repo=${STAR_ACCOUNT}/${STAR_REPO}&ordering=asc&limit=1"); then
    dump_executions
    exit 1
fi
if [[ "$JSON" != "[]" ]]; then
    echo "The repo ${STAR_ACCOUNT}/${STAR_REPO} already has star gazers"
    exit 1
fi

PAYLOAD='{
    "action": "created",
    "sender": {
        "login": "'${TEST_GITHUB_LOGIN}'"
    },
    "repository": {
        "owner": {
            "login": "'${STAR_ACCOUNT}'"
        },
        "name": "'${STAR_REPO}'"
    }
}'

SIGNATURE=$(echo -n "$PAYLOAD" | openssl dgst -sha256 -hmac "$GITHUB_WEBHOOK_SECRET" | cut -d ' ' -f2)
SIGNATURE="sha256=$SIGNATURE"

# Send the webhook event
EXECUTION_ID=$(curl --fail -X POST http://127.0.0.1:9090 \
-H "X-Hub-Signature-256:$SIGNATURE" \
-d "$PAYLOAD" -i | grep -i "execution-id" | cut -d ' ' -f2- | tr -d '\r')

# Wait until the scheduled execution of the workflow finishes.
obelisk execution get --follow $EXECUTION_ID

# Get the first and only user back from the database.
if ! JSON=$(curl --fail "http://127.0.0.1:9090?repo=${STAR_ACCOUNT}/${STAR_REPO}&ordering=asc&limit=1"); then
    dump_executions
    exit 1
fi
LOGIN=$(python3 -c 'import json, sys; print(json.load(sys.stdin)[0]["login"])' <<< "$JSON")
if [[ "$LOGIN" != ${TEST_GITHUB_LOGIN} ]]; then
    echo "Error: First stargazer should be '${TEST_GITHUB_LOGIN}', got '$LOGIN'" >&2
    exit 1
fi
DESCRIPTION=$(python3 -c 'import json, sys; value = json.load(sys.stdin)[0]["description"]; print("null" if value is None else value)' <<< "$JSON")

if [ "$DESCRIPTION" == "null" ]; then
  echo "Error: description is null" >&2
  exit 1
else
  echo "Description: $DESCRIPTION"
fi

echo "End to end test succeeded."
