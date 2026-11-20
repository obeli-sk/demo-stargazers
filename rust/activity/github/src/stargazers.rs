#![allow(dead_code)]

#[cynic::schema("github")]
mod schema {}

/*
Generated using https://generator.cynic-rs.dev/ from:

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
*/
#[derive(cynic::QueryVariables, Debug)]
pub struct QueryStargazersVariables {
    pub cursor: Option<String>,
    pub page: i32,
    pub repo: Uri,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "QueryStargazersVariables")]
pub struct Repository {
    pub __typename: String,
    #[arguments(first: $page, after: $cursor)]
    pub stargazers: StargazerConnection,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct StargazerConnection {
    pub nodes: Option<Vec<Option<User>>>,
    pub edges: Option<Vec<Option<StargazerEdge>>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct StargazerEdge {
    pub cursor: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct User {
    pub login: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "QueryStargazersVariables")]
pub struct QueryStargazers {
    #[arguments(url: $repo)]
    pub resource: Option<UniformResourceLocatable>,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(variables = "QueryStargazersVariables")]
pub enum UniformResourceLocatable {
    Repository(Repository),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "URI")]
pub struct Uri(pub String);
