# Stargazers workflow

This workflow implements the following [WIT interface](../wit-interface/stargazers_workflow/workflow.wit).


## Running the workflow
Build the workflow and run Obelisk with `deployment-rs.toml` configuration in the root of the repository.
```sh
cargo build --release
obelisk server run --app-config ./app.toml --deployment ./deployment-rs.toml
```
In another terminal run the activity.
```sh
obelisk execution submit --follow stargazers:workflow/workflow.backfill-parallel '["obeli-sk/obelisk"]'
```
