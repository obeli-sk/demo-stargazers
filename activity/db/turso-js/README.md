# turso-js Activities

JavaScript reimplementation of the [Turso](../turso/) activity, one file per exported function.
The signatures come from the [WIT interface](../wit-interface/stargazers_db/db.wit).
Follow the prerequisites section of the [Turso activity README](../turso/README.md) to set up the database.

## Running with Obelisk

```sh
# in repo root
obelisk server run --app-config app.toml --deployment deployment-js.toml
```

## Testing

```sh
obelisk execution submit --follow stargazers:db/llm.get-settings-json '[]'
obelisk execution submit --follow stargazers:db/user.list-stargazers '[5, null, "descending"]'
```
