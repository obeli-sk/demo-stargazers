# openai-go Activity

## Setting up
```sh
go mod init ...
go mod tidy # after first codegen
```

## Building

```sh
rm -rf gen
wit-bindgen-go generate --world root --out gen wit/
tinygo build -target=wasip2 -o dist/openai-go.wasm --wit-package wit/ --wit-world root main.go
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
