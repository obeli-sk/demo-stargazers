# LLM Activity (Rust)

Rust implementation of the [LLM activity](../../../activity/llm/README.md), which describes the
prerequisites. The `Settings` struct in [lib.rs](src/lib.rs) defines the `settings-json` structure.

## Running the activity
Build the Rust components and run Obelisk with `rust/deployment.toml`, from the `rust` directory:
```sh
just serve
```
In another terminal:
```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:llm/llm.respond '["Tell me about Rust programming", "{\"model\": \"gpt-4o-mini\", \"max_tokens\": 50}"]'
```

## Integration testing
```sh
export TEST_LLM_API_KEY=...
cargo nextest run -- --ignored
```
