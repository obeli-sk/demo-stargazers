use crate::exports::stargazers::db::llm::Guest as LlmGuest;
use crate::exports::stargazers::db::user::Guest as UserGuest;
use request::{NamedArg, PipelineAction, PipelineRequest, Stmt, TursoValue};
use response::{extract_first_cell_from_nth_response, PipelineResponse};
use waki::Client;
use wit_bindgen::generate;

generate!({ generate_all });
pub(crate) struct Component;
export!(Component);

pub mod request {
    use serde::Serialize;
    #[derive(Serialize)]
    pub struct PipelineRequest {
        pub requests: Vec<PipelineAction>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    #[serde(tag = "type", rename_all = "lowercase")]
    pub enum PipelineAction {
        Execute { stmt: Stmt },
        Close,
    }

    #[derive(Debug, Serialize, Default, PartialEq)]
    pub struct Stmt {
        pub sql: String,
        pub named_args: Vec<NamedArg>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    pub struct NamedArg {
        pub name: &'static str,
        pub value: TursoValue,
    }

    #[derive(Debug, Serialize, PartialEq)]
    #[serde(tag = "type", rename_all = "lowercase")]
    pub enum TursoValue {
        Text { value: String }, // TODO: Cow
    }
}

pub mod response {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct PipelineResponse {
        pub results: Vec<ResponseResult>,
    }

