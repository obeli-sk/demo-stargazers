#!/usr/bin/env bash

set -exuo pipefail
cd "$(dirname "$0")/.."

OBELISK_TOML="$1"
TRUNCATE="${2:-}"
STAR_ACCOUNT="someghaccount"
STAR_REPO="someghrepo"

export GITHUB_WEBHOOK_SECRET="It's a Secret to Everybody"

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

delete_from_all() {
    delete_from "stars"
    delete_from "users"
    delete_from "repos"
}

# If TRUNCATE is set to "truncate", delete data
if [[ "$TRUNCATE" == "truncate" ]]; then
    delete_from_all
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
EXECUTION_ID=$(curl -X POST http://127.0.0.1:9090 \
-H "X-Hub-Signature-256:$SIGNATURE" \
-d "$PAYLOAD" -i | grep -i "execution-id" | cut -d ' ' -f2- | tr -d '\r')

# Wait until the scheduled execution of the workflow finishes.
obelisk client execution get --follow $EXECUTION_ID

# Get the first and only user back from the database.
JSON=$(curl "localhost:9090?repo=${STAR_ACCOUNT}/${STAR_REPO}&ordering=asc&limit=1")
LOGIN=$(echo $JSON | jq .[0].login -r)
if [[ "$LOGIN" != ${TEST_GITHUB_LOGIN} ]]; then
    echo "Error: First stargazer should be '${TEST_GITHUB_LOGIN}', got '$LOGIN'" >&2
    exit 1
fi
DESCRIPTION=$(echo $JSON | jq .[0].description -r)

if [ "$DESCRIPTION" == "null" ]; then
  echo "Error: description is null" >&2
  exit 1
else
  echo "Description: $DESCRIPTION"
fi

echo "End to end test succeeded."
