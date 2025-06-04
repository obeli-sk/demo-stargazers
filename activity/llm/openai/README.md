# OpenAI Activity

Activity that sends a JSON request to `https://api.openai.com/v1/chat/completions`
and responds with the first generated response.
It implements the [`stargazers:llm/llm` WIT interface](../interface/llm.wit).

## Prerequisites
OpenAI token is required. Navigate to [API Tokens](https://platform.openai.com/api-keys) and
crate a new restricted token. Make sure the resource `/v1/chat/completions` under Model capabilities
has the write permission.
The token must be accesible as `OPENAI_API_KEY` environment variable.
```sh
export OPENAI_API_KEY="..."
```

## Configuration
The second argument of `respond` WIT function is `settings`, which is JSON encoded string.
The corresponding `Settings` struct in [lib.rs](src/lib.rs) provides the details on the structure,
and the `request_should_succeed` integration test shows how to set up a model,
send a system message and restrict the number of tokens returned by the service.
Those settings could be encoded directly in [llm.wit](../interface/llm.wit),
but were omitted for simplicity.

## Running the activity
Build the activity and run Obelisk with `obelisk-local.toml` configuration in the root of the repository.
```sh
cargo build --release
obelisk server run --config ./obelisk-local.toml
```
In another terminal run the activity.

Use the following parameters:
* parameter `user-prompt`: `"Tell me about Rust programming."`
* parameter `settings-json`: `"{\"model\": \"gpt-3.5-turbo\",\"max_tokens\": 50}"`

```sh
obelisk client execution submit --follow stargazers:llm/llm.respond '["Tell me about Rust programming", "{\"model\": \"gpt-3.5-turbo\", \"max_tokens\": 50}"]'
```

## Integration testing

```sh
export TEST_OPENAI_API_KEY=...
cargo nextest run -- --ignored
```

To execute an ad-hoc query directly to api.openai.com using curl:
```sh
curl -X POST https://api.openai.com/v1/chat/completions \
-H "Authorization: Bearer ${OPENAI_API_KEY}" \
-H "Content-Type: application/json" \
-d '{
    "messages": [
        {
            "role": "system",
            "content": "You are a helpful assistant"
        },
        {
            "role": "user",
            "content": "Tell me about Rust programming."
        }
    ],
    "model": "gpt-3.5-turbo",
    "max_tokens": 200
}'
```
