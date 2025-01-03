# GitHub Account Activity

Activity that fetches basic info like repositories, contributions etc. for a given account.
It implements the [Account Activity WIT](../interface/account.wit) interface.

## Prerequisites
[Classic GitHub token](https://github.com/settings/tokens/) with `read:org` permission is required.
The token must be accesible as `GITHUB_TOKEN` environment variable.

## Testing

```sh
export GITHUB_TOKEN=...
export TEST_GITHUB_LOGIN=...
cargo test -- --ignored --nocapture
```
