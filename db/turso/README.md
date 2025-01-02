# Turso Activity
Implements the [db WIT interface](../interface/db.wit) using Turso.

## Protocol
The activity uses the [libSQL Remote Protocol](https://docs.turso.tech/sdk/http/reference).

## Prerequisites
Database domain and token with read and write permission is required.
The token must be accesible as `TURSO_TOKEN` environment variable.
The database domain must be accessible as `TURSO_LOCATION`, typically in
the following form: `[databaseName]-[organizationSlug].turso.io`

## Testing

### Unit testing
```sh
cargo test
```

### Integration testing
⚠️ Warning: Executing integration tests will clear the data in the testing database.

Set up a Turso database using the [schema](ddl/schema.sql).

Create a token, then export it as environment variables:
```sh
export TURSO_TOKEN="..."
export TURSO_LOCATION="[databaseName]-[organizationSlug].turso.io"
```

To run the integration tests:
```sh
WASMTIME_BACKTRACE_DETAILS=1 cargo test -- --nocapture --ignored
```

#### Ad-hoc querying using curl
The following will gett the data from the `users` table:
```sh
curl -X POST "https://${TURSO_LOCATION}/v2/pipeline" \
-H "Authorization: Bearer ${TURSO_TOKEN}" \
-H "Content-Type: application/json" \
-d '{
  "requests": [
    { "type": "execute", "stmt": { "sql": "SELECT * FROM users" } },
    { "type": "close" }
  ]
}'
```
