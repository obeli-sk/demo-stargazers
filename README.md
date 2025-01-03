# Stargazers

A simple [obelisk](https://github.com/obeli-sk/obelisk) workflow
that monitors a project stargazers using a webhook.
When a user stars the project, a webhook event is received.

On a *star added* event a new workflow execution is submitted.
First, an activity persists the GitHub username, then, a background check is started.
The background check is just a GitHub request getting
basic info on the user, and then an activity is called that
transforms the info into a summary using an external LLM service.
The summary is then persisted.

On a *star deleted* event an activity will delete the relation. If a
user doesn't have any starred repositories anymore, the user will be deleted as well.

## Local deployment

Create three Turso databases (dev, test, prod) with the provided [SQL schema](activity/db/turso/ddl/schema.sql).
Set up all the required environment variables:
```sh
export GITHUB_TOKEN="..."
export OPENAI_API_KEY="..."
export TURSO_TOKEN="..."
export TURSO_LOCATION="[databaseName]-[organizationSlug].turso.io"
```
Details are described in each activity's documentation.

Configure the LLM system prompt:
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
        "sql": "INSERT INTO llm (id, settings) VALUES (0, :settings) ON CONFLICT (id) DO UPDATE SET settings = :settings",
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

### Running the obelisk server
```sh
obelisk server run --config ./obelisk.toml
```
After a couple of seconds the server should start listening:
* webhook at http://127.0.0.1:9090
* gRPC at 127.0.0.1:5005
* Web UI at [localhost:8080](http://127.0.0.1:8080)

The workflow can be started using the Web UI.
The webhook endpoint can be triggered using `curl` or by seting up the webhook
in a GitHub repo. See the [webhook documentation](webhook/README.md).
