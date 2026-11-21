# Stargazers workflows

Implement `stargazers:workflow/workflow`, one file per exported function;
the signatures are declared in [deployment.toml](../deployment.toml):

* `star-added`, `star-removed`: started by the [webhook](../webhook/) on a Star event.
* `star-added-parallel`: like `star-added`, fetching the account info and the LLM settings concurrently.
* `backfill`, `backfill-parallel`: walk all stargazers of a repository and run `star-added` for each.

When the LLM settings name no model, [llm-settings.js](./llm-settings.js) calls `list-models` and uses the
first one, so the choice is visible in the workflow's execution log.

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
