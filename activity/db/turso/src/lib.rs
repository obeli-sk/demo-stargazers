#[cfg_attr(not(test), expect(dead_code, reason = "used by the upcoming migrations"))]
mod split;
mod turso;

use generated::export;
use generated::exports::stargazers::db::llm::Guest as LlmGuest;
use generated::exports::stargazers::db::user::Guest as UserGuest;
use generated::exports::stargazers::db::user::{Ordering, Stargazer};
use humantime::format_rfc3339_millis;
use std::time::SystemTime;
use turso::request::Stmt;
use turso::response::{QueryResult, first_value};
use turso::{TursoClient, TursoValue};
use wstd::runtime::block_on;

pub const ENV_TURSO_TOKEN: &str = "TURSO_TOKEN";
pub const ENV_TURSO_LOCATION: &str = "TURSO_LOCATION";

// `:now`, or 1ms after the latest `updated_at` when not later, so `ORDER BY updated_at` has no ties.
const NEXT_UPDATED_AT: &str = "max(:now, coalesce(strftime('%Y-%m-%dT%H:%M:%fZ', \
    (SELECT max(updated_at) FROM users), '+0.001 seconds'), ''))";

mod generated {
    #![allow(clippy::empty_line_after_outer_attr)]
    include!(concat!(env!("OUT_DIR"), "/any.rs"));
}

struct Component;
export!(Component with_types_in generated);

fn transaction(statements: Vec<Stmt>) -> Result<Vec<QueryResult>, String> {
    block_on(async move { TursoClient::new()?.transaction(statements).await })
        .map_err(|err| err.to_string())
}

