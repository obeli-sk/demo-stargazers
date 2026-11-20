# Turso activities

Implement the [`stargazers:db/user`, `stargazers:db/llm` and `stargazers:db/migration` WIT interface](./wit-interface/stargazers_db/db.wit)
using [Turso](https://turso.tech/) over the [libSQL Remote Protocol](https://docs.turso.tech/sdk/http/reference),
one file per exported function.

## Prerequisites
A database URL and a token with read and write permission are required:
```sh
export TURSO_TOKEN="..."
export TURSO_LOCATION="https://[databaseName]-[organizationSlug].turso.io"
```

## Schema migrations
The schema is created and updated by the [migrations](./migrations/), applied by
`stargazers:db/migration.migrate`. Each deployment runs it once on activation with a `@once`
[cron](../../deployment.toml) entry.

`migrate.js` ports the runner of [refinery](https://github.com/rust-db/refinery), used by the
[Rust implementation](../../rust/activity/db/), and shares its history table, so either implementation
can migrate the same database. Each migration runs in a single transaction together with its
`refinery_schema_history` row.

To add a migration, create the next `V{version}__{name}.sql` file, regenerate the embedded copy
with `node generate-migrations.mjs` and bump the cron `params` to the new version in both
deployments. Applied migrations must never be edited, as their checksums are verified.

The migration can also be run manually:
```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:db/migration.migrate '[2]'
```

## Inserting the LLM settings
The `get-settings-json` function reads the system prompt, model and token limit passed to the
[LLM activity](../llm/). In the cloud the model must be one offered by the LLM integration,
e.g. `openai/gpt-4o-mini`; locally it is a model of the configured endpoint, e.g. `gpt-4o-mini`.
```sh
SETTINGS_JSON='{
    "messages": [
        {
            "role": "system",
            "content": "Generate concise information about GitHub users based on the JSON provided."
        }
    ],
    "model": "gpt-4o-mini",
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

## Running the activities
```sh
# in repo root
just serve
```
In another terminal:
```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:db/llm.get-settings-json '[]'
obelisk execution submit --follow stargazers:db/user.list-stargazers '[5, null, "descending"]'
```

### Ad-hoc querying using curl
```sh
curl -X POST "${TURSO_LOCATION}/v2/pipeline" \
-H "Authorization: Bearer ${TURSO_TOKEN}" \
-H "Content-Type: application/json" \
-d '{
  "requests": [
    { "type": "execute", "stmt": { "sql": "SELECT * FROM users" } },
    { "type": "close" }
  ]
}'
```

## Testing
```sh
just test-unit
```
