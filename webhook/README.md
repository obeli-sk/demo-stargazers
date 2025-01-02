# Webhook Endpoint for GitHub

This endpoint receives Star events of a configured repository, triggers
`star-added` or `star-removed` [workflows](wit/deps/workflow-interface/workflow.wit).


## Prerequisites
The endpoint must be publicly available in order for GitHub to be able to send the events.
Either deploy the Obelisk server on a VPS, or use a tunneling software (see next section).

Create a webhook under your repo settings. Go to Settings/Webhooks. The URL should match
the following template: `https://github.com/[account]/[repo]/settings/hooks` .

Add a new webhook, select individual events and make sure only Stars events are enabled.

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

To obtain a public address using `cloudflared`, run:
```sh
cloudflared tunnel --url http://127.0.0.1:9090
```

Set up the webhook on your GitHub repository and try starring it.
Check the [Web UI](http://127.0.0.1:8080) of Obelisk for execution details.
