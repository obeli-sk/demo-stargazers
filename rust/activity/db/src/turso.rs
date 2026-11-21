use anyhow::{Context, anyhow, bail};
use request::{Batch, BatchStep, Condition, PipelineAction, PipelineRequest, Stmt};
use response::{PipelineResponse, QueryResult, Response, ResponseResult};
use serde::{Deserialize, Serialize};
use wstd::http::{Body, Client, Method, Request, StatusCode};

use crate::{ENV_TURSO_LOCATION, ENV_TURSO_TOKEN};

pub mod request {
    use serde::Serialize;

    use super::TursoValue;
    #[derive(Serialize)]
    pub struct PipelineRequest {
        pub requests: Vec<PipelineAction>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    #[serde(tag = "type", rename_all = "lowercase")]
    pub enum PipelineAction {
        Batch { batch: Batch },
        Close,
    }

    #[derive(Debug, Serialize, PartialEq)]
    pub struct Batch {
        pub steps: Vec<BatchStep>,
    }

    #[derive(Debug, Serialize, PartialEq)]
    pub struct BatchStep {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub condition: Option<Condition>,
        pub stmt: Stmt,
    }

    #[derive(Debug, Serialize, PartialEq)]
    #[serde(tag = "type", rename_all = "lowercase")]
    pub enum Condition {
        Ok { step: usize },
        Not { cond: Box<Condition> },
    }

    #[derive(Debug, Serialize, Default, PartialEq)]
    pub struct Stmt {
        pub sql: String,
        pub named_args: Vec<NamedArg>,
    }

    impl Stmt {
        pub fn new(sql: impl Into<String>) -> Self {
            Self {
                sql: sql.into(),
                named_args: Vec::new(),
            }
        }

        /// Binds a text value to the named parameter `:name`.
        pub fn arg(mut self, name: &'static str, value: impl Into<String>) -> Self {
            self.named_args.push(NamedArg {
                name,
                value: TursoValue::Text {
                    value: value.into(),
                },
            });
            self
        }
    }

    #[derive(Debug, Serialize, PartialEq)]
    pub struct NamedArg {
        pub name: &'static str,
        pub value: TursoValue,
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
#[non_exhaustive]
pub enum TursoValue {
    Text { value: String }, // TODO: Cow
    Integer { value: String },
    Null,
}

pub mod response {
    use serde::Deserialize;

    use super::TursoValue;

    #[derive(Debug, Deserialize)]
    pub struct PipelineResponse {
        pub results: Vec<ResponseResult>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type", rename_all = "lowercase")]
    pub enum ResponseResult {
        Ok { response: Response },
        Error { error: ResponseResultError },
    }

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)] // The content will be printed using Debug formatting.
    pub struct ResponseResultError {
        pub message: String,
        pub code: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "type", rename_all = "lowercase")]
    pub enum Response {
        Batch { result: BatchResult },
        Close,
    }

    #[derive(Debug, Deserialize)]
    pub struct BatchResult {
        pub step_results: Vec<Option<QueryResult>>,
        pub step_errors: Vec<Option<ResponseResultError>>,
    }

    #[derive(Debug, Deserialize)]
    pub struct QueryResult {
        pub rows: Vec<QueryRow>,
        pub cols: Vec<QueryCol>,
        #[expect(dead_code)]
        pub affected_row_count: i64,
    }

    #[derive(Debug, Deserialize)]
    pub struct QueryCol {
        pub name: String,
        #[expect(dead_code)]
        pub decltype: Option<String>, // None for null values.
    }

    #[derive(Debug, Deserialize)]
    pub struct QueryRow(pub Vec<TursoValue>);