    impl PipelineResponse {
        pub fn ok_responses(self) -> Result<Vec<Option<Response>>, String> {
            self.results
                .into_iter()
                .map(|res| match res {
                    ResponseResult::Ok { response } => Ok(response),
                    ResponseResult::Error { error } => {
                        Err(format!("Got reponse result error {error:?}"))
                    }
                })
                .collect::<Result<_, _>>()
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type", rename_all = "lowercase")]
    pub enum ResponseResult {
        Ok { response: Option<Response> },
        Error { error: ResponseResultError },
    }

    #[derive(Debug, Deserialize)]
    pub struct ResponseResultError {
        pub message: String,
        pub code: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type", rename_all = "lowercase")]
    pub enum Response {
        Execute { result: Option<QueryResult> },
        Close,
    }

    #[derive(Debug, Deserialize)]
    pub struct QueryResult {
        pub rows: Vec<QueryRow>,
    }

    #[derive(Debug, Deserialize)]
    pub struct QueryRow(pub Vec<Cell>);

    #[derive(Debug, Deserialize)]
    pub struct Cell {
        #[serde(rename = "type")]
        pub cell_type: String,
        pub value: Option<String>,
    }

    /// Extracts the first [`Cell`] from the first row of the n-th response.
    pub fn extract_first_cell_from_nth_response(
        responses: Vec<Option<Response>>,
        n: usize,
    ) -> Result<Cell, String> {
        let query_result = match responses.into_iter().nth(n) {
            Some(Some(Response::Execute {
                result: Some(query_result),
            })) => query_result,
            _ => return Err("First response result is unexpected".to_string()),
        };
        let first_row = match query_result.rows.into_iter().next() {
            Some(row) => row,
            None => return Err("No rows in the result".to_string()),
        };
        let first_cell = match first_row.0.into_iter().next() {
            Some(cell) => cell,
            None => return Err("No cell in the first row".to_string()),
        };
        Ok(first_cell)
    }
}

struct TursoClient {
    url: String,
    token: String,
    client: Client,
}

impl TursoClient {
    fn new() -> Self {
        let token = std::env::var("TURSO_TOKEN")
            .expect("TURSO_TOKEN must be set as an environment variable");
        let turso_location = std::env::var("TURSO_LOCATION")
            .expect("TURSO_LOCATION must be set as an environment variable");

        let client = Client::new();
        let url = format!("https://{turso_location}/v2/pipeline");
        Self { url, token, client }
    }

    fn bearer(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn post(&self) -> waki::RequestBuilder {
        self.client
            .post(&self.url)
            .header("Authorization", self.bearer())
            .header("Content-Type", "application/json")
    }

    fn post_json(
        &self,
        request: &PipelineRequest,
    ) -> Result<Vec<Option<response::Response>>, String> {
        assert_eq!(
            Some(&PipelineAction::Close),
            request.requests.last(),
            "last action must be close"
        );
        let resp = self
            .post()
            .json(request)
            .send()
            .map_err(|err| format!("Failed to send request: {:?}", err))?;
        if resp.status_code() != 200 {
            return Err(format!("Unexpected status code: {}", resp.status_code()));
        }
        let resp: PipelineResponse = resp
            .json()
            .map_err(|err| format!("Failed to parse response: {err:?}"))?;

        // Make sure there are no errors
        resp.ok_responses()
    }
}

impl LlmGuest for Component {
    fn get_settings_json() -> Result<String, String> {
        let request_body = PipelineRequest {
            requests: vec![
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: "SELECT settings FROM llm WHERE id = 0".to_string(),
                        ..Stmt::default()
                    },
                },
                PipelineAction::Close,
            ],
        };

        let resp = TursoClient::new().post_json(&request_body)?;
        extract_first_cell_from_nth_response(resp, 0)?
            .value
            .ok_or_else(|| "No value in the first cell".to_string())
    }
}

impl UserGuest for Component {
    fn link_get_description(login: String, repo: String) -> Result<Option<String>, String> {
        const PARAM_LOGIN: &str = "login";
        const PARAM_REPO: &str = "repo";

        let request_body = PipelineRequest {
            requests: vec![
                // Add user
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: format!("INSERT INTO users (name) VALUES (:{PARAM_LOGIN}) ON CONFLICT DO NOTHING;"),
                        named_args: vec![
                            NamedArg {
                                name: PARAM_LOGIN,
                                value: TursoValue::Text {
                                    value: login.clone(),
                                },
                            },
                        ],
                    },
                },
                // Add repo
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: format!("INSERT INTO repos (name) VALUES (:{PARAM_REPO}) ON CONFLICT DO NOTHING;"),
                        named_args: vec![
                            NamedArg {
                                name: PARAM_REPO,
                                value: TursoValue::Text {
                                    value: repo.clone(),
                                },
                            },
                        ],
                    },
                },
                // Add the star relation
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: format!("INSERT INTO stars (user_name, repo_name) VALUES (:{PARAM_LOGIN}, :{PARAM_REPO}) ON CONFLICT DO NOTHING;"),
                        named_args: vec![
                            NamedArg {
                                name: PARAM_LOGIN,
                                value: TursoValue::Text {
                                    value: login.clone(),
                                },
                            },
                            NamedArg {
                                name: PARAM_REPO,
                                value: TursoValue::Text {
                                    value: repo.clone(),
                                },
                            },

                        ],
                    },
                },
                // Select user's description
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: format!("SELECT description FROM users WHERE name = :{PARAM_LOGIN}"),
                        named_args: vec![
                            NamedArg {
                                name: PARAM_LOGIN,
                                value: TursoValue::Text {
                                    value: login.clone(),
                                },
                            },
                        ]
                    },
                },
                PipelineAction::Close,
            ],
        };

        let resp = TursoClient::new().post_json(&request_body)?;
        Ok(extract_first_cell_from_nth_response(resp, 3)?.value)
    }

    fn unlink(login: String, repo: String) -> Result<(), String> {
        const PARAM_LOGIN: &str = "login";
        const PARAM_REPO: &str = "repo";

        let request_body = PipelineRequest {
            requests: vec![
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: format!("DELETE FROM stars WHERE repo_name = :{PARAM_REPO} AND user_name = :{PARAM_LOGIN};"),
                        named_args: vec![
                            NamedArg {
                                name: PARAM_REPO,
                                value: TursoValue::Text {
                                    value: repo.clone(),
                                },
                            },
                            NamedArg {
                                name: PARAM_LOGIN,
                                value: TursoValue::Text {
                                    value: login.clone(),
                                },
                            },
                        ],
                    },
                },
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: format!("DELETE FROM users WHERE name = :{PARAM_LOGIN} AND NOT EXISTS (SELECT 1 FROM stars WHERE user_name = :{PARAM_LOGIN});"),
                        named_args: vec![
                            NamedArg {
                                name: PARAM_LOGIN,
                                value: TursoValue::Text {
                                    value: login,
                                },
                            },
                        ],
                    },
                },
                PipelineAction::Close,
            ],
        };

        TursoClient::new().post_json(&request_body)?;

        Ok(())
    }

    fn user_update(username: String, description: String) -> Result<(), String> {
        const PARAM_LOGIN: &str = "login";
        const PARAM_DESCRIPTION: &str = "description";

        let request_body = PipelineRequest {
            requests: vec![
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: format!("INSERT INTO users (name, description) VALUES (:{PARAM_LOGIN}, :{PARAM_DESCRIPTION}) ON CONFLICT(name) DO UPDATE SET description = excluded.description;"),
                        named_args: vec![
                            NamedArg {
                                name: PARAM_LOGIN,
                                value: TursoValue::Text{value: username},
                            },
                            NamedArg {
                                name: PARAM_DESCRIPTION,
                                value: TursoValue::Text{value: description},
                            },
                        ],
                    },
                },
                PipelineAction::Close,
            ],
        };

        TursoClient::new().post_json(&request_body)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use response::{QueryResult, Response};
    use serde_json::json;

    fn random_string() -> String {
        use rand::SeedableRng;
        let mut rng = rand::rngs::SmallRng::from_entropy();
        (0..10)
            .map(|_| (rand::Rng::gen_range(&mut rng, b'a'..=b'z') as char))
            .collect()
    }

    #[test]
    fn test_parse_settings_from_response() {
        let resp: PipelineResponse = serde_json::from_value(json!({
            "baton": null,
            "base_url": null,
            "results": [
                {
                    "type": "ok",
                    "response": {
                        "type": "execute",
                        "result": {
                            "cols": [{"name": "settings", "decltype": "TEXT"}],
                            "rows": [
                                [{"type": "text", "value": "{\"a\":1}"}]
                            ],
                            "affected_row_count": 0,
                            "last_insert_rowid": null,
                            "replication_index": "14",
                            "rows_read": 1,
                            "rows_written": 0,
                            "query_duration_ms": 0.054
                        }
                    }
                },
                {
                    "type": "ok",
                    "response": {
                        "type": "close"
                    }
                }
            ]
        }))
        .unwrap();
        let resp = resp.ok_responses().unwrap();
        let settings_json = extract_first_cell_from_nth_response(resp, 0)
            .unwrap()
            .value
            .expect("value must be sent");
        assert_eq!(settings_json, "{\"a\":1}");
    }

    #[test]
    #[ignore]
    fn get_settings_json_should_succeed() {
        const PARAM_SETTINGS: &str = "settings";
        const EXPECTED: &str = r#"{"a":1}"#;
        delete_from("llm");
        // Create the row
        TursoClient::new()
            .post_json(&PipelineRequest {
                requests: vec![
                    PipelineAction::Execute {
                        stmt: Stmt {
                            sql: format!(
                                "INSERT INTO llm (id, settings) VALUES (0, :{PARAM_SETTINGS});"
                            ),
                            named_args: vec![NamedArg {
                                name: PARAM_SETTINGS,
                                value: TursoValue::Text {
                                    value: EXPECTED.to_string(),
                                },
                            }],
                        },
                    },
                    PipelineAction::Close,
                ],
            })
            .unwrap();
        let settings_json = Component::get_settings_json().unwrap();
        assert_eq!(EXPECTED, settings_json);
        // Make a SELECT just to make sure it is stored where we expect.
        assert_eq!(
            vec![EXPECTED],
            select_single_non_null_column("llm", "settings")
        );
    }

    fn delete_from(table: &str) {
        println!("DELETE FROM {table}");
        TursoClient::new()
            .post_json(&PipelineRequest {
                requests: vec![
                    // Add user
                    PipelineAction::Execute {
                        stmt: Stmt {
                            sql: format!("DELETE FROM {table};"),
                            ..Stmt::default()
                        },
                    },
                    PipelineAction::Close,
                ],
            })
            .unwrap();
    }

    fn select_name(table: &str) -> Vec<String> {
        select_single_non_null_column(table, "name")
    }

    fn select_single_non_null_column(table: &str, column: &str) -> Vec<String> {
        select(table, &[column])
            .into_iter()
            .map(|row| {
                // There must be one string per row
                assert_eq!(1, row.len());
                row.into_iter().next().unwrap().expect("must not be None")
            })
            .collect()
    }

    fn select(table: &str, params: &[&str]) -> Vec<Vec<Option<String>>> {
        let sql = format!("SELECT {} FROM {table}", params.join(","));
        println!("{sql}");
        let resp = TursoClient::new()
            .post_json(&PipelineRequest {
                requests: vec![
                    PipelineAction::Execute {
                        stmt: Stmt {
                            sql,
                            ..Stmt::default()
                        },
                    },
                    PipelineAction::Close,
                ],
            })
            .unwrap();
        assert_eq!(2, resp.len());
        let first_result = resp.into_iter().next().unwrap();
        let Some(Response::Execute {
            result: Some(QueryResult { rows }),
        }) = first_result
        else {
            panic!("Wrong response {first_result:?}");
        };
        rows.into_iter()
            .map(|row| row.0.into_iter().map(|cell| cell.value).collect::<Vec<_>>())
            .collect()
    }

    #[test]
    #[ignore]
    fn user_update_should_create_the_user() {
        delete_from("users");
        let login = random_string();
        let description = random_string();
        println!("Creating user `{login}` with description `{description}`");
        Component::user_update(login.clone(), description.clone()).unwrap();
        // Check user
        assert_eq!(vec![login.clone()], select_name("users"));

        println!("Deleting the user");
        Component::unlink(login, "any".to_string()).unwrap();

        assert_eq!(Vec::<String>::new(), select_name("users"));
    }

    #[test]
    #[ignore]
    fn link_and_update_should_work_on_the_same_user() {
        delete_from("users");
        delete_from("repos");
        delete_from("stars");
        let login = random_string();
        let repo = random_string();
        println!("Creating user `{login}` and repo `{repo}`");
        Component::link_get_description(login.clone(), repo.clone()).unwrap();

        let description = random_string();
        println!("Updating user `{login}` with description `{description}`");
        Component::user_update(login.clone(), description.clone()).unwrap();
        // Check the user and description directly in the database.
        assert_eq!(
            vec![vec![Some(login), Some(description)]],
            select("users", &["name", "description"])
        );
    }

    #[test]
    #[ignore]
    fn link_after_update_should_return_the_description() {
        delete_from("users");
        delete_from("repos");
        delete_from("stars");

        let login = random_string();
        let description = random_string();
        println!("Creating user `{login}` with description `{description}`");
        Component::user_update(login.clone(), description.clone()).unwrap();

        let repo = random_string();
        println!("Starring repo `{repo}`");
        let actual_description =
            Component::link_get_description(login.clone(), repo.clone()).unwrap();

        assert_eq!(Some(&description), actual_description.as_ref());

        // Check the user and description directly in the database.
        assert_eq!(
            vec![vec![Some(login), Some(description)]],
            select("users", &["name", "description"])
        );
    }

    #[test]
    #[ignore]
    fn user_link_unlink_should_retain_the_repo_only() {
        delete_from("users");
        delete_from("repos");
        delete_from("stars");
        let login = random_string();
        let repo = random_string();
        println!("Creating user `{login}` and repo `{repo}`");
        Component::link_get_description(login.clone(), repo.clone()).unwrap();
        // Check that data is inserted into `users`, `repos`, `stars`.
        assert_eq!(vec![login.clone()], select_name("users"));
        assert_eq!(vec![repo.clone()], select_name("repos"));
        assert_eq!(
            vec![vec![Some(login.clone()), Some(repo.clone())]],
            select("stars", &["user_name", "repo_name"])
        );
        println!("Deleting the user");
        Component::unlink(login, repo.clone()).unwrap();
        // Check that only `repos` is not empty,
        assert_eq!(Vec::<String>::new(), select_name("users"));
        assert_eq!(vec![repo.clone()], select_name("repos"));
        assert_eq!(
            Vec::<Vec<Option<String>>>::new(),
            select("stars", &["user_name", "repo_name"])
        );
    }
}