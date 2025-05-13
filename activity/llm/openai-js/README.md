# openai-js Activity

JavaScript reimplementation of [chatgpt](../chatgpt/) activity.

## Building
```sh
npm install
npm run build # produces dist/openai-js.wasm
```

## Deplying and running with Obelisk
```sh
# in repo root
obelisk server run --config obelisk-local-js-activity.toml
```

## Testing
```sh
npm run test:submit
```

## TODOs, quirks

* TODO: Switch to `process.env[ENV_OPENAI_API_KEY]` when https://github.com/bytecodealliance/ComponentizeJS/issues/190 is resolved.
