#!/usr/bin/env bash

# Usage: test-e2e.sh path-to-obelisk.toml
# This script starts Obelisk,
# sends a "star-added" HTTP request to the webhook endpoint,
# waits for the scheduled execution to complete,
# and verifies that the user is stored in the database along with the generated description.
#
# GitHub and Turso are exercised against the real services, so the following
# credentials must be set in the environment:
#   TURSO_TOKEN, TURSO_LOCATION
#   GITHUB_TOKEN_STARGAZERS, TEST_GITHUB_LOGIN
# OpenAI is the only paid dependency, so it is always mocked.

set -exo pipefail
cd "$(dirname "$0")/.."

OBELISK_TOML="$1"
STAR_ACCOUNT="someghaccount"
STAR_REPO="someghrepo"
MOCK_OPENAI_PORT=18080
MOCK_OPENAI_PID=""
PID=""
SERVER_TOML="$(mktemp)"
cp ./server.toml "$SERVER_TOML"

export OBELISK_API_TOKEN=$(obelisk generate token --json | python3 -c 'import json, sys; print(json.load(sys.stdin)["token"])')
export GITHUB_WEBHOOK_SECRET="It's a Secret to Everybody"

for var in TURSO_TOKEN TURSO_LOCATION GITHUB_TOKEN_STARGAZERS TEST_GITHUB_LOGIN; do
    if [[ -z "${!var:-}" ]]; then
        echo "Error: $var must be set; end-to-end tests run against the real service" >&2
        exit 1
    fi
done

# OpenAI is the only paid service, so always point it at the local mock.
export OPENAI_API_KEY="mock-api-key-for-testing"
export OPENAI_API_BASE_URL="http://127.0.0.1:$MOCK_OPENAI_PORT"

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

    if [[ -n "$MOCK_OPENAI_PID" ]]; then
        kill "$MOCK_OPENAI_PID" 2>/dev/null || true
    fi

    rm -f "$SERVER_TOML"
}

trap cleanup EXIT

python3 ./scripts/mock-openai-server.py "$MOCK_OPENAI_PORT" &
MOCK_OPENAI_PID=$!

SECONDS=0
until curl -sf -X POST "$OPENAI_API_BASE_URL/v1/chat/completions" -d '{}' >/dev/null; do
    if [[ $SECONDS -ge 5 ]]; then
        echo "Mock OpenAI server failed to start"
        exit 1
    fi
    sleep 0.5
done

for table in stars users repos; do
    curl --fail -X POST "https://${TURSO_LOCATION}/v2/pipeline" \
        -H "Authorization: Bearer ${TURSO_TOKEN}" \
        -H "Content-Type: application/json" \
        -d '{"requests":[{"type":"execute","stmt":{"sql":"DELETE FROM '${table}'"}},{"type":"close"}]}'
done

obelisk deployment verify --fix --server-config "$SERVER_TOML" --deployment "$OBELISK_TOML"
obelisk server run --server-config "$SERVER_TOML" --deployment "$OBELISK_TOML" &
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
