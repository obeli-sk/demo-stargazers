// JavaScript implementation of stargazers:github/account.list-stargazers
import { query } from './graphql.js';

const QUERY_STARGAZERS = `
query QueryStargazers($repo: URI!, $page: Int!, $cursor: String) {
  resource(url: $repo) {
    ... on Repository {
      __typename
      stargazers(first: $page, after: $cursor) {
        nodes { login }
        edges { cursor }
      }
    }
  }
}`;

// Returns null when the page is empty.
export default async function list_stargazers(repo, pageSize, cursor) {
    const resp = await query(QUERY_STARGAZERS, { repo, page: pageSize, cursor });
    const stargazers = resp.data?.resource?.stargazers;
    if (resp.errors || resp.data?.resource?.__typename !== 'Repository' || !stargazers?.nodes || !stargazers?.edges) {
        throw `Unexpected response - ${JSON.stringify(resp)}`;
    }
    const last = stargazers.edges.at(-1);
    if (!last) return null;
    return {
        logins: stargazers.nodes.filter((user) => user).map((user) => user.login),
        cursor: last.cursor,
    };
}
