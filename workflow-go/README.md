# WIP for porting the workflow to Go

Blocker: https://github.com/tinygo-org/tinygo/issues/4843

> The wasip2 target of TinyGo assumes that the component is targeting wasi:cli/command@0.2.0 world (part of wasi:cli) so it requires the imports of wasi:cli/imports@0.2.0.
[Source](https://component-model.bytecodealliance.org/language-support/go.html#2-determine-which-world-the-component-will-implement)

## Generate go bindings
Get `wit-bindgen-go` from https://github.com/bytecodealliance/go-modules/ .

```sh
wit-bindgen-go generate wit/
```

## TODOs
Investigate virtualizing wasi interfaces
Rewrite an activity using [http-client-example](https://github.com/wasmCloud/go/tree/main/examples/component/http-client)
Rewrite a webhook using [http-server-example](https://github.com/wasmCloud/go/tree/main/examples/component/http-server)
