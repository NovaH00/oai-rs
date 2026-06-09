//! Non-streaming chat completion.
//!
//! Contains the [`LLMClient::chat`] method.

use crate::core::response::ChatResponse;
use crate::types::{message, error};

use super::LLMClient;

impl LLMClient {
    /// Sends a non-streaming chat completion request.
    ///
    /// Returns a [`ChatResponse`] that includes the assistant's content,
    /// optional reasoning tokens, refusal, finish reason, and usage.
    pub async fn chat(&self, messages: &[message::Message]) -> Result<ChatResponse, error::Error> {
        let response = self.send_request(messages, false).await?;
        let json_data: serde_json::Value = response.json().await?;

        ChatResponse::from_oai_chat_completion(json_data)
    }
}
