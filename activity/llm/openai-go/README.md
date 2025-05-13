# openai-go Activity

Go reimplementation of [openai](../openai/) activity.

## Setting up
```sh
go mod init ...
rm -rf gen
# Regenerate bindings after modifying `wit` folder
wit-bindgen-go generate --world root --out gen wit/
go mod tidy
```

## Building
```sh
./build.sh
```

## Deployment
```sh
# in repo root
obelisk server run --config obelisk-go-activity.toml
```

## Testing
```sh
obelisk client execution submit --follow stargazers:llm/llm.respond \
    '["Tell me about Rust programming", "{\"model\": \"gpt-3.5-turbo\", \"max_tokens\": 50}"]'
```
