use request::{PipelineAction, PipelineRequest};
use response::PipelineResponse;
use serde::{Deserialize, Serialize};
use waki::Client;

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

    impl PipelineResponse {
        pub fn ok_responses(self) -> Result<Vec<Response>, String> {
            self.results
                .into_iter()
                .map(|res| match res {
                    ResponseResult::Ok { response } => Ok(response),
                    ResponseResult::Error { error } => {
                        Err(format!("Got response result error {error:?}"))
                    }
                })
                .collect::<Result<_, _>>()
        }
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

    /// Extracts the first [`TursoValue`] from the first row of the n-th response.
    pub fn extract_first_value_from_nth_response(
        responses: Vec<Response>,
        n: usize,
    ) -> Result<TursoValue, String> {
        let query_result = match responses.into_iter().nth(n) {
            Some(Response::Execute {
                result: Some(query_result),
            }) => query_result,
            _ => return Err("First response result is unexpected".to_string()),
        };
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

pub struct TursoClient {
    url: String,
    token: String,
    client: Client,
}

impl TursoClient {
    pub fn new() -> Result<Self, String> {
        let token = std::env::var(ENV_TURSO_TOKEN)
            .map_err(|_| format!("{ENV_TURSO_TOKEN} must be set as an environment variable"))?;
        let turso_location = std::env::var(ENV_TURSO_LOCATION)
            .map_err(|_| format!("{ENV_TURSO_LOCATION} must be set as an environment variable"))?;
        let client = Client::new();
        let url = format!("https://{turso_location}/v2/pipeline");
        Ok(Self { url, token, client })
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

    pub fn post_json(&self, request: &PipelineRequest) -> Result<Vec<response::Response>, String> {
        assert_eq!(
            Some(&PipelineAction::Close),
            request.requests.last(),
            "last action must be close"
        );
        let resp = self
            .post()
            .json(request)
            .send()
            .map_err(|err| format!("Failed to send request: {err:?}"))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use response::PipelineResponse;

    const PIPELINE_RESPONSE_JSON: &str = r#"{
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
    fn extract_first_value_from_nth_response_string_should_work() {
        let response: PipelineResponse = serde_json::from_str(PIPELINE_RESPONSE_JSON).unwrap();
        let response = response.ok_responses().unwrap();
        assert_eq!(3, response.len());
        let value = response::extract_first_value_from_nth_response(response, 1).unwrap();
        assert_eq!(
            TursoValue::Text {
                value: "suqbjkasrl".to_string()
            },
            value
        );
    }

    #[test]
    fn extract_first_value_from_nth_response_empty_rows_should_mean_null() {
        let response: PipelineResponse = serde_json::from_str(PIPELINE_RESPONSE_JSON).unwrap();
        let response = response.ok_responses().unwrap();
        assert_eq!(3, response.len());
        let value = response::extract_first_value_from_nth_response(response, 0).unwrap();
        assert_eq!(TursoValue::Null, value);
    }
}
