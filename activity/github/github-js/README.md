# github-js Activities

JavaScript reimplementation of the [GitHub](../impl/) activity, one file per exported function.
The signatures come from the [WIT interface](../wit-interface/stargazers_github/github.wit).

## Running with Obelisk

```sh
# in repo root
obelisk server run --app-config app.toml --deployment deployment-js.toml
```

## Testing

```sh
obelisk execution submit --follow stargazers:github/account.account-info '["torvalds"]'
obelisk execution submit --follow stargazers:github/account.list-stargazers '["obeli-sk/obelisk", 5, null]'
```
