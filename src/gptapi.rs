use reqwest::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use std::env;

const CHAT_COMPLETION: &str = "https://api.openai.com/v1/chat/completions";
const COMPLETION: &str = "https://api.openai.com/v1/completions";
const LIST_MODELS: &str = "https://api.openai.com/v1/models";

pub struct GptBot {
    key: String,
    client: Client,
}
impl GptBot {
    pub fn new() -> Self {
        let key = env::var("OPENAI_KEY").expect("token");
        let client = reqwest::Client::new();
        GptBot { key, client }
    }

    // pub async fn chat_req_builder(&self, )

    // let req = ChatRequest {
    //     // model: "gpt-4".to_owned(),
    //     model: "gpt-3.5-turbo".to_owned(),
    //     messages: vec![MessageObject {
    //         role: "user".to_owned(),
    //         content: body.to_owned(),
    //     }],
    // };

    pub async fn gpt_req(
        &self,
        messages: Vec<MessageObject>,
    ) -> Result<std::string::String, reqwest::Error> {
        let req = ChatRequest::new(messages);
        let gpt_res = self
            .client
            .post(CHAT_COMPLETION)
            .header(AUTHORIZATION, format!("Bearer {}", self.key))
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .json(&req)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let res_val: Result<GptResponsePayload, serde_json::Error> = serde_json::from_str(&gpt_res);

        if let Ok(payload) = res_val {
            Ok(match payload.choices.first() {
                Some(c) => c.message.content.clone(),
                None => "json can be hard sometimes. Something went wrong".to_owned(),
            })
        } else {
            Ok("json can be hard sometimes. Something went wrong".to_owned())
        }
    }
}
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<MessageObject>,
}
impl Default for ChatRequest {
    fn default() -> Self {
        ChatRequest {
            model: "gpt-4".to_owned(),
            messages: vec![MessageObject::default()],
        }
    }
}
impl ChatRequest {
    fn new(messages: Vec<MessageObject>) -> Self {
        ChatRequest {
            // model: "gpt-4".to_owned(),
            model: "gpt-3.5-turbo".to_owned(),
            messages,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MessageObject {
    pub role: String,
    pub content: String,
}
impl Default for MessageObject {
    fn default() -> Self {
        MessageObject {
            role: "user".to_owned(),
            content: "Hello!".to_owned(),
        }
    }
}
impl MessageObject {
    fn new(s: String) -> Self {
        MessageObject {
            role: "user".to_owned(),
            content: s.to_owned(),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct GptResponsePayload {
    id: String,
    object: String,
    created: i64,
    model: String,
    usage: serde_json::Value,
    choices: Vec<ChoicesObject>,
}
#[derive(Deserialize, Serialize)]
struct ChoicesObject {
    message: MessageObject,
    finish_reason: String,
    index: i64,
}
