#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

OBELISK_TOML="$1"
TRUNCATE="${2:-}"

# Make sure all components are fresh
cargo check --workspace

obelisk server verify --config $OBELISK_TOML
obelisk server run --config $OBELISK_TOML &
PID=$!
cleanup() {
    echo "Sending SIGINT to process $PID..."
    kill -SIGINT $PID

    # Wait up to 5 seconds for the process to exit
    SECONDS=0
    while kill -SIGINT $PID 2>/dev/null; do
        if [[ $SECONDS -ge 5 ]]; then
            echo "Cleanup timeout reached. Sending SIGKILL to process $PID..."
            kill -SIGKILL $PID
            break
        fi
        sleep 1
    done
}

trap cleanup EXIT

delete_from() {
    TABLE="$1"
    curl -X POST "https://${TURSO_LOCATION}/v2/pipeline" \
    -H "Authorization: Bearer ${TURSO_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{
    "requests": [
        { "type": "execute", "stmt": { "sql": "DELETE FROM '${TABLE}'" } },
        { "type": "close" }
    ]
    }'
}

# If TRUNCATE is set to "truncate", delete data
if [[ "$TRUNCATE" == "truncate" ]]; then
    delete_from "stars"
    delete_from "users"
    delete_from "repos"
fi

# Wait for obelisk to start responding
SECONDS=0
while ! obelisk client component list 2>/dev/null; do
    if [[ $SECONDS -ge 10 ]]; then
        echo "Timeout reached"
        exit 1
    fi
    sleep 1
done

obelisk client execution submit --follow stargazers:workflow/workflow.backfill '["obeli-sk/demo-stargazers"]'
JSON=$(curl "localhost:9090?repo=obeli-sk/demo-stargazers&ordering=asc&limit=1")
LOGIN=$(echo $JSON | jq .[0].login -r)
if [[ "$LOGIN" != "tomasol" ]]; then
    echo "Error: First stargazer should be 'tomasol', got '$LOGIN'" >&2
    exit 1
fi
echo "End to end test succeeded."
