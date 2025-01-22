# Turso Activity
Implements the [db WIT interface](../interface/db.wit) using [Turso](https://turso.tech/).

## Prerequisites
Database domain and token with read and write permission is required.
The token must be accesible as `TURSO_TOKEN` environment variable.
The database domain must be accessible as `TURSO_LOCATION`, typically in
the following form: `[databaseName]-[organizationSlug].turso.io`

```sh
export TURSO_TOKEN="..."
export TURSO_LOCATION="[databaseName]-[organizationSlug].turso.io"
```

The activity needs an active database with the following [schema](ddl/schema.sql).
```sh
generate_json() {
  local file=$1
  local DDL=$(cat "$file")
  local json='{"requests": ['

  local FIRST=true
  local STMT=""
  while IFS= read -r line; do
    if [[ -n "$line" && "$line" != "--"* ]]; then
      if [[ "$line" == *';' ]]; then
        STMT+="$line"
        [[ "$FIRST" == true ]] && FIRST=false || json+=','
        json+="  {\"type\": \"execute\", \"stmt\": {\"sql\": \"$STMT\"}}"
        STMT=""
      else
        STMT+="$line "
      fi
    fi
  done <<< "$DDL"

  json+='  ,{"type": "close"}'
  json+=']}'
  echo "$json"
}

DDL="ddl/schema.sql"
generate_json $DDL | curl -X POST "https://${TURSO_LOCATION}/v2/pipeline" \
-H "Authorization: Bearer ${TURSO_TOKEN}" \
-H "Content-Type: application/json" \
--data @-
```


### Inserting initial data

Configure the LLM system prompt, which will be read by the `get-settings-json` [WIT function](./wit/deps/db-interface/db.wit):
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
}' | curl -X POST "https://${TURSO_LOCATION}/v2/pipeline" \
-H "Authorization: Bearer ${TURSO_TOKEN}" \
-H "Content-Type: application/json" \
--data @-
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
export TEST_TURSO_LOCATION="[databaseName]-[organizationSlug].turso.io"
```

To run the integration tests:
```sh
cargo nextest run --test-threads=1 -- --ignored
```

#### Ad-hoc querying using curl
The following will get the data from the `users` table
using the [libSQL Remote Protocol](https://docs.turso.tech/sdk/http/reference):
```sh
curl -X POST "https://${TEST_TURSO_LOCATION}/v2/pipeline" \
-H "Authorization: Bearer ${TEST_TURSO_TOKEN}" \
-H "Content-Type: application/json" \
-d '{
  "requests": [
    { "type": "execute", "stmt": { "sql": "SELECT * FROM users" } },
    { "type": "close" }
  ]
}'
```
