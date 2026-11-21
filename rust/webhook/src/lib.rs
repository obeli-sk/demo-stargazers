use crate::generated::obelisk::types::time::ScheduleAt::Now;
use generated::stargazers::workflow_obelisk_schedule::workflow::{
    star_added_schedule, star_removed_schedule,
};
use hmac::KeyInit as _;
use wstd::http::{Error, Request, Response, StatusCode};
use wstd::http::{Method, body::Body};

mod generated {
    #![allow(clippy::empty_line_after_outer_attr)]
    include!(concat!(env!("OUT_DIR"), "/any.rs"));
}

const HTTP_HEADER_SIGNATURE: &str = "X-Hub-Signature-256";
const HTTP_HEADER_EVENT: &str = "X-GitHub-Event";
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

#[wstd::http_server]
async fn main(req: Request<Body>) -> Result<Response<Body>, Error> {
    if req.method() == Method::POST {
        handle_webhook(req).await
    } else {
        Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::empty())
            .unwrap())
    }
}

async fn handle_webhook(req: Request<Body>) -> Result<Response<Body>, Error> {
    let sha256_signature = req.headers().get(HTTP_HEADER_SIGNATURE).cloned();
    let event_type = req
        .headers()
        .get(HTTP_HEADER_EVENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let mut body = req.into_body();
    let body = match body.contents().await {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("cannot get request body contents - {err:?}");
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap());
        }
    };
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
        verify_signature(&secret, body, sha256_signature);
    }
    // GitHub sends `ping` when the webhook is created, and other events if subscribed.
    match event_type.as_deref() {
        None | Some("star") => {}
        Some("ping") => return Ok(Response::new(Body::from("pong"))),
        Some(other) => {
            println!("Ignoring the {other} event");
            return Ok(Response::new(Body::from(format!(
                "Ignoring the {other} event"
            ))));
        }
    }
    let event: StarEvent = match serde_json::from_slice(body) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!("Cannot deserialize - {err:?}");
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap());
        }
    };

    println!("Got event {event:?}");
    let repo = event.repository.to_string();
    // Execute the workflow.
    let execution_id = match event.action {
        Action::Created => star_added_schedule(Now, &event.sender.login, &repo),
        Action::Deleted => star_removed_schedule(Now, &event.sender.login, &repo),
    }
    .expect("workflow scheduling failed");
    let resp = Response::builder();
    let resp = resp.header("execution-id", execution_id.id);
    Ok(resp.body(Body::empty()).unwrap()) // Send response: 200 OK
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
