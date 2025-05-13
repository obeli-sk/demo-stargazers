# Workflow-js

JavaScript reimplementation of the [stargazers workflow](../workflow/)


## Building
```sh
npm install
npm run build # produces dist/workflow-js.wasm
```

## Deplying and running with Obelisk

```sh
obelisk server run --config $reporoot/obelisk-local-js-workflow.toml
```

## Executing `star-added-parallel` workflow
```sh
npm run test:submit
```
