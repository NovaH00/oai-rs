//! Request model for the OpenAI chat completion API.
//!
//! [`ChatRequest`] is serialized directly into the JSON body sent to
//! `/chat/completions`.

use serde::{self, Serialize};
use super::chat_config::ChatConfig;

/// Serialisable JSON body for a chat completion request.
#[derive(Serialize)]
pub struct ChatRequest<'a> {
    /// The model to use (e.g. `"gpt-4o"`, `"deepseek-chat"`).
    pub model: &'a str,

    /// Conversation history in OpenAI message format.
    pub messages: Vec<serde_json::Value>,

    /// Whether to stream tokens via SSE.
    pub stream: bool,

    /// Options for streaming (e.g. `include_usage`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<serde_json::Value>,

    /// JSON Schema to constrain the model output.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub response_format: Option<serde_json::Value>,

    /// Generation parameters (flattened into the top-level body).
    #[serde(flatten)]
    pub config: &'a ChatConfig,
}
