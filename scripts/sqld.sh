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
