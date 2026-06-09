//! Structured output via JSON Schema.
//!
//! Contains the [`LLMClient::parse`] method which constrains the model
//! to produce valid JSON matching a compile-time-derived schema.

use crate::core::response::{ChatResponse, ParsedResponse};
use crate::types::{message, error};

use super::LLMClient;

impl LLMClient {
    /// Sends a chat completion with structured output (JSON Schema).
    ///
    /// The schema for `T` is derived at compile time via `schemars`.
    /// The model is instructed (via `response_format`) to output JSON
    /// matching that schema.  Returns a [`ParsedResponse`] containing
    /// both the deserialized `T` and the raw [`ChatResponse`] metadata.
    pub async fn parse<T>(&self, messages: &[message::Message]) -> Result<ParsedResponse<T>, error::Error>
    where
        T: serde::de::DeserializeOwned + schemars::JsonSchema,
    {
        let response = self.send_request_with_schema::<T>(messages, false).await?;

        let json_data: serde_json::Value = response.json().await?;

        let chat_response = ChatResponse::from_oai_chat_completion(json_data)?;

        let chat_content = chat_response.clone().content.ok_or(error::Error::NoContent)?;

        let parsed_output = serde_json::from_str::<T>(&chat_content)?;

        Ok(ParsedResponse {
            parsed: parsed_output,
            raw: chat_response
        })
    }
}
