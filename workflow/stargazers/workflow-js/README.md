# workflow-js

JavaScript reimplementation of the [Stargazers workflow](../workflow-rs/).

## Running with Obelisk

```sh
# in repo root
obelisk server run --dev --app-config app.toml --deployment deployment-js.toml
```

## Executing `star-added-parallel` workflow

```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:workflow/workflow.star-added-parallel '["tomasol","obeli-sk/obelisk"]'
```

## Testing

See [workflow readme](../workflow-rs/README.md).
