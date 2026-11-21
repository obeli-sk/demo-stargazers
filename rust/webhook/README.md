# Webhook Endpoint for GitHub (Rust)

Rust implementation of the [webhook endpoint](../../webhook/README.md), which describes the
GitHub, secret and tunnel setup and manual testing.

Locally built components are not reproducible, so the `exposed_to` grant of `GITHUB_WEBHOOK_SECRET`
cannot be pinned in `app.toml`. `just serve` copies `app.toml` to `target/app.toml` and approves the
freshly built webhook there.

## Running
From the `rust` directory:
```sh
just serve
```

## Testing

### Unit testing
Unit tests must be compiled to the native target, e.g. for linux:
```sh
cargo test --target=x86_64-unknown-linux-gnu
```

### End-to-end testing
```sh
just test-e2e
```
