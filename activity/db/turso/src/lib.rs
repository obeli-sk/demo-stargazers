mod turso;
use std::time::SystemTime;

use crate::exports::stargazers::db::llm::Guest as LlmGuest;
use crate::exports::stargazers::db::user::Guest as UserGuest;
use exports::stargazers::db::user::{Ordering, Stargazer};
use humantime::format_rfc3339_millis;
use turso::request::{NamedArg, PipelineAction, PipelineRequest, Stmt};
use turso::response::{QueryResult, Response, extract_first_value_from_nth_response};
use turso::{TursoClient, TursoValue};
use wit_bindgen::generate;

pub const ENV_TURSO_TOKEN: &str = "TURSO_TOKEN";
pub const ENV_TURSO_LOCATION: &str = "TURSO_LOCATION";

generate!({ generate_all, additional_derives: [PartialEq] });
pub(crate) struct Component;
export!(Component);

impl LlmGuest for Component {
    fn get_settings_json() -> Result<String, String> {
        let request_body = PipelineRequest {
            requests: vec![
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: "SELECT settings FROM llm WHERE id = 1".to_string(),
                        ..Stmt::default()
                    },
                },
                PipelineAction::Close,
            ],
        };

        let resp = TursoClient::new()?.post_json(&request_body)?;
        if let TursoValue::Text { value: first_value } =
            extract_first_value_from_nth_response(resp, 0)?
        {
            Ok(first_value)
        } else {
            Err("No text value in the first cell".to_string())
        }
    }
}

fn get_user_description_and_star_status(
    login: &str,
    repo: &str,
) -> Result<(Option<String>, bool), String> {
    const PARAM_LOGIN: &str = "login";
    const PARAM_REPO: &str = "repo";

    let request_body = PipelineRequest {
        requests: vec![
            // Check if the user already starred the repo
            PipelineAction::Execute {
                stmt: Stmt {
                    sql: format!(
                        "SELECT 1 FROM stars WHERE user_name = :{PARAM_LOGIN} AND repo_name = :{PARAM_REPO}"
                    ),
                    named_args: vec![
                        NamedArg {
                            name: PARAM_LOGIN,
                            value: TursoValue::Text {
                                value: login.to_string(),
                            },
                        },
                        NamedArg {
                            name: PARAM_REPO,
                            value: TursoValue::Text {
                                value: repo.to_string(),
                            },
                        },
                    ],
                },
            },
            // Select user's description
            PipelineAction::Execute {
                stmt: Stmt {
                    sql: format!("SELECT description FROM users WHERE name = :{PARAM_LOGIN}"),
                    named_args: vec![NamedArg {
                        name: PARAM_LOGIN,
                        value: TursoValue::Text {
                            value: login.to_string(),
                        },
                    }],
                },
            },
            PipelineAction::Close,
        ],
    };

    let resp = TursoClient::new()?.post_json(&request_body)?;
    parse_user_description_and_star_status(resp)
}

fn parse_user_description_and_star_status(
    mut resp: Vec<Response>,
) -> Result<(Option<String>, bool), String> {
    if resp.len() != 3 {
        return Err(format!(
            "unexpected responses count, expected 3, got {}",
            resp.len()
        ));
    }
    resp.pop().unwrap(); // Close
    let description = resp.pop().unwrap(); // Select user's description
    // Check if the user already starred the repo
    let already_starred = match extract_first_value_from_nth_response(resp, 0)? {
        TursoValue::Integer { .. } => true,
        TursoValue::Null => false,
        other => {
            return Err(format!(
                "unexpected data type, expected Integer or Null, got {other:?}"
            ));
        }
    };
    // Get the user's description
    let description = match extract_first_value_from_nth_response(vec![description], 0)? {
        TursoValue::Text { value } => Some(value),
        TursoValue::Null => None,
        other => {
            return Err(format!(
                "unexpected data type, expected Text or Null, got {other:?}"
            ));
        }
    };
    Ok((description, already_starred))
}

