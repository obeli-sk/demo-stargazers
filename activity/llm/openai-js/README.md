# openai-js Activity

JavaScript reimplementation of the [openai](../openai/) activity.

## Running with Obelisk

```sh
# in repo root
obelisk server run --server-config server.toml --app-config app.toml --deployment obelisk-local-js-all.toml
```

## Testing

```sh
obelisk execution submit --follow stargazers:llm/llm.respond \
    '["Tell me about Rust programming", "{\"model\": \"gpt-3.5-turbo\", \"max_tokens\": 50}"]'
```
