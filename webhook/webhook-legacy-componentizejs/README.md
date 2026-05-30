# webhook-legacy-componentizejs

Legacy ComponentizeJS reimplementation of [webhook](../webhook-rs/)

## Building
```sh
npm install
npm run build # produces dist/webhook-legacy-componentizejs.wasm
```

## Deplying and running with Obelisk
```sh
# in repo root
obelisk server run --config obelisk-local-legacy-componentizejs-webhook.toml
```

## Testing
See [webhook readme](../webhook-rs/README.md).
