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

# Usage: create_schema URL [LLM_SETTINGS_JSON]. Optionally inserts the LLM settings row.
create_schema() {
    python3 -c '
import json, sys
requests = [{"type": "sequence", "sql": open(sys.argv[1]).read()}]
if len(sys.argv) > 2:
    requests.append({"type": "execute", "stmt": {"sql": "INSERT INTO llm (id, settings) VALUES (1, :settings)",
        "named_args": [{"name": "settings", "value": {"type": "text", "value": sys.argv[2]}}]}})
print(json.dumps({"requests": requests + [{"type": "close"}]}))' activity/db/turso/ddl/schema.sql "${@:2}" \
        | curl --fail -sS -X POST "$1/v2/pipeline" -H "Content-Type: application/json" --data @- \
        | python3 -c 'import json, sys; errors = [r["error"] for r in json.load(sys.stdin)["results"] if r["type"] != "ok"]; sys.exit(f"cannot set up the database: {errors}" if errors else 0)'
}
