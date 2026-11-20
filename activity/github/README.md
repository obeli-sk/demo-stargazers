# GitHub activities

JavaScript reimplementation of the [GitHub](../../rust/activity/github/) activity, one file per exported function.
The signatures come from the [WIT interface](./wit-interface/stargazers_github/github.wit).

## Running with Obelisk

```sh
# in repo root
obelisk server run --dev --app-config app.toml --deployment deployment.toml
```

## Testing

```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:github/account.account-info '["torvalds"]'
obelisk execution submit --follow stargazers:github/account.list-stargazers '["obeli-sk/obelisk", 5, null]'
```