impl UserGuest for Component {
    fn add_star_get_description(login: String, repo: String) -> Result<Option<String>, String> {
        const PARAM_LOGIN: &str = "login";
        const PARAM_REPO: &str = "repo";
        const PARAM_NOW: &str = "now";
        let now = SystemTime::now();
        let now = format_rfc3339_millis(now).to_string();

        // If the user already starred the repo, do not update the user's `updated_at`.
        // In fact, do not update anything.
        let (description, already_starred) = get_user_description_and_star_status(&login, &repo)?;
        if !already_starred {
            let request_body = PipelineRequest {
                requests: vec![
                    // Add user: if the user already exists and did not have a star relation to the repo, update the `updated_at` field.
                    PipelineAction::Execute {
                        stmt: Stmt {
                            sql: format!(
                                "INSERT INTO users (name, updated_at) VALUES
                                (:{PARAM_LOGIN}, :{PARAM_NOW}) \
                                ON CONFLICT(name) DO UPDATE \
                                SET updated_at = :{PARAM_NOW}"
                            ),
                            named_args: vec![
                                NamedArg {
                                    name: PARAM_LOGIN,
                                    value: TursoValue::Text {
                                        value: login.clone(),
                                    },
                                },
                                NamedArg {
                                    name: PARAM_NOW,
                                    value: TursoValue::Text { value: now.clone() },
                                },
                            ],
                        },
                    },
                    // Add repo if it does not exist.
                    PipelineAction::Execute {
                        stmt: Stmt {
                            sql: format!(
                                "INSERT INTO repos (name) VALUES (:{PARAM_REPO}) ON CONFLICT DO NOTHING;"
                            ),
                            named_args: vec![NamedArg {
                                name: PARAM_REPO,
                                value: TursoValue::Text {
                                    value: repo.clone(),
                                },
                            }],
                        },
                    },
                    // Add the star relation
                    PipelineAction::Execute {
                        stmt: Stmt {
                            sql: format!(
                                "INSERT INTO stars (user_name, repo_name) VALUES \
                                (:{PARAM_LOGIN}, :{PARAM_REPO}) ON CONFLICT DO NOTHING;"
                            ),
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
                    PipelineAction::Close,
                ],
            };
            TursoClient::new()?.post_json(&request_body)?;
        }
        Ok(description)
    }

    fn remove_star(login: String, repo: String) -> Result<(), String> {
        const PARAM_LOGIN: &str = "login";
        const PARAM_REPO: &str = "repo";

        let request_body = PipelineRequest {
            requests: vec![
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: format!(
                            "DELETE FROM stars WHERE \
                            repo_name = :{PARAM_REPO} AND user_name = :{PARAM_LOGIN};"
                        ),
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
                        sql: format!(
                            "DELETE FROM users WHERE \
                            name = :{PARAM_LOGIN} AND NOT EXISTS \
                            (SELECT 1 FROM stars WHERE user_name = :{PARAM_LOGIN});"
                        ),
                        named_args: vec![NamedArg {
                            name: PARAM_LOGIN,
                            value: TursoValue::Text { value: login },
                        }],
                    },
                },
                PipelineAction::Close,
            ],
        };

        TursoClient::new()?.post_json(&request_body)?;

        Ok(())
    }

    fn update_user_description(username: String, description: String) -> Result<(), String> {
        const PARAM_LOGIN: &str = "login";
        const PARAM_DESCRIPTION: &str = "description";
        const PARAM_NOW: &str = "now";
        let now = SystemTime::now();
        let now = format_rfc3339_millis(now);

        let request_body = PipelineRequest {
            requests: vec![
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: format!(
                            "INSERT INTO users (name, description, updated_at) VALUES \
                            (:{PARAM_LOGIN}, :{PARAM_DESCRIPTION}, :{PARAM_NOW}) \
                            ON CONFLICT(name) DO UPDATE \
                            SET description = excluded.description, updated_at = :{PARAM_NOW};"
                        ),
                        named_args: vec![
                            NamedArg {
                                name: PARAM_LOGIN,
                                value: TursoValue::Text { value: username },
                            },
                            NamedArg {
                                name: PARAM_DESCRIPTION,
                                value: TursoValue::Text { value: description },
                            },
                            NamedArg {
                                name: PARAM_NOW,
                                value: TursoValue::Text {
                                    value: now.to_string(),
                                },
                            },
                        ],
                    },
                },
                PipelineAction::Close,
            ],
        };

        TursoClient::new()?.post_json(&request_body)?;

        Ok(())
    }

    fn list_stargazers(
        last: u8,
        repo: Option<String>,
        ordering: Ordering,
    ) -> Result<Vec<Stargazer>, String> {
        const PARAM_REPO: &str = "repo";
        let ordering = if ordering == Ordering::Descending {
            "DESC"
        } else {
            ""
        };

        let request_body = PipelineRequest {
            requests: vec![
                PipelineAction::Execute {
                    stmt: Stmt {
                        sql: format!(
                            "SELECT u.name as login, u.description, s.repo_name as repo \
                            FROM users u \
                            INNER JOIN stars s ON u.name = s.user_name \
                            {where}
                            ORDER BY u.updated_at {ordering} LIMIT {last}",
                            where = if repo.is_some() {
                                format!("WHERE s.repo_name=:{PARAM_REPO}")
                            } else {
                                String::new()
                            }
                        ),
                        named_args: if let Some(repo) = repo {
                            vec![NamedArg {
                                name: PARAM_REPO,
                                value: TursoValue::Text { value: repo },
                            }]
                        } else {
                            vec![]
                        },
                    },
                },
                PipelineAction::Close,
            ],
        };
        let resp = TursoClient::new()?.post_json(&request_body)?;
        process_resp_list_stargazers(resp)
    }
}

