# openai-go Activity

Go reimplementation of [openai](../openai/) activity.

## Setting up
Required versions of `tinygo`, `wit-bindgen-go-cli`, `wasm-tools` can be found in [dev-deps.txt](../dev-deps.txt).
See [Go tooling](https://component-model.bytecodealliance.org/language-support/go.html) for more information.

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

## Deplying and running with Obelisk
```sh
# in repo root
obelisk server run --config obelisk-local-go-activity.toml
```

## Testing
```sh
obelisk client execution submit --follow stargazers:llm/llm.respond \
    '["Tell me about Rust programming", "{\"model\": \"gpt-3.5-turbo\", \"max_tokens\": 50}"]'
```
