# Webhook Endpoint for GitHub

This webook endpoint receives Star events of a configured repository, triggers
`star-added` or `star-removed` [workflows](wit/deps/workflow-interface/workflow.wit).

## Prerequisites

### Security
It is advised to [verify each webhook request](https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries).
Set up the shared secret and export it as an environemnt variable:
```sh
export GITHUB_WEBHOOK_SECRET="..."
```
Then in your `obelisk.toml` verify that the variable is forwarded:
```toml
[[webhook_endpoint]]
name = "webhook"
env_vars = ["GITHUB_WEBHOOK_SECRET"]
```

The verification can be turned off for testing purposes in your `obelisk.toml`:
```toml
[[webhook_endpoint]]
name = "webhook"
env_vars = ["GITHUB_WEBHOOK_INSECURE=true"]
```

### Exposing the HTTP server
The endpoint must be publicly available in order for GitHub to be able to send the events.
Either deploy the Obelisk server on a VPS, or use a tunneling software.

To obtain a public address using `cloudflared`, run:
```sh
cloudflared tunnel --url http://127.0.0.1:9090
```

### Configuring GitHub
Create a webhook under your repo settings. Go to Settings/Webhooks. The URL should match
the following template: `https://github.com/[account]/[repo]/settings/hooks`.

Add a new webhook, select individual events and make sure only Stars events are enabled.
Don't forget to set up a secret as mentioned in the section above.

When the GitHub configuration is saved and the HTTP server is up and running try starring the configured repository.
Check the [Web UI](http://127.0.0.1:8080) of Obelisk for execution details.
Your GitHub user should appear in the Turso database, together with a generated description.

## Testing

### Unit testing
Unit tests must be compiled to the native target, e.g. for linux:
```sh
cargo test --target=x86_64-unknown-linux-gnu
```

## Manual end-to-end testing
Disable the request verification as mentioned above.
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
