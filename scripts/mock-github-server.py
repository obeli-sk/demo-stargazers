#!/usr/bin/env python3

import json
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


class MockGitHubHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path != "/graphql":
            self.send_error(404)
            return

        request = json.loads(read_body(self))
        variables = request.get("variables", {})
        if "repo" in variables:
            response = {
                "data": {
                    "resource": {
                        "__typename": "Repository",
                        "stargazers": {"nodes": [], "edges": []},
                    }
                }
            }
        else:
            login = variables.get("login", "test-stargazer")
            response = {
                "data": {
                    "user": {
                        "login": login,
                        "organizations": {"nodes": []},
                        "topRepositories": {"nodes": []},
                    }
                }
            }
        body = json.dumps(response).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, message, *args):
        print(f"[MockGitHub] {message % args}")


def main():
    port = int(sys.argv[1])
    server = HTTPServer(("127.0.0.1", port), MockGitHubHandler)
    print(f"Mock GitHub server running on http://127.0.0.1:{port}", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
