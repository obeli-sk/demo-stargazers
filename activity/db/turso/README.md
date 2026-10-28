# Turso Activity
Implements the [`stargazers:db/user`, `stargazers:db/llm` and `stargazers:db/migration` WIT interface](../wit-interface/stargazers_db/db.wit) using [Turso](https://turso.tech/).

## Prerequisites
Database domain and token with read and write permission is required.
The token must be accesible as `TURSO_TOKEN` environment variable.
The database URL must be accessible as `TURSO_LOCATION`, typically in
the following form: `https://[databaseName]-[organizationSlug].turso.io`

```sh
export TURSO_TOKEN="..."
export TURSO_LOCATION="https://[databaseName]-[organizationSlug].turso.io"
```

### Schema migrations
The schema is created and updated by the [migrations](../migrations/), applied by the
`stargazers:db/migration.migrate` function using [refinery](https://github.com/rust-db/refinery).
Each migration runs in a single transaction together with its `refinery_schema_history` row.
Both deployments run it once on activation with a `@once` [cron](../../../deployment-rs.toml) entry.

To add a migration, create the next `V{version}__{name}.sql` file, regenerate the JavaScript copy
with `node ../turso-js/generate-migrations.mjs` and bump the cron `params` to the new version
in both deployments. Applied migrations must never be edited, as their checksums are verified.

The migration can also be run manually:
```sh
obelisk execution submit --follow stargazers:db/migration.migrate '[2]'
```

### Inserting initial data

Configure the LLM system prompt, which will be read by the `get-settings-json` [WIT function](./wit/deps/stargazers_db/db.wit):
```sh
SETTINGS_JSON='{
    "messages": [
        {
            "role": "system",
            "content": "Generate conscise information about GitHub users based on the JSON provided."
        }
    ],
    "model": "gpt-3.5-turbo",
    "max_tokens": 200
}'

echo '{
  "requests": [
    {
      "type": "execute",
      "stmt": {
        "sql": "INSERT INTO llm (id, settings) VALUES (1, :settings) ON CONFLICT (id) DO UPDATE SET settings = :settings",
        "named_args": [
          {
            "name": "settings",
            "value": {
              "type": "text",
              "value": "'$(echo $SETTINGS_JSON | sed 's/\"/\\"/g')'"
            }
          }
        ]
      }
    },
    {
      "type": "close"
    }
  ]
}' | curl -X POST "${TURSO_LOCATION}/v2/pipeline" \
-H "Authorization: Bearer ${TURSO_TOKEN}" \
-H "Content-Type: application/json" \
--data @-
```

## Running the activity
Build the activity and run Obelisk with `deployment-rs.toml` configuration in the root of the repository.
```sh
cargo build --release
obelisk server run --app-config ./app.toml --deployment ./deployment-rs.toml
```
In another terminal run the activity.
```sh
obelisk execution submit --follow stargazers:db/llm.get-settings-json []
```

## Testing

### Unit testing
```sh
cargo nextest run
```

### Integration testing
⚠️ Warning: Executing integration tests will clear the data in the testing database.
Each integration test expects to be the sole writer to the database.

Create a token, then export it as environment variables:
```sh
export TEST_TURSO_TOKEN="..."
export TEST_TURSO_LOCATION="https://[databaseName]-[organizationSlug].turso.io"
```

To run the integration tests, which apply the migrations first:
```sh
cargo nextest run --test-threads=1 -- --ignored
```

#### Ad-hoc querying using curl
The following will get the data from the `users` table
using the [libSQL Remote Protocol](https://docs.turso.tech/sdk/http/reference):
```sh
curl -X POST "${TEST_TURSO_LOCATION}/v2/pipeline" \
-H "Authorization: Bearer ${TEST_TURSO_TOKEN}" \
-H "Content-Type: application/json" \
-d '{
  "requests": [
    { "type": "execute", "stmt": { "sql": "SELECT * FROM users" } },
    { "type": "close" }
  ]
}'
```
