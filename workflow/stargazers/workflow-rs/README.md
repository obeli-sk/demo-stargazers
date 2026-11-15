# Stargazers workflow

This workflow implements the following [WIT interface](../wit-interface/stargazers_workflow/workflow.wit).


## Running the workflow
Build the workflow and run Obelisk with `deployment-rs.toml` configuration in the root of the repository.
```sh
cargo build --release
obelisk server run --dev --app-config ./app.toml --deployment ./deployment-rs.toml
```
In another terminal run the activity.
```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:workflow/workflow.backfill-parallel '["obeli-sk/obelisk"]'
```
