# workflow-legacy-componentizejs

Legacy ComponentizeJS reimplementation of the [Stargazers workflow](../workflow-rs/).


## Building
```sh
npm install
npm run build # produces dist/workflow-legacy-componentizejs.wasm
```

## Deplying and running with Obelisk

```sh
# in repo root
obelisk server run --config obelisk-local-legacy-componentizejs-all.toml
```

## Executing `star-added-parallel` workflow
```sh
npm run test:submit
```

## Testing
See [workflow readme](../workflow-rs/README.md).
