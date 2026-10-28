# Sourced by the test scripts: a local libSQL server (sqld), the server behind Turso.

# Usage: start_sqld PORT DIR. Keeps the database and log in DIR and sets SQLD_PID.
start_sqld() {
    sqld --db-path "$2/data.sqld" --http-listen-addr "127.0.0.1:$1" > "$2/sqld.log" 2>&1 &
    SQLD_PID=$!
    SECONDS=0
    until curl -sf "http://127.0.0.1:$1/health" >/dev/null; do
        if [[ $SECONDS -ge 30 ]]; then
            echo "sqld failed to start"
            cat "$2/sqld.log"
            exit 1
        fi
        sleep 0.5
    done
}

# Usage: wait_for_migrations URL. Waits until every migration file is in the history table.
wait_for_migrations() {
    local expected
    expected=$(find activity/db/migrations -name 'V*.sql' | wc -l)
    SECONDS=0
    until [[ "$(sqld_query "$1" 'SELECT count(*) FROM refinery_schema_history' 2>/dev/null)" == "$expected" ]]; do
        if [[ $SECONDS -ge 30 ]]; then
            echo "migrations did not finish"
            return 1
        fi
        sleep 0.5
    done
}

# Usage: sqld_query URL SQL [TEXT_ARG...]. Prints the first cell, args bind to ?1, ?2...
sqld_query() {
    python3 -c '
import json, sys
args = [{"type": "text", "value": arg} for arg in sys.argv[2:]]
print(json.dumps({"requests": [{"type": "execute", "stmt": {"sql": sys.argv[1], "args": args}}, {"type": "close"}]}))' "${@:2}" \
        | curl --fail -sS -X POST "$1/v2/pipeline" -H "Content-Type: application/json" --data @- \
        | python3 -c '
import json, sys
result = json.load(sys.stdin)["results"][0]
if result["type"] != "ok":
    sys.exit("query failed: " + json.dumps(result["error"]))
rows = result["response"]["result"]["rows"]
print(rows[0][0].get("value", "") if rows else "")'
}
