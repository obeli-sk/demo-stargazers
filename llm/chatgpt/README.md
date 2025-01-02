# ChatGPT Activity

Activity that sends a JSON request to `https://api.openai.com/v1/chat/completions`
and responds with the first generated response.
It implements the [LLM Activity WIT](../interface/llm.wit) interface.

## Prerequisites
OpenAI token is required. Navigate to [API Tokens](https://platform.openai.com/api-keys) and
crate a new restricted token. Make sure the resource `/v1/chat/completions` under Model capabilities
has the write permission.
The token must be accesible as `OPENAI_API_KEY` environment variable.

## Configuration
The second argument of `respond` WIT function is `settings`, which is JSON encoded string.
The corresponding `Settings` struct in [lib.rs](src/lib.rs) provides the details on the structure,
and the `request_should_succeed` integration test shows how to set up a model,
send a system message and restrict the number of tokens returned by the service.
Those settings could be encoded directly in [llm.wit](../interface/llm.wit),
but were omitted for simplicity.

## Testing

```sh
export OPENAI_API_KEY=...
cargo test -- --ignored --nocapture
```
