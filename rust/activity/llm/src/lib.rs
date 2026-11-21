use generated::export;
use generated::exports::stargazers::llm::llm::Guest;
use serde::{Deserialize, Serialize};
use std::env;
use wstd::{
    http::{Body, Client, Method, Request, StatusCode},
    runtime::block_on,
};

const ENV_LLM_API_KEY: &str = "LLM_API_KEY";
const ENV_LLM_BASE_URL: &str = "LLM_BASE_URL";
const DEFAULT_LLM_BASE_URL: &str = "https://api.openai.com";

mod generated {
    #![allow(clippy::empty_line_after_outer_attr)]
    include!(concat!(env!("OUT_DIR"), "/root.rs"));
}

struct Component;
export!(Component with_types_in generated);

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

#[derive(Serialize, Deserialize)]
struct Settings {
    model: String,
    #[serde(default)]
    messages: Vec<Message>,
    max_tokens: usize,
}

#[derive(Deserialize)]
struct ModelList {
    data: Vec<Model>,
}

#[derive(Deserialize)]
struct Model {
    id: String,
}

fn base_url() -> String {
    env::var(ENV_LLM_BASE_URL).unwrap_or_else(|_| DEFAULT_LLM_BASE_URL.to_string())
}

fn request_builder() -> wstd::http::request::Builder {
    // The cloud LLM integration authenticates requests itself, so the key is optional.
    match env::var(ENV_LLM_API_KEY) {
        Ok(api_key) => Request::builder().header("Authorization", format!("Bearer {api_key}")),
        Err(_) => Request::builder(),
    }
}

async fn list_models() -> Result<Vec<String>, String> {
    let req = request_builder()
        .method(Method::GET)
        .uri(format!("{}/v1/models", base_url()))
        .body(Body::empty())
        .map_err(|err| format!("cannot create the request - {err:?}"))?;
    let resp = Client::new()
        .send(req)
        .await
        .map_err(|err| format!("cannot send the request - {err:?}"))?;
    if resp.status() != StatusCode::OK {
        return Err(format!("Unexpected status code: {}", resp.status()));
    }
    let models: ModelList = resp
        .into_body()
        .json()
        .await
        .map_err(|err| format!("{err:?}"))?;
    Ok(models.data.into_iter().map(|model| model.id).collect())
}

async fn respond(user_prompt: String, settings: String) -> Result<String, String> {
    let base_url = base_url();

    let settings: Settings =
        serde_json::from_str(&settings).expect("`settings_json` must be parseable");

    let mut messages = settings.messages;
    messages.push(Message {
        role: Role::User,
        content: user_prompt,
    });

    let request_body = OpenAIRequest {
        model: settings.model,
        messages,
        max_tokens: settings.max_tokens,
    };

    let req = request_builder()
        .header("Content-Type", "application/json")
        .method(Method::POST)
        .uri(format!("{base_url}/v1/chat/completions"))
        .body(
            Body::from_json(&request_body)
                .map_err(|err| format!("cannot serialize the request - {err:?}"))?,
        )
        .map_err(|err| format!("cannot create the request - {err:?}"))?;
    let resp = Client::new()
        .send(req)
        .await
        .map_err(|err| format!("cannot send the request - {err:?}"))?;

    if resp.status() != StatusCode::OK {
        return Err(format!("Unexpected status code: {}", resp.status()));
    }
    let response: OpenAIResponse = resp
        .into_body()
        .json()
        .await
        .map_err(|err| format!("{err:?}"))?;

    if let Some(choice) = response.choices.into_iter().next() {
        Ok(choice.message.content)
    } else {
        Err("No response from OpenAI".to_string())
    }
}

impl Guest for Component {
    fn respond(user_prompt: String, settings: String) -> Result<String, String> {
        block_on(respond(user_prompt, settings))
    }

    fn list_models() -> Result<Vec<String>, String> {
        block_on(list_models())
    }
}

#[cfg(test)]
mod tests {
    use crate::Component;
    use crate::generated::exports::stargazers::llm::llm::Guest;
    use crate::{ENV_LLM_API_KEY, ENV_LLM_BASE_URL};
    use crate::{Message, Role, Settings};

    fn set_up() {
        let test_token = std::env::var(format!("TEST_{ENV_LLM_API_KEY}")).unwrap_or_else(|_| {
            panic!("TEST_{ENV_LLM_API_KEY} must be set as an environment variable")
        });
        unsafe { std::env::set_var(ENV_LLM_API_KEY, test_token) };
        if let Ok(base_url) = std::env::var(format!("TEST_{ENV_LLM_BASE_URL}")) {
            unsafe { std::env::set_var(ENV_LLM_BASE_URL, base_url) };
        }
    }

    #[test]
    #[ignore]
    fn request_should_succeed() {
        set_up();

        let user_prompt = std::env::var("TEST_OPENAI_USER_PROMPT")
            .unwrap_or_else(|_| "Tell me about Rust programming.".to_string());
        let settings_json = std::env::var("TEST_OPENAI_SETTINGS_JSON").unwrap_or_else(|_| {
            serde_json::to_string(&Settings {
                messages: vec![Message {
                    role: Role::System,
                    content: "You are a helpful assistant".to_string(),
                }],
                model: "gpt-3.5-turbo".to_string(),
                max_tokens: 50,
            })
            .unwrap()
        });
        let res = Component::respond(user_prompt, settings_json);
        let res = res.unwrap();
        println!("{res}");
    }

    #[test]
    #[ignore]
    fn list_models_should_succeed() {
        set_up();
        let models = Component::list_models().unwrap();
        assert!(!models.is_empty());
    }
}
