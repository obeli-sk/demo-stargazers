# Webhook Endpoint for GitHub

This webhook endpoint receives Star events of a configured repository and starts the
`star-added` or `star-removed` [workflows](../workflow/). A `GET` request with `?repo=owner/name`
lists the last updated stargazers.

## Prerequisites

### Security
It is advised to [verify each webhook request](https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries).
Set up the shared secret and export it as an environment variable:
```sh
export GITHUB_WEBHOOK_SECRET="..."
```
The secret is exposed only to the webhook component whose digest is listed in `exposed_to` in
[app.toml](../app.toml). After changing `webhook.js`, update the grant:
```sh
obelisk server verify --skip-db --fix-app-secret-exposure --app-config app.toml --deployment deployment.toml
```

The verification can be turned off for testing purposes in [deployment.toml](../deployment.toml):
```toml
[[webhook_endpoint_js]]
name = "webhook"
env_vars = ["GITHUB_WEBHOOK_INSECURE=true"]
```

### Creating a tunnel to expose the local HTTP server
In the cloud the webhook is served at `https://<app>.obelisk.works/`. Locally the `external`
HTTP server must be made publicly available so that GitHub can send the events.

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
Navigate to your GitHub repository, then go to Settings/Webhooks.

Add a new webhook, select individual events and make sure only Stars events are enabled.
Don't forget to set up the shared secret.

When the GitHub configuration is saved and Obelisk is up and running try starring the configured repository.
Check the [Web UI](http://127.0.0.1:8080) for execution details.
Your GitHub user should appear in the Turso database, together with a generated description.

## Manual end-to-end testing
Disable the request verification using `GITHUB_WEBHOOK_INSECURE` as mentioned above.
Start the `obelisk` server according to the root [README](../README.md).
Execute a request locally:
```sh
export TEST_GITHUB_LOGIN="..."

curl -v -X POST http://127.0.0.1:9090 -d '{
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
curl -v http://127.0.0.1:9090?repo=obeli-sk/obelisk
```

## Automated end-to-end testing
`just test-e2e` runs [test-e2e.sh](../scripts/test-e2e.sh), which signs the request with the secret.
