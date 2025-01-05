# GitHub Account Activity

Activity that fetches basic info like repositories, contributions etc. for a given account.
It implements the [Account Activity WIT](../interface/account.wit) interface.

## Prerequisites
[Classic GitHub token](https://github.com/settings/tokens/) with `read:org` permission is required.
The token must be accesible as `GITHUB_TOKEN` environment variable.
```sh
export GITHUB_TOKEN=...
```

## Testing

## Unit testing
```sh
cargo test
```

## Integration testing

```sh
export TEST_GITHUB_LOGIN="..."
export TEST_GITHUB_REPO="..."
# optinally export TEST_GITHUB_STARGAZERS_CURSOR="..."
cargo test -- --ignored --nocapture
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
