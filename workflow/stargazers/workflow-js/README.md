# Workflow-js

Go reimplementation of the [Stargazers workflow](../worflow-rs/).


## Building
```sh
npm install
npm run build # produces dist/workflow-js.wasm
```

## Deplying and running with Obelisk

```sh
# in repo root
obelisk server run --config obelisk-local-js-workflow.toml
```

## Executing `star-added-parallel` workflow
```sh
npm run test:submit
```

## Testing
See [workflow readme](../workflow-rs/README.md).
