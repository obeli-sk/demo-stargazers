# Turso Activity (Rust)

Rust implementation of the [Turso activities](../../../activity/db/README.md), which describe the
prerequisites, migrations and LLM settings. The migrations are embedded and applied by
[refinery](https://github.com/rust-db/refinery) through `stargazers:db/migration.migrate`,
which `rust/deployment.toml` runs once per deployment with an `@once` cron.

## Running the activity
Build the Rust components and run Obelisk with `rust/deployment.toml`, from the `rust` directory:
```sh
just serve
```
In another terminal run the activity.
```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:db/llm.get-settings-json '[]'
```

## Testing

### Unit testing
```sh
cargo nextest run
```

### Integration testing
⚠️ Warning: Executing integration tests will clear the data in the testing database.
Each integration test expects to be the sole writer to the database.

```sh
export TEST_TURSO_TOKEN="..."
export TEST_TURSO_LOCATION="https://[databaseName]-[organizationSlug].turso.io"
cargo nextest run --test-threads=1 -- --ignored
```
`just test-integration` runs all integration tests against a local sqld and mocks instead.
