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
Then in your `obelisk.toml` make sure that the variable is forwarded to the WASM instance:
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

### Creating a tunnel to expose the local HTTP server
The webhook server configured in `obelisk.toml` must be publicly available
so that GitHub can start sending the events.

To create a tunnel with a public address,
use [Cloudflare Tunnel client](https://github.com/cloudflare/cloudflared):
```sh
cloudflared tunnel --url http://127.0.0.1:9090
```
To test the tunnel, run
```sh
curl -v https://some-domain.trycloudflare.com
```

### Configuring GitHub
Navigate to your Github repository, then go to Settings/Webhooks.

Add a new webhook, select individual events and make sure only Stars events are enabled.
Don't forget to set up a shared secret.

When the GitHub configuration is saved and Obelisk is up and running try starring the configured repository.
Check the [Web UI](http://127.0.0.1:8080) for execution details.
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

To obtain a list of 5 last updated stargazers, run
```sh
curl -v http://127.0.0.1:9090
```

## Automated end-to-end testing
Check out [test-e2e.sh](/scripts/test-e2e.sh) for details.
