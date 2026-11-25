# Turso activities

Persist stargazers and the LLM settings in [Turso](https://turso.tech/) over the
[libSQL Remote Protocol](https://docs.turso.tech/sdk/http/reference), one file per exported function;
the signatures are declared in [deployment.toml](../../../deployment.toml).

## Prerequisites
A database URL and a token with read and write permission are required:
```sh
export TURSO_TOKEN="..."
export TURSO_LOCATION="https://[databaseName]-[organizationSlug].turso.io"
```

## Schema migrations
The [migrations](./migrations/) use the format of [refinery](https://github.com/rust-db/refinery),
which the [Rust implementation](../../../rust/activity/db/) embeds. Apply them before deploying:
```sh
just migrate
```
[migrate.mjs](./migrate/migrate.mjs) is a Node.js port of refinery's runner and shares its
`refinery_schema_history` table, so either implementation can migrate the same database.
Each migration runs in a single transaction together with its history row.

To add a migration, create the next `V{version}__{name}.sql` file and bump the `params` of the
Rust deployment's migration cron. Applied migrations must never be edited, as their checksums are verified.

## LLM settings
`get-settings-json` reads the settings passed to the [LLM activity](../llm/): messages prepended to
the user prompt, `max_tokens` and an optional `model`. The `V3__default_llm_settings` migration stores
a basic prompt without a model, so the workflow picks the first model the LLM endpoint offers.
To change the prompt or pin a model offered by the endpoint (`<LLM_BASE_URL>/v1/models`,
e.g. `openai/gpt-4o-mini` in the cloud or `gpt-4o-mini` with OpenAI):
```sh
turso db shell stargazers "UPDATE llm SET settings = json_set(settings, '$.model', 'openai/gpt-4o-mini') WHERE id = 1"
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
The runner's tests are in [migrate/test](./migrate/test/):
```sh
just test-unit
```
