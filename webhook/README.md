# Webhook Endpoint for GitHub

This endpoint receives Star events of a configured repository, triggers
`star-added` or `star-removed` [workflows](wit/deps/workflow-interface/workflow.wit).

## Testing
Start the `obelisk` server according to the root [README](../README.md).
Execute a request locally:
```sh
export TEST_GITHUB_LOGIN="..."

curl -X POST http://127.0.0.1:9090 -d '{
    "action": "created",
    "sender": {
        "login": "'$TEST_GITHUB_LOGIN'"
    },
    "repository": {
        "owner": {
            "login": "obeli-sk"
        },
        "name": "obelisk"
    }
}'
```

Observe the execution log. After the workflow succeeds, the database should contain
the user, repo, their relation, and the user should have a generated description.
