#!/usr/bin/env python3
"""Mock OpenAI API server for testing.

Returns a canned response for chat completions requests.
"""

import json
import sys
from http.server import HTTPServer, BaseHTTPRequestHandler

DEFAULT_PORT = 18080


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


class MockOpenAIHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        if self.path == "/v1/chat/completions":
            body = read_body(self)
            
            request_data = {}
            user_prompt = 'unknown'
            try:
                request_data = json.loads(body)
                # Extract user message for a more contextual mock response
                user_messages = [m for m in request_data.get('messages', []) if m.get('role') == 'user']
                user_prompt = user_messages[-1]['content'] if user_messages else 'unknown'
            except (json.JSONDecodeError, KeyError, IndexError):
                pass
            
            # Return a mock response
            response = {
                "id": "chatcmpl-mock-12345",
                "object": "chat.completion",
                "created": 1234567890,
                "model": request_data.get('model', 'gpt-3.5-turbo'),
                "choices": [
                    {
                        "index": 0,
                        "message": {
                            "role": "assistant",
                            "content": f"This is a mock response for testing. User asked about: {user_prompt[:100]}"
                        },
                        "finish_reason": "stop"
                    }
                ],
                "usage": {
                    "prompt_tokens": 10,
                    "completion_tokens": 20,
                    "total_tokens": 30
                }
            }
            
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Length', str(len(json.dumps(response).encode())))
            self.end_headers()
            self.wfile.write(json.dumps(response).encode())
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        print(f"[MockOpenAI] {args[0]}")

def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_PORT
    server = HTTPServer(('127.0.0.1', port), MockOpenAIHandler)
    print(f"Mock OpenAI server running on http://127.0.0.1:{port}")
    print(f"Use OPENAI_API_BASE_URL=http://127.0.0.1:{port} for testing")
    sys.stdout.flush()
    server.serve_forever()

if __name__ == '__main__':
    main()
