# Stargazers in Rust

Rust implementation of the [JavaScript app](../README.md), built locally into WASM components
referenced by [deployment.toml](./deployment.toml). The WIT interfaces shared by the crates are in [wit](./wit/).

From this directory:
```sh
just build              # needs the rustup toolchain from rust-toolchain.toml, provided by the Nix flake
just serve              # builds, approves the webhook's secret exposure and runs Obelisk
just test-unit
just test-integration   # against local mocks and sqld
just test-e2e
```
After changing the WIT, regenerate the extension packages with `./scripts/obelisk-generate-wit-extensions.sh`.
