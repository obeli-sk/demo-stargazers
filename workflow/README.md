# Stargazers workflow

This workflow implements the following [WIT interface](interface/workflow.wit).


## Running the workflow
Build the workflow and run Obelisk with `obelisk-local.toml` configuration in the root of the repository.
```sh
cargo build --release
obelisk server run --config ./obelisk-local.toml
```
In another terminal run the activity.
```sh
obelisk client execution submit --follow stargazers:workflow/workflow.backfill-parallel '["obeli-sk/obelisk"]'
```
