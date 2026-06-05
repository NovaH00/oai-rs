use serde::{Serialize, Deserialize};
use serde_json::Value;

use super::chat_config::ChatConfig;

#[derive(Serialize)]
pub struct ChatRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<Value>,

    #[serde(skip_serializing_if = "is_false")]
    pub stream: bool,

    #[serde(flatten)]
    pub config: &'a ChatConfig,
}

fn is_false(b: &bool) -> bool {
    !b
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub message: AssistantMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssistantMessage {
    #[serde(default)]
    pub content: Option<String>,

    #[serde(default)]
    pub refusal: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChunk {
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkChoice {
    pub delta: Delta,
}

#[derive(Debug, Deserialize)]
pub struct Delta {
    pub content: Option<String>,
}

impl ChatChunk {
    pub fn content_delta(&self) -> Option<&str> {
        self.choices.first()?.delta.content.as_deref()
    }
}
