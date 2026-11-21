# GitHub activities

Implement `stargazers:github/account` using the GitHub GraphQL API, one file per exported function;
the signatures are declared in [deployment.toml](../../deployment.toml).
`account-info` fetches basic info like repositories and contributions of an account,
`list-stargazers` pages through the stargazers of a repository.

## Prerequisites
In the cloud the requests go through the GitHub integration, no token is needed.

Locally a [classic GitHub token](https://github.com/settings/tokens/) with `read:org` permission is required:
```sh
export GITHUB_TOKEN_STARGAZERS="..."
```

## Running the activities
```sh
# in repo root
just serve
```
In another terminal:
```sh
export OBELISK_API_TOKEN="..." # API startup token printed by `server run --dev`
obelisk execution submit --follow stargazers:github/account.account-info '["torvalds"]'
obelisk execution submit --follow stargazers:github/account.list-stargazers '["obeli-sk/obelisk", 5, null]'
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
-H "Authorization: Bearer $GITHUB_TOKEN_STARGAZERS" https://api.github.com/graphql \
-d @-
```
