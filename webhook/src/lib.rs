use crate::obelisk::types::time::ScheduleAt::Now;
use stargazers::{
    db::{self, user::Ordering},
    workflow_obelisk_ext::workflow::{star_added_schedule, star_removed_schedule},
};
use waki::{handler, ErrorCode, Method, Request, Response};
use wit_bindgen::generate;

generate!({
    generate_all,
    additional_derives: [serde::Serialize],
});

const HTTP_HEADER_SIGNATURE: &str = "X-Hub-Signature-256";
const ENV_GITHUB_WEBHOOK_INSECURE: &str = "GITHUB_WEBHOOK_INSECURE";
const ENV_GITHUB_WEBHOOK_SECRET: &str = "GITHUB_WEBHOOK_SECRET";

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
    if matches!(req.method(), Method::Post) {
        handle_webhook(req)
    } else if matches!(req.method(), Method::Get) {
        handle_get(req)
    } else {
        Err(ErrorCode::HttpRequestMethodInvalid)
    }
}

fn handle_webhook(req: Request) -> Result<Response, ErrorCode> {
    let sha256_signature = req.header(HTTP_HEADER_SIGNATURE).cloned();
    let body = req.body().unwrap();
    if matches!(
        std::env::var(ENV_GITHUB_WEBHOOK_INSECURE).as_deref(),
        Ok("true")
    ) {
        println!(
        "WARN: Not verifying the request because {ENV_GITHUB_WEBHOOK_INSECURE} is set to `true`!"
    );
    } else {
        let secret = std::env::var(ENV_GITHUB_WEBHOOK_SECRET).unwrap_or_else(|_| {
            panic!("{ENV_GITHUB_WEBHOOK_SECRET} must be passed as environment variable")
        });
        let sha256_signature = sha256_signature
            .unwrap_or_else(|| panic!("HTTP header {HTTP_HEADER_SIGNATURE} must be set"));
        let sha256_signature = sha256_signature.to_str().unwrap_or_else(|_| {
            panic!("HTTP header {HTTP_HEADER_SIGNATURE} must be ASCII-encoded")
        });
        verify_signature(&secret, &body, sha256_signature);
    }
    let event: StarEvent = serde_json::from_slice(&body).map_err(|err| {
        eprintln!("Cannot deserialize - {err:?}");
        ErrorCode::HttpRequestDenied
    })?;
    println!("Got event {event:?}");
    let repo = event.repository.to_string();
    // Execute the workflow.
    let execution_id = match event.action {
        Action::Created => star_added_schedule(Now, &event.sender.login, &repo),
        Action::Deleted => star_removed_schedule(Now, &event.sender.login, &repo),
    };
    let resp = Response::builder();
    let resp = resp.header("execution-id", execution_id.id);
    resp.build() // Send response: 200 OK
}

/// Verify a message using a shared secret and X-Hub-Signature-256 formatted hash.
///
/// See [Validating webhook deliveries](https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries#validating-webhook-deliveries)
/// for details.
fn verify_signature(secret: &str, payload: &[u8], sha256_signature: &str) {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let sha256_signature = sha256_signature
        .strip_prefix("sha256=")
        .expect("`X-Hub-Signature-256` must start with `sha256=`");
    let sha256_signature =
        hex::decode(sha256_signature).expect("`X-Hub-Signature-256` must be hex-ecoded");

    // Create alias for HMAC-SHA256
    type HmacSha256 = Hmac<Sha256>;

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");

    mac.update(payload);
    mac.verify_slice(&sha256_signature)
        .expect("verification must succeed");
}

/// Render a JSON array with last few stargazers.
fn handle_get(req: Request) -> Result<Response, ErrorCode> {
    const MAX_LIMIT: u8 = 5;
    let query = req.query();
    let limit = query
        .get("limit")
        .and_then(|limit| limit.parse::<u8>().ok())
        .map(|limit| limit.min(MAX_LIMIT))
        .unwrap_or(MAX_LIMIT);
    let repo = query.get("repo").map(|x| x.as_str());
    let ordering = if query.get("ordering").map(|s| s.as_str()) == Some("asc") {
        Ordering::Ascending
    } else {
        Ordering::Descending
    };
    let list = db::user::list_stargazers(limit, repo, ordering).map_err(|err| {
        eprintln!("{err}");
        ErrorCode::InternalError(None)
    })?;
    Response::builder().json(&list).build()
}

#[cfg(test)]
mod tests {
    use crate::verify_signature;

    #[test]
    fn sha256_should_work() {
        let secret = "It's a Secret to Everybody";
        let payload = "Hello, World!";
        verify_signature(
            secret,
            payload.as_bytes(),
            "sha256=757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17",
        );
    }

    #[test]
    #[should_panic]
    fn wrong_sha256_should_panic() {
        let secret = "It's a Secret to Everybody";
        let payload = "Hello, World!";
        verify_signature(
            secret,
            payload.as_bytes(),
            "sha256=757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e18",
        );
    }

    #[test]
    #[should_panic]
    fn wrong_prefix_should_panic() {
        let secret = "It's a Secret to Everybody";
        let payload = "Hello, World!";
        verify_signature(
            secret,
            payload.as_bytes(),
            "757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e18",
        );
    }

    #[test]
    #[should_panic]
    fn invalid_hex_should_panic() {
        let secret = "It's a Secret to Everybody";
        let payload = "Hello, World!";
        verify_signature(
            secret,
            payload.as_bytes(),
            "sha256=757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e1X",
        );
    }
}
