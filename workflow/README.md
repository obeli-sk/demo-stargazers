# Stargazers workflows

Implement the [`stargazers:workflow/workflow` WIT interface](./wit-interface/stargazers_workflow/workflow.wit),
one file per exported function:

* `star-added`, `star-removed`: started by the [webhook](../webhook/) on a Star event.
* `star-added-parallel`: like `star-added`, fetching the account info and the LLM settings concurrently.
* `backfill`, `backfill-parallel`: walk all stargazers of a repository and run `star-added` for each.

## Running the workflows
```sh
# in repo root
just serve
```
In another terminal:
```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:workflow/workflow.star-added-parallel '["tomasol","obeli-sk/obelisk"]'
obelisk execution submit --follow stargazers:workflow/workflow.backfill-parallel '["obeli-sk/obelisk"]'
```