impl LlmGuest for Component {
    fn get_settings_json() -> Result<String, String> {
        let resp = transaction(vec![Stmt::new("SELECT settings FROM llm WHERE id = 1")])?;
        let resp = resp.into_iter().next().ok_or("missing result")?;
        if let TursoValue::Text { value: first_value } = first_value(resp)? {
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
    let resp = transaction(vec![
        // Check if the user already starred the repo
        Stmt::new("SELECT 1 FROM stars WHERE user_name = :login AND repo_name = :repo")
            .arg("login", login)
            .arg("repo", repo),
        // Select user's description
        Stmt::new("SELECT description FROM users WHERE name = :login").arg("login", login),
    ])?;
    parse_user_description_and_star_status(resp)
}

fn parse_user_description_and_star_status(
    resp: Vec<QueryResult>,
) -> Result<(Option<String>, bool), String> {
    let Ok([star, description]) = <[QueryResult; 2]>::try_from(resp) else {
        return Err("unexpected responses count, expected 2".to_string());
    };
    // Check if the user already starred the repo
    let already_starred = match first_value(star)? {
        TursoValue::Integer { .. } => true,
        TursoValue::Null => false,
        other => {
            return Err(format!(
                "unexpected data type, expected Integer or Null, got {other:?}"
            ));
        }
    };
    // Get the user's description
    let description = match first_value(description)? {
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
        let now = format_rfc3339_millis(SystemTime::now()).to_string();

        // If the user already starred the repo, do not update the user's `updated_at`.
        // In fact, do not update anything.
        let (description, already_starred) = get_user_description_and_star_status(&login, &repo)?;
        if !already_starred {
            transaction(vec![
                // Add user: if the user already exists and did not have a star relation to the repo, update the `updated_at` field.
                Stmt::new(format!(
                    "INSERT INTO users (name, updated_at) VALUES (:login, {NEXT_UPDATED_AT}) \
                    ON CONFLICT(name) DO UPDATE SET updated_at = excluded.updated_at"
                ))
                .arg("login", &login)
                .arg("now", now),
                // Add repo if it does not exist.
                Stmt::new("INSERT INTO repos (name) VALUES (:repo) ON CONFLICT DO NOTHING")
                    .arg("repo", &repo),
                // Add the star relation
                Stmt::new(
                    "INSERT INTO stars (user_name, repo_name) VALUES (:login, :repo) \
                    ON CONFLICT DO NOTHING",
                )
                .arg("login", login)
                .arg("repo", repo),
            ])?;
        }
        Ok(description)
    }

    fn remove_star(login: String, repo: String) -> Result<(), String> {
        transaction(vec![
            Stmt::new("DELETE FROM stars WHERE repo_name = :repo AND user_name = :login")
                .arg("repo", repo)
                .arg("login", &login),
            Stmt::new(
                "DELETE FROM users WHERE name = :login AND NOT EXISTS \
                (SELECT 1 FROM stars WHERE user_name = :login)",
            )
            .arg("login", login),
        ])?;
        Ok(())
    }

    fn update_user_description(username: String, description: String) -> Result<(), String> {
        let now = format_rfc3339_millis(SystemTime::now()).to_string();
        transaction(vec![
            Stmt::new(format!(
                "INSERT INTO users (name, description, updated_at) VALUES \
                (:login, :description, {NEXT_UPDATED_AT}) \
                ON CONFLICT(name) DO UPDATE \
                SET description = excluded.description, updated_at = excluded.updated_at"
            ))
            .arg("login", username)
            .arg("description", description)
            .arg("now", now),
        ])?;
        Ok(())
    }

    fn list_stargazers(
        last: u8,
        repo: Option<String>,
        ordering: Ordering,
    ) -> Result<Vec<Stargazer>, String> {
        let ordering = if ordering == Ordering::Descending {
            "DESC"
        } else {
            ""
        };
        let stmt = Stmt::new(format!(
            "SELECT u.name as login, u.description, s.repo_name as repo \
            FROM users u \
            INNER JOIN stars s ON u.name = s.user_name \
            {where}
            ORDER BY u.updated_at {ordering}, s.rowid {ordering} LIMIT {last}",
            where = if repo.is_some() {
                "WHERE s.repo_name = :repo"
            } else {
                ""
            }
        ));
        let stmt = match repo {
            Some(repo) => stmt.arg("repo", repo),
            None => stmt,
        };
        process_resp_list_stargazers(transaction(vec![stmt])?)
    }
}

fn process_resp_list_stargazers(resp: Vec<QueryResult>) -> Result<Vec<Stargazer>, String> {
    let Ok([resp]) = <[QueryResult; 1]>::try_from(resp) else {
        return Err("unexpected responses count, expected 1".to_string());
    };
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
        generated::exports::stargazers::db::user::Stargazer,
        process_resp_list_stargazers,
        turso::{
            TursoValue,
            response::{QueryResult, first_value},
        },
    };

    #[test]
    fn test_parse_settings_from_response() {
        let resp: Vec<QueryResult> = serde_json::from_str(
            r#"[
  {
    "cols": [
      {
        "name": "settings",
        "decltype": "TEXT"
      }
    ],
    "rows": [
      [
        {
          "type": "text",
          "value": "{\"a\":1}"
        }
      ]
    ],
    "affected_row_count": 0,
    "last_insert_rowid": null,
    "replication_index": "14",
    "rows_read": 1,
    "rows_written": 0,
    "query_duration_ms": 0.054
  }
]"#,
        )
        .unwrap();
        let TursoValue::Text {
            value: settings_json,
        } = first_value(resp.into_iter().next().unwrap()).unwrap()
        else {
            panic!("No text value in the expected coordinates");
        };
        assert_eq!(settings_json, "{\"a\":1}");
    }

    #[test]
    fn process_resp_list_stargazers_should_parse_ok_response() {
        let resp: Vec<QueryResult> = serde_json::from_str(
            r#"[
  {
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
]"#,
        )
        .unwrap();
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

    const USER_WITHOUT_DESCRIPTION_AND_WITH_STAR_STATUS_JSON: &str = r#"[
  {
    "affected_row_count": 0,
    "cols": [
      {
        "decltype": null,
        "name": "1"
      }
    ],
    "last_insert_rowid": null,
    "query_duration_ms": 0.163,
    "replication_index": null,
    "rows": [
      [
        {
          "type": "integer",
          "value": "1"
        }
      ]
    ],
    "rows_read": 1,
    "rows_written": 0
  },
  {
    "affected_row_count": 0,
    "cols": [
      {
        "decltype": "TEXT",
        "name": "description"
      }
    ],
    "last_insert_rowid": null,
    "query_duration_ms": 0.085,
    "replication_index": null,
    "rows": [
      [
        {
          "type": "null"
        }
      ]
    ],
    "rows_read": 1,
    "rows_written": 0
  }
]"#;