    /// Extracts the first [`TursoValue`] from the first row.
    pub fn first_value(query_result: QueryResult) -> Result<TursoValue, String> {
        let first_row = match query_result.rows.into_iter().next() {
            Some(row) => row,
            None => return Ok(TursoValue::Null), // rows: [] is a valid response when the select returns null.
        };
        let first_cell = match first_row.0.into_iter().next() {
            Some(cell) => cell,
            None => return Err("No cell in the first row".to_string()),
        };
        Ok(first_cell)
    }
}

/// Builds the batch steps running `statements` atomically: several statements run in an
/// immediate transaction, each one only if the previous step succeeded.
fn transaction_steps(statements: Vec<Stmt>) -> Vec<BatchStep> {
    let explicit = statements.len() > 1;
    let ok = |step: usize| Condition::Ok { step };
    let mut steps = Vec::new();
    if explicit {
        steps.push(BatchStep {
            condition: None,
            stmt: Stmt::new("BEGIN IMMEDIATE"),
        });
    }
    for stmt in statements {
        let condition = steps.len().checked_sub(1).map(ok);
        steps.push(BatchStep { condition, stmt });
    }
    if explicit {
        let condition = Some(ok(steps.len() - 1));
        steps.push(BatchStep {
            condition,
            stmt: Stmt::new("COMMIT"),
        });
        let condition = Some(Condition::Not {
            cond: Box::new(ok(steps.len() - 1)),
        });
        steps.push(BatchStep {
            condition,
            stmt: Stmt::new("ROLLBACK"),
        });
    }
    steps
}

/// Returns the results of the `statement_count` statements from a [`transaction_steps`] batch.
fn transaction_results(
    response: PipelineResponse,
    statement_count: usize,
) -> Result<Vec<QueryResult>, anyhow::Error> {
    let explicit = statement_count > 1;
    let mut results = response.results.into_iter();
    let (Some(batch), Some(close), None) = (results.next(), results.next(), results.next()) else {
        bail!("expected a batch and a close result");
    };
    let result = match batch {
        ResponseResult::Ok {
            response: Response::Batch { result },
        } => result,
        ResponseResult::Ok { response } => bail!("unexpected response {response:?}"),
        ResponseResult::Error { error } => bail!("Got batch error {error:?}"),
    };
    match close {
        ResponseResult::Ok {
            response: Response::Close,
        } => {}
        other => bail!("Got close error {other:?}"),
    }
    if let Some(error) = result.step_errors.into_iter().flatten().next() {
        bail!("Got statement error {error:?}");
    }
    let first = usize::from(explicit);
    let mut step_results = result.step_results.into_iter().skip(first);
    let statement_results = step_results
        .by_ref()
        .take(statement_count)
        .map(|result| result.ok_or_else(|| anyhow!("missing statement result")))
        .collect::<Result<Vec<_>, _>>()?;
    // Without errors, a missing COMMIT result means the transaction did not commit.
    if explicit && step_results.next().flatten().is_none() {
        bail!("Transaction was not committed");
    }
    Ok(statement_results)
}

pub struct TursoClient {
    url: String,
    token: String,
}

impl TursoClient {
    pub fn new() -> Result<Self, anyhow::Error> {
        let token = std::env::var(ENV_TURSO_TOKEN)
            .with_context(|| format!("{ENV_TURSO_TOKEN} must be set as an environment variable"))?;
        let turso_location = std::env::var(ENV_TURSO_LOCATION).with_context(|| {
            format!("{ENV_TURSO_LOCATION} must be set as an environment variable")
        })?;
        let url = format!("{turso_location}/v2/pipeline");
        Ok(Self { url, token })
    }