fn process_resp_list_stargazers(
    resp: Vec<turso::response::Response>,
) -> Result<Vec<Stargazer>, String> {
    // must contain two responses: execute and close
    let resp: Vec<QueryResult> = resp
        .into_iter()
        .filter_map(|r| match r {
            Response::Execute {
                result: Some(result),
            } => Some(result),
            _ => None,
        })
        .collect();
    if resp.len() != 1 {
        return Err(format!(
            "unexpected responses count, expected 1, got {}",
            resp.len()
        ));
    }
    let resp = resp.into_iter().next().expect("length already checked");
    let cols: Vec<_> = resp.cols.into_iter().map(|col| col.name).collect();
    if cols != ["login", "description", "repo"] {
        return Err(format!("wrong cols returned: {cols:?}"));
    }
    resp.rows
        .into_iter()
        .map(|mut row| {
            let mut values = row.0.drain(..).map(|value| match value {
                TursoValue::Text { value } => Ok(Some(value.clone())),
                TursoValue::Null => Ok(None),
                other => Err(format!("unexpected type of {other:?}")),
            });

            let login = values
                .next()
                .ok_or_else(|| "missing value".to_string())??
                .ok_or("mandatory value is missing")?;
            let description = values.next().ok_or_else(|| "missing value".to_string())??;
            let repo = values
                .next()
                .ok_or_else(|| "missing value".to_string())??
                .ok_or("mandatory value is missing")?;
            Ok(Stargazer {
                login,
                description,
                repo,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        exports::stargazers::db::user::Stargazer,
        process_resp_list_stargazers,
        turso::{
            TursoValue,
            response::{PipelineResponse, extract_first_value_from_nth_response},
        },
    };

    #[test]
    fn test_parse_settings_from_response() {
        let resp: PipelineResponse = serde_json::from_str(
            r#"
            {
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
            }
            "#,
        )
        .unwrap();
        let resp = resp.ok_responses().unwrap();
        let TursoValue::Text {
            value: settings_json,
        } = extract_first_value_from_nth_response(resp, 0).unwrap()
        else {
            panic!("No text value in the expected coordinates");
        };
        assert_eq!(settings_json, "{\"a\":1}");
    }

    #[test]
    fn process_resp_list_stargazers_should_parse_ok_response() {
        let resp: PipelineResponse = serde_json::from_str(
            r#"
            {
                "baton": null,
                "base_url": null,
                "results": [
                    {
                        "type": "ok",
                        "response": {
                            "type": "execute",
                            "result": {
                                "cols": [
                                    {
                                        "name": "login",
                                        "decltype": "TEXT"
                                    },
                                    {
                                        "name": "description",
                                        "decltype": "TEXT"
                                    },
                                    {
                                        "name": "repo",
                                        "decltype": "TEXT"
                                    }
                                ],
                                "rows": [
                                    [
                                        {
                                            "type": "text",
                                            "value": "u1"
                                        },
                                        {
                                            "type": "text",
                                            "value": "d1"
                                        },
                                        {
                                            "type": "text",
                                            "value": "repo"
                                        }
                                    ],
                                    [
                                        {
                                            "type": "text",
                                            "value": "u2"
                                        },
                                        {
                                            "type": "text",
                                            "value": "d2"
                                        },
                                        {
                                            "type": "text",
                                            "value": "repo"
                                        }
                                    ],
                                    [
                                        {
                                            "type": "text",
                                            "value": "none"
                                        },
                                        {
                                            "type": "null"
                                        },
                                        {
                                            "type": "text",
                                            "value": "repo"
                                        }
                                    ]
                                ],
                                "affected_row_count": 0,
                                "last_insert_rowid": null,
                                "replication_index": "727",
                                "rows_read": 20,
                                "rows_written": 0,
                                "query_duration_ms": 0.132
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
            }
            "#,
        )
        .unwrap();
        let resp = resp.ok_responses().unwrap();
        let resp = process_resp_list_stargazers(resp).unwrap();
        let expected = vec![
            Stargazer {
                login: "u1".to_string(),
                description: Some("d1".to_string()),
                repo: "repo".to_string(),
            },
            Stargazer {
                login: "u2".to_string(),
                description: Some("d2".to_string()),
                repo: "repo".to_string(),
            },
            Stargazer {
                login: "none".to_string(),
                description: None,
                repo: "repo".to_string(),
            },
        ];
        assert_eq!(expected, resp);
    }

    const USER_WITHOUT_DESCRIPTION_AND_WITH_STAR_STATUS_JSON: &str = r#"
    {
      "base_url": null,
      "baton": null,
      "results": [
        {
          "response": {
            "result": {
              "affected_row_count": 0,
              "cols": [{ "decltype": null, "name": "1" }],
              "last_insert_rowid": null,
              "query_duration_ms": 0.163,
              "replication_index": null,
              "rows": [[{ "type": "integer", "value": "1" }]],
              "rows_read": 1,
              "rows_written": 0
            },
            "type": "execute"
          },
          "type": "ok"
        },
        {
          "response": {
            "result": {
              "affected_row_count": 0,
              "cols": [{ "decltype": "TEXT", "name": "description" }],
              "last_insert_rowid": null,
              "query_duration_ms": 0.085,
              "replication_index": null,
              "rows": [[{ "type": "null" }]],
              "rows_read": 1,
              "rows_written": 0
            },
            "type": "execute"
          },
          "type": "ok"
        },
        { "response": { "type": "close" }, "type": "ok" }
      ]
    }
    "#;

    #[test]
    fn user_without_description_and_with_star_status_should_work() {
        let resp: PipelineResponse =
            serde_json::from_str(USER_WITHOUT_DESCRIPTION_AND_WITH_STAR_STATUS_JSON).unwrap();
        let resp = resp.ok_responses().unwrap();
        let (description, already_starred) =
            super::parse_user_description_and_star_status(resp).unwrap();
        assert_eq!(None, description);
        assert!(already_starred);
    }

    const USER_WITH_DESCRIPTION_AND_WITHOUT_STAR_STATUS_JSON: &str = r#"{
      "base_url": null,
      "baton": null,
      "results": [
        {
          "response": {
            "result": {
              "affected_row_count": 0,
              "cols": [{ "decltype": null, "name": "1" }],
              "last_insert_rowid": null,
              "query_duration_ms": 0.167,
              "replication_index": null,
              "rows": [],
              "rows_read": 0,
              "rows_written": 0
            },
            "type": "execute"
          },
          "type": "ok"
        },
        {
          "response": {
            "result": {
              "affected_row_count": 0,
              "cols": [{ "decltype": "TEXT", "name": "description" }],
              "last_insert_rowid": null,
              "query_duration_ms": 0.088,
              "replication_index": null,
              "rows": [[{ "type": "text", "value": "suqbjkasrl" }]],
              "rows_read": 1,
              "rows_written": 0
            },
            "type": "execute"
          },
          "type": "ok"
        },
        { "response": { "type": "close" }, "type": "ok" }
      ]
    }
    "#;

    #[test]
    fn user_with_description_and_without_star_status_should_work() {
        let resp: PipelineResponse =
            serde_json::from_str(USER_WITH_DESCRIPTION_AND_WITHOUT_STAR_STATUS_JSON).unwrap();
        let resp = resp.ok_responses().unwrap();
        let (description, already_starred) =
            super::parse_user_description_and_star_status(resp).unwrap();
        assert_eq!(Some("suqbjkasrl".to_string()), description);
        assert!(!already_starred);
    }

    mod integration {
        use crate::{
            Component, ENV_TURSO_LOCATION, ENV_TURSO_TOKEN,
            exports::stargazers::db::{
                llm::Guest as _,
                user::{Guest as _, Ordering, Stargazer},
            },
            turso::{
                TursoClient, TursoValue,
                request::{NamedArg, PipelineAction, PipelineRequest, Stmt},
                response::{QueryResult, Response},
            },
        };

        fn set_up() {
            let test_token =
                std::env::var(format!("TEST_{ENV_TURSO_TOKEN}")).unwrap_or_else(|_| {
                    panic!("TEST_{ENV_TURSO_TOKEN} must be set as an environment variable")
                });
            unsafe { std::env::set_var(ENV_TURSO_TOKEN, test_token) };

            let test_location =
                std::env::var(format!("TEST_{ENV_TURSO_LOCATION}")).unwrap_or_else(|_| {
                    panic!("TEST_{ENV_TURSO_LOCATION} must be set as an environment variable")
                });
            unsafe { std::env::set_var(ENV_TURSO_LOCATION, test_location) };
        }

        fn random_string() -> String {
            use rand::SeedableRng;
            let mut rng = rand::rngs::SmallRng::from_os_rng();
            (0..10)
                .map(|_| rand::Rng::random_range(&mut rng, b'a'..=b'z') as char)
                .collect()
        }

        fn delete_from(table: &str) {
            println!("DELETE FROM {table}");
            TursoClient::new()
                .unwrap()
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
                .unwrap()
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
            let Response::Execute {
                result: Some(QueryResult { rows, .. }),
            } = first_result
            else {
                panic!("Wrong response {first_result:?}");
            };
            rows.into_iter()
                .map(|row| {
                    row.0
                        .into_iter()
                        .map(|cell| match cell {
                            TursoValue::Text { value } => Some(value),
                            TursoValue::Null => None,
                            other => panic!("wrong data type {other:?}"),
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        }

        #[test]
        #[ignore]
        fn get_settings_json_should_succeed() {
            set_up();
            const PARAM_SETTINGS: &str = "settings";
            const EXPECTED: &str = r#"{"a":1}"#;
            delete_from("llm");
            // Create the row
            TursoClient::new()
                .unwrap()
                .post_json(&PipelineRequest {
                    requests: vec![
                        PipelineAction::Execute {
                            stmt: Stmt {
                                sql: format!(
                                    "INSERT INTO llm (id, settings) VALUES (1, :{PARAM_SETTINGS});"
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

        #[test]
        #[ignore]
        fn user_update_should_create_the_user() {
            set_up();
            delete_from("users");
            let login = random_string();
            let description = random_string();
            println!("Creating user `{login}` with description `{description}`");
            Component::update_user_description(login.clone(), description.clone()).unwrap();
            // Check user
            assert_eq!(vec![login.clone()], select_name("users"));

            println!("Deleting the user");
            Component::remove_star(login, "any".to_string()).unwrap();

            assert_eq!(Vec::<String>::new(), select_name("users"));
        }

        #[test]
        #[ignore]
        fn link_and_update_should_work_on_the_same_user() {
            set_up();
            delete_from("users");
            delete_from("repos");
            delete_from("stars");
            let login = random_string();
            let repo = random_string();
            println!("Creating user `{login}` and repo `{repo}`");
            let description =
                Component::add_star_get_description(login.clone(), repo.clone()).unwrap();
            assert!(description.is_none());

            let description = random_string();
            println!("Updating user `{login}` with description `{description}`");
            Component::update_user_description(login.clone(), description.clone()).unwrap();
            // Check the user and description directly in the database.
            assert_eq!(
                vec![vec![Some(login), Some(description)]],
                select("users", &["name", "description"])
            );
        }

        #[test]
        #[ignore]
        fn list_stargazers_should_work() {
            set_up();
            delete_from("users");
            delete_from("repos");
            delete_from("stars");
            let repo1 = random_string();
            let repo2 = random_string();

            let insert = |stargazer: &Stargazer| {
                Component::add_star_get_description(
                    stargazer.login.clone(),
                    stargazer.repo.clone(),
                )
                .unwrap();
                if let Some(description) = stargazer.description.clone() {
                    Component::update_user_description(stargazer.login.clone(), description)
                        .unwrap();
                }
            };

            let s_old_repo1 = Stargazer {
                login: random_string(),
                description: Some(random_string()),
                repo: repo1.clone(),
            };
            insert(&s_old_repo1);

            let s_new_repo1 = Stargazer {
                login: random_string(),
                description: None,
                repo: repo1.clone(),
            };
            insert(&s_new_repo1);

            let s_repo2 = Stargazer {
                login: random_string(),
                description: None,
                repo: repo2.clone(),
            };
            insert(&s_repo2);

            // get all 3 ordered by latest first
            let actual = Component::list_stargazers(3, None, Ordering::Descending).unwrap();
            assert_eq!(
                vec![s_repo2.clone(), s_new_repo1.clone(), s_old_repo1.clone()],
                actual
            );
            // Get only the latest from repo1
            let actual =
                Component::list_stargazers(1, Some(repo1.clone()), Ordering::Descending).unwrap();
            assert_eq!(vec![s_new_repo1.clone()], actual);
            // Get the oldest only from all repos
            let actual = Component::list_stargazers(1, None, Ordering::Ascending).unwrap();
            assert_eq!(vec![s_old_repo1.clone()], actual);
        }

        #[test]
        #[ignore]
        fn list_stargazers_should_be_updated_on_description_update() {
            set_up();
            delete_from("users");
            delete_from("repos");
            delete_from("stars");

            let insert = |stargazer: &Stargazer| {
                Component::add_star_get_description(
                    stargazer.login.clone(),
                    stargazer.repo.clone(),
                )
                .unwrap();
                if let Some(description) = stargazer.description.clone() {
                    Component::update_user_description(stargazer.login.clone(), description)
                        .unwrap();
                }
            };
            let mut s_old = Stargazer {
                login: random_string(),
                description: None,
                repo: random_string(),
            };
            insert(&s_old);
            let s_new = Stargazer {
                login: random_string(),
                description: None,
                repo: random_string(),
            };
            insert(&s_new);
            let actual = Component::list_stargazers(2, None, Ordering::Descending).unwrap();
            assert_eq!(vec![s_new.clone(), s_old.clone()], actual);
            // Update the description of s_old to change its `updated_at`
            s_old.description = Some(random_string());
            Component::update_user_description(
                s_old.login.clone(),
                s_old.description.clone().expect("description was just set"),
            )
            .unwrap();
            // Get the reordered list
            let actual = Component::list_stargazers(2, None, Ordering::Descending).unwrap();
            assert_eq!(vec![s_old, s_new], actual);
        }

        #[test]
        #[ignore]
        fn link_after_update_should_return_the_description() {
            set_up();
            delete_from("users");
            delete_from("repos");
            delete_from("stars");

            let login = random_string();
            let description = random_string();
            println!("Creating user `{login}` with description `{description}`");
            Component::update_user_description(login.clone(), description.clone()).unwrap();

            let repo = random_string();
            println!("Starring repo `{repo}`");
            let actual_description =
                Component::add_star_get_description(login.clone(), repo.clone()).unwrap();

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
            set_up();
            delete_from("users");
            delete_from("repos");
            delete_from("stars");
            let login = random_string();
            let repo = random_string();
            println!("Creating user `{login}` and repo `{repo}`");
            Component::add_star_get_description(login.clone(), repo.clone()).unwrap();
            // Check that data is inserted into `users`, `repos`, `stars`.
            assert_eq!(vec![login.clone()], select_name("users"));
            assert_eq!(vec![repo.clone()], select_name("repos"));
            assert_eq!(
                vec![vec![Some(login.clone()), Some(repo.clone())]],
                select("stars", &["user_name", "repo_name"])
            );
            println!("Deleting the user");
            Component::remove_star(login, repo.clone()).unwrap();
            // Check that only `repos` is not empty,
            assert_eq!(Vec::<String>::new(), select_name("users"));
            assert_eq!(vec![repo.clone()], select_name("repos"));
            assert_eq!(
                Vec::<Vec<Option<String>>>::new(),
                select("stars", &["user_name", "repo_name"])
            );
        }

        #[test]
        #[ignore]
        fn user_updated_at_should_not_change_if_already_starred() {
            set_up();
            delete_from("users");
            delete_from("repos");
            delete_from("stars");

            let login = random_string();
            let repo = random_string();
            println!("Creating user `{login}` and repo `{repo}`");
            Component::add_star_get_description(login.clone(), repo.clone()).unwrap();

            // Capture the initial updated_at timestamp
            let initial_updated_at: String = select("users", &["updated_at"])
                .into_iter()
                .next()
                .expect("user should exist")
                .into_iter()
                .next()
                .expect("updated_at should be present")
                .expect("updated_at should not be null");

            // Star the same repo again with the same user
            Component::add_star_get_description(login.clone(), repo.clone()).unwrap();

            // Capture the updated updated_at timestamp
            let updated_updated_at: String = select("users", &["updated_at"])
                .into_iter()
                .next()
                .expect("user should exist")
                .into_iter()
                .next()
                .expect("updated_at should be present")
                .expect("updated_at should not be null");

            // Verify that the updated_at timestamp has not changed
            assert_eq!(initial_updated_at, updated_updated_at);
        }
    }
}
