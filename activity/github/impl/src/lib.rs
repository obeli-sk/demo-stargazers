use crate::exports::stargazers::github::account::Guest;
use cynic::GraphQlResponse;
use exports::stargazers::github::account::Stargazers;
use serde::Serialize;
use stargazers::{
    QueryStargazers, QueryStargazersVariables, Repository, StargazerConnection, StargazerEdge,
    UniformResourceLocatable,
};
use waki::Client;
use wit_bindgen::generate;
mod stargazers;

const ENV_GITHUB_TOKEN: &str = "GITHUB_TOKEN";

generate!({ generate_all });
pub(crate) struct Component;
export!(Component);

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphQLRequest {
    query: &'static str,
    variables: serde_json::Value,
}

fn send_query<T: Serialize + ?Sized, R: serde::de::DeserializeOwned>(
    query: &T,
) -> Result<R, String> {
    let github_token = std::env::var(ENV_GITHUB_TOKEN)
        .map_err(|_| format!("{ENV_GITHUB_TOKEN} must be passed as environment variable"))?;
    let resp = Client::new()
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {github_token}"))
        .header("User-Agent", "test")
        .json(&query)
        .send()
        .map_err(|err| format!("cannot send the request - {err:?}"))?;
    if resp.status_code() != 200 {
        return Err(format!("Unexpected status code: {}", resp.status_code()));
    }
    resp.json()
        .map_err(|err| format!("deserialization error - {err:?}"))
}

fn extract_stargazers(
    resp: GraphQlResponse<QueryStargazers>,
) -> Result<Option<Stargazers>, String> {
    match resp {
        GraphQlResponse {
            data:
                Some(QueryStargazers {
                    resource:
                        Some(UniformResourceLocatable::Repository(Repository {
                            stargazers:
                                StargazerConnection {
                                    nodes: Some(nodes),
                                    edges: Some(mut edges),
                                },
                            __typename: _,
                        })),
                }),
            errors: None,
        } => match edges.pop() {
            Some(Some(StargazerEdge { cursor })) => {
                let logins = nodes
                    .into_iter()
                    .filter_map(|user| user.map(|u| u.login))
                    .collect();
                Ok(Some(Stargazers { cursor, logins }))
            }
            _ => Ok(None),
        },
        other => Err(format!("Unexpected response - {other:?}")),
    }
}

impl Guest for Component {
    fn account_info(login: String) -> Result<String, String> {
        let query = GraphQLRequest {
            query: QUERY_ACCOUNT_INFO,
            variables: serde_json::to_value(&UserArguments { login })
                .expect("`UserArguments` must be serializable"),
        };
        let resp: GraphQlResponse<serde_json::Value> = send_query(&query)?;
        if let Some(data) = resp.data {
            Ok(data.to_string())
        } else {
            Err(format!("data part is missing, errors: {:?}", resp.errors))
        }
    }

    fn list_stargazers(
        repo: String,
        page_size: u8,
        cursor: Option<String>,
    ) -> Result<Option<Stargazers>, String> {
        use cynic::QueryBuilder;
        let vars = QueryStargazersVariables {
            cursor: cursor.as_deref(),
            page: i32::from(page_size),
            repo: stargazers::Uri(repo),
        };
        let query = QueryStargazers::build(vars);
        let resp: GraphQlResponse<QueryStargazers> = send_query(&query)?;
        extract_stargazers(resp)
    }
}

const QUERY_ACCOUNT_INFO: &str = r#"
query UserInfoQuery($login: String!) {
  user(login: $login) {
    login

    ... on User {
        organizations(orderBy: {direction: DESC, field: CREATED_AT}, last: 10) {
          nodes {
            name
          }
        }
        topRepositories(orderBy: {direction: DESC, field: STARGAZERS}, first: 10) {
          nodes {
            owner {
              login
            }
            name
            homepageUrl
            isFork
            description
            languages(orderBy: {direction: DESC, field: SIZE}, first: 3) {
              nodes {
                name
              }
            }
            stargazers {
              totalCount
            }
          }
        }
      }
  }
}
"#;

#[derive(serde::Serialize, Debug)]
pub struct UserArguments {
    pub login: String,
}

#[cfg(test)]
mod tests {
    use crate::Component;
    use crate::exports::stargazers::github::account::Guest;
    use crate::{ENV_GITHUB_TOKEN, extract_stargazers, stargazers::QueryStargazers};
    use cynic::GraphQlResponse;

    fn set_up() {
        let test_token = std::env::var(format!("TEST_{ENV_GITHUB_TOKEN}")).unwrap_or_else(|_| {
            panic!("TEST_{ENV_GITHUB_TOKEN} must be set as an environment variable")
        });
        unsafe { std::env::set_var(ENV_GITHUB_TOKEN, test_token) };
    }

    #[test]
    #[ignore]
    fn account_info_request_should_succeed() {
        set_up();
        let username =
            std::env::var("TEST_GITHUB_LOGIN").expect("`TEST_GITHUB_LOGIN` envvar must be set");
        let res = Component::account_info(username);
        let res = res.unwrap();
        println!("{res}");
    }

    #[test]
    #[ignore]
    fn list_stargazers_request_should_succeed() {
        set_up();
        let repo =
            std::env::var("TEST_GITHUB_REPO").expect("`TEST_GITHUB_REPO` envvar must be set");
        let page_size = 5;
        let cursor = std::env::var("TEST_GITHUB_STARGAZERS_CURSOR").ok();
        let res = Component::list_stargazers(repo, page_size, cursor);
        let res = res.unwrap();
        println!("{res:?}");
    }

    #[test]
    fn extract_stargazers_should_return_last_cursor_and_logins() {
        let resp = serde_json::json!(
            {
              "data": {
                "resource": {
                  "__typename": "Repository",
                  "stargazers": {
                    "nodes": [
                      {
                        "login": "aa"
                      },
                      {
                        "login": "bb"
                      }
                    ],
                    "edges": [
                      {
                        "cursor": "cc"
                      },
                      {
                        "cursor": "dd"
                      }
                    ]
                  }
                }
              }
            }
        );
        let resp: GraphQlResponse<QueryStargazers> = serde_json::from_value(resp).unwrap();
        let resp = extract_stargazers(resp).unwrap();
        let resp = resp.unwrap();
        assert_eq!("dd", resp.cursor);
        assert_eq!(vec!["aa".to_string(), "bb".to_string()], resp.logins);
    }

    #[test]
    fn extract_empty_stargazers_should_return_none() {
        let resp = serde_json::json!(
            {
              "data": {
                "resource": {
                  "__typename": "Repository",
                  "stargazers": {
                    "nodes": [],
                    "edges": []
                  }
                }
              }
            }
        );
        let resp: GraphQlResponse<QueryStargazers> = serde_json::from_value(resp).unwrap();
        let resp = extract_stargazers(resp).unwrap();
        assert!(resp.is_none());
    }
}
