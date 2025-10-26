# webhook-go

Go reimplementation of [webhook](../webhook/).

## Setting up
Required versions of `tinygo`, `wit-bindgen-go-cli`, `wasm-tools` can be found in [dev-deps.txt](../dev-deps.txt).
See [Go tooling](https://component-model.bytecodealliance.org/language-support/go.html) for more information.

```sh
go mod init <module-path>
./regen.sh
```

## Building
```sh
./build.sh
```

## Deplying and running with Obelisk
```sh
# in repo root
obelisk server run --config obelisk-local-go-webhook.toml
```

## Testing
See [webhook readme](../webhook/README.md).
