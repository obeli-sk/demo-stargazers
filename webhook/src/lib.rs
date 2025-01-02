use stargazers::workflow::workflow::{star_added, star_removed};
use waki::{handler, ErrorCode, Method, Request, Response};
use wit_bindgen::generate;

generate!({ generate_all });

#[derive(Debug, Clone, serde::Deserialize)]
struct StarEvent {
    action: Action,
    sender: User,
    repository: Repository,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    Created,
    Deleted,
}

#[derive(Debug, Clone, serde::Deserialize, derive_more::Display)]
#[display("{login}")]
struct User {
    login: String,
}

#[derive(Debug, Clone, serde::Deserialize, derive_more::Display)]
#[display("{owner}/{name}")]
struct Repository {
    name: String,
    owner: User,
}

#[handler]
fn handle(req: Request) -> Result<Response, ErrorCode> {
    if !matches!(req.method(), Method::Post) {
        return Err(ErrorCode::HttpRequestMethodInvalid);
    }
    // FIXME: Verify GitHub signature
    let json_value: serde_json::Value = req.json().unwrap();
    println!("Got {json_value}");

    let event: StarEvent = serde_json::from_value(json_value).map_err(|err| {
        eprintln!("Cannot deserialize - {err:?}");
        ErrorCode::HttpRequestDenied
    })?;
    println!("Got event {event:?}");
    let repo = event.repository.to_string();
    // FIXME: Use -schedule instead, return the execution id as a response header.
    match event.action {
        Action::Created => star_added(&event.sender.login, &repo),
        Action::Deleted => star_removed(&event.sender.login, &repo),
    }
    .map_err(|_| ErrorCode::InternalError(None))?;
    Response::builder().build() // Send response: 200 OK
}
