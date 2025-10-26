# Workflow-go

Go reimplementation of the [Stargazers workflow](../workflow-rs/).

## CLI world workaround
TinyGo is used to create the WASI 0.2 component. Unfortunately, it is not currently possible to create a component without including the `wasi:cli/command` world.

Obelisk can stub those imports with functions that trap or return a deterministic value (e.g. random functions, clock functions).
This **should** mitigate the nondeterminism coming by the `cli` world.See [TinyGo issue](https://github.com/tinygo-org/tinygo/issues/4843) for details.

Obelisk must be configured to stub the WASI imports of workflows authored in Go:

```toml
[[workflow]]
stub_wasi = true
```

## Setting up
Required versions of `tinygo`, `wit-bindgen-go-cli`, `wasm-tools` can be found in [dev-deps.txt](../../../dev-deps.txt).
See [Go tooling](https://component-model.bytecodealliance.org/language-support/go.html) for more information.

```sh
go mod init ...
./regen.sh
```

## Building
```sh
./build.sh
```

## Deplying and running with Obelisk
```sh
# in repo root
obelisk server run --config obelisk-local-go-workflow.toml
```

## Testing
See [workflow readme](../workflow-rs/README.md).
