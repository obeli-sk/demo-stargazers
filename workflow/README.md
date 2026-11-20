# Stargazers workflow

JavaScript reimplementation of the [Stargazers workflow](../rust/workflow/).

## Running with Obelisk

```sh
# in repo root
obelisk server run --dev --app-config app.toml --deployment deployment.toml
```

## Executing `star-added-parallel` workflow

```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:workflow/workflow.star-added-parallel '["tomasol","obeli-sk/obelisk"]'
```

## Testing

See [workflow readme](../rust/workflow/README.md).
