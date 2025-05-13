# webhook-go

Go reimplementation of [webhook](../webhook/).

## Setting up
```sh
go mod init ...
rm -rf gen
# Regenerate bindings after modifying `wit` folder
wit-bindgen-go generate --world root --out gen wit/
go mod tidy
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
