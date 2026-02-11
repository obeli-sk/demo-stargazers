# GitHub Activity

Activity that queries GitHub.
It implements the [`stargazers:github/account` WIT interface](../interface/github.wit).
The `account` interface fetches basic info like repositories, contributions etc. for a given account.

## Prerequisites
[Classic GitHub token](https://github.com/settings/tokens/) with `read:org` permission is required.
The token must be accesible as `GITHUB_TOKEN` environment variable.
```sh
export GITHUB_TOKEN="..."
```

## Running the activity
Build the activity and run Obelisk with `obelisk-local.toml` configuration in the root of the repository.
```sh
cargo build --release
obelisk server run --config ./obelisk-local.toml
```
In another terminal run the activity.
```sh
obelisk execution submit --follow stargazers:github/account.account-info '["your-github-login"]'
```

## Testing

## Unit testing
```sh
cargo nextest run
```

## Integration testing

```sh
export TEST_GITHUB_TOKEN="..."
export TEST_GITHUB_LOGIN="..."
export TEST_GITHUB_REPO="..."
# optinally export TEST_GITHUB_STARGAZERS_CURSOR="..."
cargo nextest run -- --ignored
```

### Ad-hoc GraphQL query for debugging
```sh
QUERY='
query QueryStargazers($repo: URI!, $page: Int!, $cursor: String) {
  resource(url: $repo) {
    ... on Repository {
      __typename
      stargazers(first: $page, after: $cursor) {
        nodes {
          login
        }
        edges {
          cursor
        }
      }
    }
  }
}
'
echo '
{"query":"'$(echo $QUERY)'", "variables":{"repo":"obeli-sk/obelisk", "page":2}}
' | curl -X POST \
-H "User-Agent: test" \
-H "Authorization: Bearer $GITHUB_TOKEN" https://api.github.com/graphql \
-d @-

```
