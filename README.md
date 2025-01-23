# Stargazers

A simple [Obelisk](https://github.com/obeli-sk/obelisk) workflow
that monitors a project stargazers using a webhook.
When a user stars the project, a webhook event is received.

## Workflow functions

### Star Added Event

On receiving a *star added* GitHub webhook event:

1. **Persist Username**: A workflow execution is triggered to store the GitHub username in the database.
2. **Fetch User Info**: Requests basic user info from GitHub, like repositories, organizations etc.
3. **Transform User Info**: An external Language Model (LLM) service is used to process this info into a summary.
4. **Persist Summary**: The generated summary is stored in the database.

### Star Deleted Event

On receiving a *star deleted* GitHub webhook event:

1. **Delete Relations**: An action is triggered to remove the user's star relation from the database.
2. **User Deletion**: If no repositories are starred by the user anymore, the user's data is removed completely.

### Backfill
Supports reprocessing of current stargazers.

## Workflow Code Example

```rust
impl Guest for Component {
    fn star_added(login: String, repo: String) -> Result<(), String> {
        // Persist the user giving a star to the project.
        let description = db::user::link_get_description(&login, &repo)?;
        if description.is_none() {
            // Fetch the account info from GitHub.
            let info = account::account_info(&login)?;
            let settings_json = db::llm::get_settings_json()?;
            // Generate the user's description.
            let description = llm::respond(&info, &settings_json)?;
            db::user::user_update(&login, &description)?;
        }
        Ok(())
    }

    fn star_removed(login: String, repo: String) -> Result<(), String> {
        db::user::unlink(&login, &repo)
    }

    fn backfill(repo: String) -> Result<(), String> {
        let mut cursor = None;
        while let Some(resp) = account::list_stargazers(&repo, cursor.as_deref())? {
            for login in resp.logins {
                // Submit a child workflow
                imported_workflow::star_added(&login, &repo)?;
            }
            cursor = Some(resp.cursor);
        }
        Ok(())
    }
}
```

The complete workflow source can be found [here](./workflow/src/lib.rs). Note that this code has no parallelism.
The parallel version of the `star-added` function is in branch `parallel`.

## Setting up the external services
The activities require tokens to be present.

#### Turso activity
Follow the prerequisites section of the [activity-db-turso README](./activity/db/turso/README.md).

#### ChatGPT activity
Follow the prerequisites section of the [activity-llm-chatgpt README](./activity/llm/chatgpt/README.md).

#### GitHub activity
Follow the prerequisites section of the [activity-account-github README](./activity/account/github/README.md).

#### GitHub webhook endpoint
Follow the prerequisites section of the [webhook README](./webhook//README.md).

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

The exact versions of dependencies used for development and testing are in [dev-deps.txt](./dev-deps.txt).

### Running the obelisk server
```sh
obelisk server run --config ./obelisk-oci.toml
```

The server will start downloading the WASM components from the Docker Hub. Wait for the following
lines in the process output:

```log
INFO init:spawn_executors_and_webhooks: HTTP server `webhook_server` is listening on http://127.0.0.1:9090
INFO init:spawn_executors_and_webhooks: HTTP server `webui` is listening on http://127.0.0.1:8080
INFO Serving gRPC requests at 127.0.0.1:5005
```

The workflow can be started using the Web UI.
The webhook endpoint can be triggered using `curl` or by seting up the webhook
in a GitHub repo. See the [webhook documentation](webhook/README.md) for details
on how to set up GitHub and a https tunnel to the local instance.

### Building the WASM components locally
The configuration above downloads the WASM Components from the Docker Hub.
To build all the components locally run
```sh
cargo build
obelisk server run --config ./obelisk-local.toml
```

## Testing
### Unit testing
```sh
scripts/test-unit.sh
```
### Integration testing
```sh
scripts/test-integration.sh
```
### End to end testing
```sh
scripts/test-e2e.sh ./obelisk-local.toml truncate
```
