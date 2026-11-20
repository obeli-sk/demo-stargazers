# LLM activity

Implements the [`stargazers:llm/llm` WIT interface](./wit-interface/stargazers_llm/llm.wit):
sends a request to `<LLM_BASE_URL>/v1/chat/completions` of an OpenAI-compatible endpoint
and responds with the first generated message.

## Prerequisites
In the cloud `LLM_BASE_URL` points to the LLM integration, which authenticates the requests,
no key is needed. The model must be one offered by the integration, listed at `<LLM_BASE_URL>/v1/models`.

Locally the endpoint defaults to `https://api.openai.com`. Create an [OpenAI API key](https://platform.openai.com/api-keys)
with the write permission for `/v1/chat/completions`:
```sh
export LLM_API_KEY="..."
# export LLM_BASE_URL="..." # another OpenAI-compatible endpoint
```

## Configuration
The second argument of `respond` is `settings-json`, a JSON-encoded object with
`model`, optional `messages` prepended to the user prompt (e.g. a system message) and `max_tokens`.
The workflows read it from the database, see [Inserting the LLM settings](../db/README.md#inserting-the-llm-settings).

## Running the activity
```sh
# in repo root
just serve
```
In another terminal:
```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
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
