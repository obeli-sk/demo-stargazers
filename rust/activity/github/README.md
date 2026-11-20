# GitHub Activity (Rust)

Rust implementation of the [GitHub activities](../../../activity/github/README.md), which describe the prerequisites.

## Running the activity
Build the Rust components and run Obelisk with `rust/deployment.toml`, from the `rust` directory:
```sh
just serve
```
In another terminal:
```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:github/account.account-info '["your-github-login"]'
```

## Testing

### Unit testing
```sh
cargo nextest run
```

### Integration testing
```sh
export TEST_GITHUB_TOKEN_STARGAZERS="..."
export TEST_GITHUB_LOGIN="..."
export TEST_GITHUB_REPO="..."
# optionally export TEST_GITHUB_STARGAZERS_CURSOR="..."
cargo nextest run -- --ignored
```
