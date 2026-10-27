// JavaScript implementation of stargazers:github/account.account-info
import { query } from './graphql.js';

const QUERY_ACCOUNT_INFO = `
query UserInfoQuery($login: String!) {
  user(login: $login) {
    login
    ... on User {
      organizations(orderBy: {direction: DESC, field: CREATED_AT}, last: 10) {
        nodes { name }
      }
      topRepositories(orderBy: {direction: DESC, field: STARGAZERS}, first: 10) {
        nodes {
          owner { login }
          name
          homepageUrl
          isFork
          description
          languages(orderBy: {direction: DESC, field: SIZE}, first: 3) {
            nodes { name }
          }
          stargazers { totalCount }
        }
      }
    }
  }
}`;

export default async function account_info(login) {
    const resp = await query(QUERY_ACCOUNT_INFO, { login });
    if (!resp.data) throw `data part is missing, errors: ${JSON.stringify(resp.errors)}`;
    return JSON.stringify(resp.data);
}
