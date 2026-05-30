# workflow-js

JavaScript reimplementation of the [Stargazers workflow](../workflow-rs/).

## Running with Obelisk

```sh
# in repo root
obelisk server run --config obelisk-local-js-all.toml
```

## Executing `star-added-parallel` workflow

```sh
obelisk execution submit --follow stargazers:workflow/workflow.star-added-parallel '["tomasol","obeli-sk/obelisk"]'
```

## Testing

See [workflow readme](../workflow-rs/README.md).
