# openai-js Activity

JavaScript reimplementation of [chatgpt](../chatgpt/) activity.

## Building
```sh
npm install
npm run build # produces dist/openai-js.wasm
```

## Deplying and running with Obelisk

```sh
obelisk server run --config $reporoot/obelisk-local-js-activity.toml
```

## Executing activity
```sh
npm run test:submit
```

## TODOs, quirks

* TODO: Automatically add  WIT imports to `external` section of `esbuild.config.js`.
* `console.log` does not directly print to stdout, fixed by polyfilling it.
