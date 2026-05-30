# openai-legacy-componentizejs Activity

Legacy ComponentizeJS reimplementation of [openai](../openai/) activity.

## Building
```sh
npm install
npm run build # produces dist/openai-legacy-componentizejs.wasm
```

## Deplying and running with Obelisk
```sh
# in repo root
obelisk server run --config obelisk-local-legacy-componentizejs-activity.toml
```

## Testing
```sh
npm run test:submit
```

## TODOs, quirks

* TODO: Switch to `process.env[ENV_OPENAI_API_KEY]` when https://github.com/bytecodealliance/ComponentizeJS/issues/190 is resolved.
