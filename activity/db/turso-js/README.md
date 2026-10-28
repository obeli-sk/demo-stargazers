# turso-js Activities

JavaScript reimplementation of the [Turso](../turso/) activity, one file per exported function.
The signatures come from the [WIT interface](../wit-interface/stargazers_db/db.wit).
Follow the prerequisites section of the [Turso activity README](../turso/README.md) to set up the database.

`migrate.js` ports refinery's runner and shares its history table, so either implementation can migrate the same database.
The migrations are embedded in the generated `migrations.js`; run `node generate-migrations.mjs` after changing them.

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
