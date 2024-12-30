# GitHub account activity

Activity that fetches basic info like repositories, contributions etc. for a given account.

## Prerequisites
[Classic GitHub token](https://github.com/settings/tokens/) with `read:org` permission.

## Testing

```sh
export GITHUB_TOKEN=...
export GITHUB_LOGIN=...
cargo test -- --ignored --nocapture
```
