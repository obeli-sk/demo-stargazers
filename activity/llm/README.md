# LLM activity

Activities calling an endpoint compatible with the OpenAI Chat Completions API, with signatures declared
in [deployment.toml](../../deployment.toml). Other APIs, such as OpenAI's Responses API or Anthropic's
Messages API, are not supported.

* `respond` sends a request to `<LLM_BASE_URL>/v1/chat/completions` and responds with the first generated message.
* `list-models` returns the model IDs from `<LLM_BASE_URL>/v1/models`. The workflow uses the first one
  when the settings name no model.

## Prerequisites
In the cloud `LLM_BASE_URL` points to the LLM integration, which authenticates the requests,
no key is needed.

Locally the endpoint defaults to `https://api.openai.com`. Create an [OpenAI API key](https://platform.openai.com/api-keys)
with the write permission for `/v1/chat/completions`:
```sh
export LLM_API_KEY="..."
# export LLM_BASE_URL="..." # another endpoint with the Chat Completions API
```

## Configuration
The second argument of `respond` is `settings-json`, a JSON-encoded object with
`model`, optional `messages` prepended to the user prompt (e.g. a system message) and `max_tokens`.
The workflows read it from the database, see [LLM settings](../db/README.md#llm-settings).

## Running the activity
```sh
# in repo root
just serve
```
In another terminal:
```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:llm/llm.list-models '[]'
obelisk execution submit --follow stargazers:llm/llm.respond \
    '["Tell me about Rust programming", "{\"model\": \"gpt-4o-mini\", \"max_tokens\": 50}"]'
```

### Ad-hoc query using curl
```sh
curl -X POST "${LLM_BASE_URL:-https://api.openai.com}/v1/chat/completions" \
-H "Authorization: Bearer ${LLM_API_KEY}" \
-H "Content-Type: application/json" \
-d '{
    "messages": [
        { "role": "system", "content": "You are a helpful assistant" },
        { "role": "user", "content": "Tell me about Rust programming." }
    ],
    "model": "gpt-4o-mini",
    "max_tokens": 200
}'
```
