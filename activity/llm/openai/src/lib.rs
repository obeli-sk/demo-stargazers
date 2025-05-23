use exports::stargazers::llm::llm::Guest;
use serde::{Deserialize, Serialize};
use std::env;
use waki::Client;
use wit_bindgen::generate;

const ENV_OPENAI_API_KEY: &str = "OPENAI_API_KEY";

generate!({ generate_all });
pub(crate) struct Component;
export!(Component);

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

impl Guest for Component {
    fn respond(user_prompt: String, settings: String) -> Result<String, String> {
        let api_key = env::var(ENV_OPENAI_API_KEY)
            .map_err(|_| format!("{ENV_OPENAI_API_KEY} must be set as an environment variable"))?;

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

        let resp = Client::new()
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .map_err(|err| format!("{:?}", err))?;

        if resp.status_code() != 200 {
            return Err(format!("Unexpected status code: {}", resp.status_code()));
        }

        let response: OpenAIResponse = resp.json().map_err(|err| format!("{:?}", err))?;

        if let Some(choice) = response.choices.into_iter().next() {
            Ok(choice.message.content)
        } else {
            Err("No response from OpenAI".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::exports::stargazers::llm::llm::Guest;
    use crate::Component;
    use crate::ENV_OPENAI_API_KEY;
    use crate::{Message, Role, Settings};

    fn set_up() {
        let test_token = std::env::var(format!("TEST_{ENV_OPENAI_API_KEY}")).unwrap_or_else(|_| {
            panic!("TEST_{ENV_OPENAI_API_KEY} must be set as an environment variable")
        });
        std::env::set_var(ENV_OPENAI_API_KEY, test_token);
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
}
