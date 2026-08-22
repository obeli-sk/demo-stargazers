#!/usr/bin/env python3

import datetime
import itertools
import json
import sqlite3
import sys
from http.server import BaseHTTPRequestHandler, HTTPServer


def read_body(handler):
    content_length = handler.headers.get("Content-Length")
    if content_length is not None:
        return handler.rfile.read(int(content_length))
    if handler.headers.get("Transfer-Encoding", "").lower() != "chunked":
        return b""

    body = bytearray()
    while chunk_size := int(handler.rfile.readline().split(b";", 1)[0], 16):
        body.extend(handler.rfile.read(chunk_size))
        handler.rfile.read(2)
    while handler.rfile.readline().strip():
        pass
    return bytes(body)


# The activity generates `updated_at` client-side at millisecond resolution.
# Against real Turso, network latency spaces successive calls more than a
# millisecond apart, so writes always get strictly increasing timestamps and
# `ORDER BY updated_at` is unambiguous. This local mock answers sub-millisecond,
# so consecutive writes would tie and sort by rowid instead. Hand out a
# monotonic timestamp for the `now` argument to reproduce Turso's ordering.
_mock_clock = itertools.count()


def mock_now():
    tick = next(_mock_clock)
    stamp = datetime.datetime(2026, 1, 1, tzinfo=datetime.timezone.utc) + datetime.timedelta(
        milliseconds=tick
    )
    return stamp.strftime("%Y-%m-%dT%H:%M:%S.") + f"{stamp.microsecond // 1000:03d}Z"


def turso_value(value):
    if value is None:
        return {"type": "null"}
    if isinstance(value, int):
        return {"type": "integer", "value": str(value)}
    return {"type": "text", "value": str(value)}


class MockTursoHandler(BaseHTTPRequestHandler):
    database = sqlite3.connect(":memory:", check_same_thread=False)
    database.executescript(
        """
        CREATE TABLE users (name TEXT PRIMARY KEY, description TEXT, updated_at TEXT NOT NULL);
        CREATE TABLE repos (name TEXT PRIMARY KEY);
        CREATE TABLE stars (
            user_name TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            PRIMARY KEY (user_name, repo_name)
        );
        CREATE TABLE llm (id INTEGER PRIMARY KEY, settings TEXT NOT NULL);
        INSERT INTO llm VALUES (1, '{"model":"mock-model","messages":[],"max_tokens":50}');
        """
    )

    def do_POST(self):
        if self.path != "/v2/pipeline":
            self.send_error(404)
            return
        if self.headers.get("Transfer-Encoding", "").lower() == "chunked":
            self.send_json(411, {"error": "content-length required"})
            return

        request = json.loads(read_body(self))
        results = []
        try:
            for action in request["requests"]:
                if action["type"] == "close":
                    results.append({"type": "ok", "response": {"type": "close"}})
                    continue

                statement = action["stmt"]
                parameters = {
                    arg["name"]: arg["value"].get("value")
                    for arg in statement.get("named_args", [])
                }
                if "now" in parameters:
                    parameters["now"] = mock_now()
                cursor = self.database.execute(statement["sql"], parameters)
                rows = [[turso_value(value) for value in row] for row in cursor.fetchall()]
                columns = [
                    {"name": column[0], "decltype": None}
                    for column in (cursor.description or [])
                ]
                result = {
                    "cols": columns,
                    "rows": rows,
                    "affected_row_count": max(cursor.rowcount, 0),
                }
                results.append(
                    {"type": "ok", "response": {"type": "execute", "result": result}}
                )
            self.database.commit()
            self.send_json(200, {"results": results})
        except (KeyError, sqlite3.Error) as error:
            self.database.rollback()
            self.send_json(400, {"error": str(error)})

    def send_json(self, status, response):
        body = json.dumps(response).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, message, *args):
        print(f"[MockTurso] {message % args}")


def main():
    port = int(sys.argv[1])
    server = HTTPServer(("127.0.0.1", port), MockTursoHandler)
    print(f"Mock Turso server running on http://127.0.0.1:{port}", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