    fn bearer(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn post(&self) -> wstd::http::request::Builder {
        Request::builder()
            .header("Authorization", &self.bearer())
            .header("Content-Type", "application/json")
            .method(Method::POST)
            .uri(&self.url)
    }

    /// Runs the statements atomically in one stateless request and returns each statement's result.
    pub async fn transaction(
        &self,
        statements: Vec<Stmt>,
    ) -> Result<Vec<QueryResult>, anyhow::Error> {
        let result = async {
            let statement_count = statements.len();
            let request = PipelineRequest {
                requests: vec![
                    PipelineAction::Batch {
                        batch: Batch {
                            steps: transaction_steps(statements),
                        },
                    },
                    PipelineAction::Close,
                ],
            };
            let body = serde_json::to_vec(&request)?;
            let req = self
                .post()
                .header("Content-Length", body.len())
                .body(Body::from(body))?;
            let mut resp = Client::new().send(req).await?;
            let status = resp.status();
            let body = resp.body_mut().contents().await?;
            if status != StatusCode::OK {
                bail!(
                    "Unexpected status code {status}: {}",
                    String::from_utf8_lossy(body)
                );
            }
            let resp: PipelineResponse = serde_json::from_slice(body)
                .context("cannot deserialize the Turso pipeline response")?;
            transaction_results(resp, statement_count)
        }
        .await;
        if let Err(err) = &result {
            eprintln!("Turso request failed: {err:#}");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_steps_should_chain_the_statements() {
        let steps = transaction_steps(vec![Stmt::new("A"), Stmt::new("B")]);
        let steps = serde_json::to_value(&steps).unwrap();
        assert_eq!(
            serde_json::json!([
                {"stmt": {"sql": "BEGIN IMMEDIATE", "named_args": []}},
                {"condition": {"type": "ok", "step": 0}, "stmt": {"sql": "A", "named_args": []}},
                {"condition": {"type": "ok", "step": 1}, "stmt": {"sql": "B", "named_args": []}},
                {"condition": {"type": "ok", "step": 2}, "stmt": {"sql": "COMMIT", "named_args": []}},
                {"condition": {"type": "not", "cond": {"type": "ok", "step": 3}},
                    "stmt": {"sql": "ROLLBACK", "named_args": []}},
            ]),
            steps
        );
    }

    #[test]
    fn transaction_steps_should_not_wrap_a_single_statement() {
        let steps = transaction_steps(vec![Stmt::new("A").arg("x", "1")]);
        let steps = serde_json::to_value(&steps).unwrap();
        assert_eq!(
            serde_json::json!([
                {"stmt": {"sql": "A", "named_args": [{"name": "x", "value": {"type": "text", "value": "1"}}]}},
            ]),
            steps
        );
    }

    fn batch_response(step_results: &str, step_errors: &str) -> PipelineResponse {
        serde_json::from_str(&format!(
            r#"{{"baton": null, "base_url": null, "results": [
                {{"type": "ok", "response": {{"type": "batch", "result":
                    {{"step_results": {step_results}, "step_errors": {step_errors}}}}}}},
                {{"type": "ok", "response": {{"type": "close"}}}}
            ]}}"#
        ))
        .unwrap()
    }

    const EMPTY_RESULT: &str = r#"{"cols": [], "rows": [], "affected_row_count": 0}"#;

    #[test]
    fn transaction_results_should_return_the_statement_results() {
        let results = transaction_results(
            batch_response(
                &format!("[{EMPTY_RESULT}, {EMPTY_RESULT}, {EMPTY_RESULT}, {EMPTY_RESULT}, null]"),
                "[null, null, null, null, null]",
            ),
            2,
        )
        .unwrap();
        assert_eq!(2, results.len());
    }

    #[test]
    fn transaction_results_should_fail_on_a_statement_error() {
        let err = transaction_results(
            batch_response(
                &format!("[{EMPTY_RESULT}, null, null, null, {EMPTY_RESULT}]"),
                r#"[null, {"message": "UNIQUE constraint failed", "code": "SQLITE_CONSTRAINT"}, null, null, null]"#,
            ),
            2,
        )
        .unwrap_err();
        assert!(err.to_string().contains("UNIQUE"), "{err}");
    }

    #[test]
    fn transaction_results_should_fail_without_commit() {
        let err = transaction_results(
            batch_response(
                &format!("[{EMPTY_RESULT}, {EMPTY_RESULT}, {EMPTY_RESULT}, null, null]"),
                "[null, null, null, null, null]",
            ),
            2,
        )
        .unwrap_err();
        assert_eq!("Transaction was not committed", err.to_string());
    }

    #[test]
    fn first_value_of_empty_rows_should_mean_null() {
        let result: QueryResult = serde_json::from_str(EMPTY_RESULT).unwrap();
        assert_eq!(TursoValue::Null, response::first_value(result).unwrap());
    }
}
