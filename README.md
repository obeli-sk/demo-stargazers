# Stargazers

A simple [Obelisk](https://github.com/obeli-sk/obelisk) workflow
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

## Running

Set up the environment:
If [direnv](https://github.com/direnv/direnv) and [Nix](https://nixos.org/) are available:
```sh
cp .envrc-example .envrc
direnv allow
```
Otherwise install the following:
* [Obelisk](https://github.com/obeli-sk/obelisk)
* Optionally [Rust](https://rustup.rs/) for building the WASM components locally, version and other components are specified in [rust-toolchain.toml](./rust-toolchain.toml)
* Optinally [Wasmtime](https://wasmtime.dev/) for integration testing of activities
* Optinally [Cloudflared](https://github.com/cloudflare/cloudflared) for exposing the webhook endpoint


### Setting up the external services

#### Turso activity
Follow the prerequisites section of the [activity-db-turso README](./activity/db/turso/README.md).

#### ChatGPT activity
Follow the prerequisites section of the [activity-llm-chatgpt README](./activity/llm/chatgpt/README.md).

#### GitHub activity
Follow the prerequisites section of the [activity-account-github README](./activity/account/github/README.md).

#### GitHub webhook endpoint
Follow the prerequisites section of the [webhook README](./webhook//README.md).

### Running the obelisk server
```sh
obelisk server run --config ./obelisk-oci.toml
```

The server start downloading the WASM components from the Docker Hub. Wait for the following
lines in the process output:

```log
INFO init:spawn_executors_and_webhooks: HTTP server `webhook_server` is listening on http://127.0.0.1:9090
INFO init:spawn_executors_and_webhooks: HTTP server `webui` is listening on http://127.0.0.1:8080
INFO Serving gRPC requests at 127.0.0.1:5005
```

The workflow can be started using the Web UI.
The webhook endpoint can be triggered using `curl` or by seting up the webhook
in a GitHub repo. See the [webhook documentation](webhook/README.md).

### Building the WASM components locally
The configuration above downloads the WASM Components from the Docker Hub.
To build all the components locally run
```sh
cargo build
obelisk server run --config ./obelisk-local.toml
```
