use crate::exports::stargazers::account::account::Guest;
use waki::Client;
use wit_bindgen::generate;

generate!({ generate_all });
pub(crate) struct Component;
export!(Component);

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphQLRequest {
    query: &'static str,
    variables: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
struct GraphqlResponse {
    data: Option<serde_json::Value>,
    errors: Option<Vec<serde_json::Value>>,
}

impl Guest for Component {
    fn info(login: String) -> Result<String, String> {
        let query = GraphQLRequest {
            query: QUERY,
            variables: serde_json::to_value(&UserArguments { login })
                .expect("`UserArguments` must be serializable"),
        };

        let github_token = std::env::var("GITHUB_TOKEN")
            .expect("GITHUB_TOKEN must be passed as environment variable");
        let resp = Client::new()
            .post("https://api.github.com/graphql")
            .header("Authorization", format!("Bearer {github_token}"))
            .header("User-Agent", "test")
            .json(&query)
            .send()
            .map_err(|err| format!("{err:?}"))?;
        if resp.status_code() != 200 {
            return Err(format!("Unexpected status code: {}", resp.status_code()));
        }
        let resp: GraphqlResponse = resp.json().map_err(|err| format!("{err:?}"))?;
        if let Some(data) = resp.data {
            Ok(data.to_string())
        } else {
            Err(format!("data part is missing, errors: {:?}", resp.errors))
        }
    }
}

const QUERY: &str = r#"
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

    #[test]
    #[ignore]
    fn request_should_succeed() {
        use crate::exports::stargazers::account::account::Guest;
        use crate::Component;
        let username =
            std::env::var("TEST_GITHUB_LOGIN").expect("`TEST_GITHUB_LOGIN` envvar must be set");
        let res = Component::info(username);
        let res = res.unwrap();
        println!("{res}");
    }
}
