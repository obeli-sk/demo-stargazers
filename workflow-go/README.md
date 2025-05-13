# WIP workflow-go

Go reimplementation of [workflow](../workflow/).

Blocker: https://github.com/tinygo-org/tinygo/issues/4843
> The wasip2 target of TinyGo assumes that the component is targeting wasi:cli/command@0.2.0 world (part of wasi:cli) so it requires the imports of wasi:cli/imports@0.2.0.
[Source](https://component-model.bytecodealliance.org/language-support/go.html#2-determine-which-world-the-component-will-implement)


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