    #[test]
    fn user_without_description_and_with_star_status_should_work() {
        let resp: Vec<QueryResult> =
            serde_json::from_str(USER_WITHOUT_DESCRIPTION_AND_WITH_STAR_STATUS_JSON).unwrap();
        let (description, already_starred) =
            super::parse_user_description_and_star_status(resp).unwrap();
        assert_eq!(None, description);
        assert!(already_starred);
    }

    const USER_WITH_DESCRIPTION_AND_WITHOUT_STAR_STATUS_JSON: &str = r#"[
  {
    "affected_row_count": 0,
    "cols": [
      {
        "decltype": null,
        "name": "1"
      }
    ],
    "last_insert_rowid": null,
    "query_duration_ms": 0.167,
    "replication_index": null,
    "rows": [],
    "rows_read": 0,
    "rows_written": 0
  },
  {
    "affected_row_count": 0,
    "cols": [
      {
        "decltype": "TEXT",
        "name": "description"
      }
    ],
    "last_insert_rowid": null,
    "query_duration_ms": 0.088,
    "replication_index": null,
    "rows": [
      [
        {
          "type": "text",
          "value": "suqbjkasrl"
        }
      ]
    ],
    "rows_read": 1,
    "rows_written": 0
  }
]"#;

    #[test]
    fn user_with_description_and_without_star_status_should_work() {
        let resp: Vec<QueryResult> =
            serde_json::from_str(USER_WITH_DESCRIPTION_AND_WITHOUT_STAR_STATUS_JSON).unwrap();
        let (description, already_starred) =
            super::parse_user_description_and_star_status(resp).unwrap();
        assert_eq!(Some("suqbjkasrl".to_string()), description);
        assert!(!already_starred);
    }

    mod integration {
        use wstd::runtime::block_on;

        use crate::{
            Component, ENV_TURSO_LOCATION, ENV_TURSO_TOKEN,
            generated::exports::stargazers::db::llm::Guest as _,
            generated::exports::stargazers::db::user::{Guest as _, Ordering, Stargazer},
            turso::{
                TursoClient, TursoValue,
                request::Stmt,
                response::QueryResult,
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

        async fn delete_from(table: &str) {
            println!("DELETE FROM {table}");
            TursoClient::new()
                .unwrap()
                .transaction(vec![Stmt::new(format!("DELETE FROM {table}"))])
                .await
                .unwrap();
        }

        async fn select_name(table: &str) -> Vec<String> {
            select_single_non_null_column(table, "name").await
        }

        async fn select_single_non_null_column(table: &str, column: &str) -> Vec<String> {
            select(table, &[column])
                .await
                .into_iter()
                .map(|row| {
                    // There must be one string per row
                    assert_eq!(1, row.len());
                    row.into_iter().next().unwrap().expect("must not be None")
                })
                .collect()
        }

        async fn select(table: &str, params: &[&str]) -> Vec<Vec<Option<String>>> {
            let sql = format!("SELECT {} FROM {table}", params.join(","));
            println!("{sql}");
            let resp = TursoClient::new()
                .unwrap()
                .transaction(vec![Stmt::new(sql)])
                .await
                .unwrap();
            let Ok([QueryResult { rows, .. }]) = <[QueryResult; 1]>::try_from(resp) else {
                panic!("expected one result");
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
            block_on(async move {
                delete_from("llm").await;
            });
            // Create the row
            block_on(async move {
                TursoClient::new()
                    .unwrap()
                    .transaction(vec![
                        Stmt::new(format!(
                            "INSERT INTO llm (id, settings) VALUES (1, :{PARAM_SETTINGS})"
                        ))
                        .arg(PARAM_SETTINGS, EXPECTED),
                    ])
                    .await
                    .unwrap();
            });
            let settings_json = Component::get_settings_json().unwrap();
            assert_eq!(EXPECTED, settings_json);
            // Make a SELECT just to make sure it is stored where we expect.
            block_on(async move {
                assert_eq!(
                    vec![EXPECTED],
                    select_single_non_null_column("llm", "settings").await
                );
            });
        }

        #[test]
        #[ignore]
        fn user_update_should_create_the_user() {
            set_up();
            block_on(async move {
                delete_from("users").await;
            });
            let login = "login-user-update-should-create-the-user";
            let description = "desc 1";
            println!("Creating user `{login}` with description `{description}`");
            Component::update_user_description(login.to_string(), description.to_string()).unwrap();
            // Check user
            block_on({
                async move {
                    assert_eq!(vec![login], select_name("users").await);
                }
            });

            println!("Deleting the user");
            Component::remove_star(login.to_string(), "any".to_string()).unwrap();
            block_on(async move {
                assert_eq!(Vec::<String>::new(), select_name("users").await);
            });
        }

        #[test]
        #[ignore]
        fn link_and_update_should_work_on_the_same_user() {
            set_up();
            block_on(async move {
                delete_from("users").await;
                delete_from("repos").await;
                delete_from("stars").await;
            });
            let login = "login-link-and-update-should-work-on-the-same-user";
            let repo = "repo-link-and-update-should-work-on-the-same-user";
            println!("Creating user `{login}` and repo `{repo}`");
            let description =
                Component::add_star_get_description(login.to_string(), repo.to_string()).unwrap();
            assert!(description.is_none());

            let description = "desc";
            println!("Updating user `{login}` with description `{description}`");
            Component::update_user_description(login.to_string(), description.to_string()).unwrap();
            // Check the user and description directly in the database.
            block_on(async move {
                assert_eq!(
                    vec![vec![Some(login.to_string()), Some(description.to_string())]],
                    select("users", &["name", "description"]).await
                )
            });
        }

        #[test]
        #[ignore]
        fn list_stargazers_should_work() {
            set_up();
            block_on(async move {
                delete_from("users").await;
                delete_from("repos").await;
                delete_from("stars").await;
            });
            let repo1 = "repo1-list-stargazers-should-work";
            let repo2 = "repo2-list-stargazers-should-work";

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
                login: "login1-list-stargazers-should-work".to_string(),
                description: Some("desc".to_string()),
                repo: repo1.to_string(),
            };
            insert(&s_old_repo1);

            let s_new_repo1 = Stargazer {
                login: "login2-list-stargazers-should-work".to_string(),
                description: None,
                repo: repo1.to_string(),
            };
            insert(&s_new_repo1);

            let s_repo2 = Stargazer {
                login: "login3-list-stargazers-should-work".to_string(),
                description: None,
                repo: repo2.to_string(),
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
                Component::list_stargazers(1, Some(repo1.to_string()), Ordering::Descending)
                    .unwrap();
            assert_eq!(vec![s_new_repo1.clone()], actual);
            // Get the oldest only from all repos
            let actual = Component::list_stargazers(1, None, Ordering::Ascending).unwrap();
            assert_eq!(vec![s_old_repo1.clone()], actual);
        }

        #[test]
        #[ignore]
        fn list_stargazers_should_be_updated_on_description_update() {
            set_up();
            block_on(async move {
                delete_from("users").await;
                delete_from("repos").await;
                delete_from("stars").await;
            });

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
                login: "login1-list-stargazers-should-be-updated-on-description-update".to_string(),
                description: None,
                repo: "repo1-list-stargazers-should-be-updated-on-description-update".to_string(),
            };
            insert(&s_old);
            let s_new = Stargazer {
                login: "login2-list-stargazers-should-be-updated-on-description-update".to_string(),
                description: None,
                repo: "repo2-list-stargazers-should-be-updated-on-description-update".to_string(),
            };
            insert(&s_new);
            let actual = Component::list_stargazers(2, None, Ordering::Descending).unwrap();
            assert_eq!(vec![s_new.clone(), s_old.clone()], actual);
            // Update the description of s_old to change its `updated_at`
            s_old.description = Some("desc".to_string());
            Component::update_user_description(
                s_old.login.clone(),
                s_old.description.clone().expect("description was just set"),
            )
            .unwrap();
            // Get the reordered list
            let actual = Component::list_stargazers(2, None, Ordering::Descending).unwrap();
            assert_eq!(vec![s_old, s_new], actual);
        }

        #[ignore]
        #[test]
        fn link_after_update_should_return_the_description() {
            set_up();
            block_on(async move {
                delete_from("users").await;
                delete_from("repos").await;
                delete_from("stars").await;
            });

            let login = "login1-link-after-update-should-return-the-description".to_string();
            let description = "desc".to_string();
            println!("Creating user `{login}` with description `{description}`");
            Component::update_user_description(login.clone(), description.clone()).unwrap();

            let repo = "repo1-link-after-update-should-return-the-description".to_string();
            println!("Starring repo `{repo}`");
            let actual_description =
                Component::add_star_get_description(login.clone(), repo.clone()).unwrap();

            assert_eq!(Some(&description), actual_description.as_ref());

            // Check the user and description directly in the database.
            block_on(async move {
                assert_eq!(
                    vec![vec![Some(login), Some(description)]],
                    select("users", &["name", "description"]).await
                );
            });
        }

        #[test]
        #[ignore]
        fn user_link_unlink_should_retain_the_repo_only() {
            set_up();
            block_on(async move {
                delete_from("users").await;
                delete_from("repos").await;
                delete_from("stars").await;
            });
            let login = "login1-user-link-unlink-should-retain-the-repo-only".to_string();
            let repo = "repo1-user-link-unlink-should-retain-the-repo-only".to_string();
            println!("Creating user `{login}` and repo `{repo}`");
            Component::add_star_get_description(login.clone(), repo.clone()).unwrap();
            // Check that data is inserted into `users`, `repos`, `stars`.
            block_on({
                let login = login.clone();
                let repo = repo.clone();
                async move {
                    assert_eq!(vec![login.clone()], select_name("users").await);
                    assert_eq!(vec![repo.clone()], select_name("repos").await);
                    assert_eq!(
                        vec![vec![Some(login.clone()), Some(repo.clone())]],
                        select("stars", &["user_name", "repo_name"]).await
                    );
                }
            });
            println!("Deleting the user");
            Component::remove_star(login, repo.clone()).unwrap();
            // Check that only `repos` is not empty
            block_on(async move {
                assert_eq!(Vec::<String>::new(), select_name("users").await);
                assert_eq!(vec![repo.clone()], select_name("repos").await);
                assert_eq!(
                    Vec::<Vec<Option<String>>>::new(),
                    select("stars", &["user_name", "repo_name"]).await
                );
            });
        }

        #[test]
        #[ignore]
        fn user_updated_at_should_not_change_if_already_starred() {
            set_up();
            block_on(async move {
                delete_from("users").await;
                delete_from("repos").await;
                delete_from("stars").await;
            });

            let login = "login1-user-updated-at-should-not-change-if-already-starred".to_string();
            let repo = "repo1-user-updated-at-should-not-change-if-already-starred".to_string();
            println!("Creating user `{login}` and repo `{repo}`");
            Component::add_star_get_description(login.clone(), repo.clone()).unwrap();

            // Capture the initial updated_at timestamp

            let initial_updated_at: String = block_on(async move {
                select("users", &["updated_at"])
                    .await
                    .into_iter()
                    .next()
                    .expect("user should exist")
                    .into_iter()
                    .next()
                    .expect("updated_at should be present")
                    .expect("updated_at should not be null")
            });

            // Star the same repo again with the same user
            Component::add_star_get_description(login.clone(), repo.clone()).unwrap();

            // Capture the updated updated_at timestamp
            let updated_updated_at: String = block_on(async move {
                select("users", &["updated_at"])
                    .await
                    .into_iter()
                    .next()
                    .expect("user should exist")
                    .into_iter()
                    .next()
                    .expect("updated_at should be present")
                    .expect("updated_at should not be null")
            });

            // Verify that the updated_at timestamp has not changed
            assert_eq!(initial_updated_at, updated_updated_at);
        }
    }
}
