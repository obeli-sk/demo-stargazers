# Stargazers workflow

This workflow implements the following [WIT interface](../../workflow/wit-interface/stargazers_workflow/workflow.wit).


## Running the workflow
Build the Rust components and run Obelisk with `rust/deployment.toml`, from the `rust` directory:
```sh
just serve
```
In another terminal run the activity.
```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:workflow/workflow.backfill-parallel '["obeli-sk/obelisk"]'
```
