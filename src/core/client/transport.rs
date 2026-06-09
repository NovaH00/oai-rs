//! Internal HTTP transport logic.
//!
//! Methods here are `pub(super)` — visible within the `client` module
//! but not exported to external users.

use serde_json::json;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

use crate::core::request::ChatRequest;
use crate::types::{message, error};
use super::LLMClient;

impl LLMClient {
    /// Build the JSON body for a plain (non-schema) chat request.
    ///
    /// Always requests `stream_options: { include_usage: true }`.
    pub(super) fn get_body(
        &self,
        messages: &[message::Message],
        streaming: bool,
    ) -> Result<serde_json::Value, error::Error> {

        let stream_options = json!({
            "include_usage": true
        });

        Ok(serde_json::to_value(ChatRequest {
            model: &self.model,
            stream: streaming,
            stream_options: Some(stream_options),
            response_format: None,
            messages: messages
                .iter()
                .map(message::Message::to_oai_json)
                .collect(),
            config: &self.config,
        })?)
    }

    /// Build the JSON body for a structured-output (JSON Schema) request.
    ///
    /// Derives the schema from `T` at compile time via `schemars` and
    /// injects it as `response_format.json_schema`.
    pub(super) fn get_body_with_response_schema<T> (
        &self,
        messages: &[message::Message],
        streaming: bool,
    ) -> Result<serde_json::Value, error::Error>
    where
        T: serde::de::DeserializeOwned + schemars::JsonSchema
    {

        let schema = schemars::schema_for!(T);
        let name = std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap_or("Response");

        let response_format = json!({
            "type": "json_schema",
            "json_schema": {
                "name": name,
                "schema": serde_json::to_value(&schema)?,
                "strict": true,
            }
        });

        let stream_options = json!({
            "include_usage": true
        });

        Ok(serde_json::to_value(ChatRequest {
            model: &self.model,
            stream: streaming,
            stream_options: Some(stream_options),
            response_format: Some(response_format),
            messages: messages
                .iter()
                .map(message::Message::to_oai_json)
                .collect(),
            config: &self.config,
        })?)
    }


    /// Returns the full `/chat/completions` URL for this client's base URL.
    pub(super) fn get_chat_endpoint(&self) -> String {
        format!(
            "{}/chat/completions",
            self.base_url.trim_end_matches('/')
        )
    }

    /// Builds the standard HTTP headers for the API request.
    ///
    /// Includes the `Authorization: Bearer <key>` header.
    pub(super) fn get_headers(&self) -> Result<HeaderMap, error::Error> {
        let mut headers = HeaderMap::new();

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(
                &format!("Bearer {}", self.api_key)
            )?,
        );

        Ok(headers)
    }

    /// Send a POST request to the chat completions endpoint.
    ///
    /// Constructs the URL, headers, and body (via [`get_body`]) then
    /// dispatches the request.
    pub(super) async fn send_request(
        &self,
        messages: &[message::Message],
        streaming: bool
    ) -> Result<reqwest::Response, error::Error> {
        let client = reqwest::Client::new();
        let response = client
            .post(self.get_chat_endpoint())
            .headers(self.get_headers()?)
            .json(&self.get_body(messages, streaming)?)
            .send()
            .await?;

        Ok(response)
    }

    /// Send a POST request with JSON Schema structured output.
    ///
    /// Same as [`send_request`] but uses [`get_body_with_response_schema`]
    /// instead of [`get_body`].
    pub(super) async fn send_request_with_schema<T>(
        &self,
        messages: &[message::Message],
        streaming: bool
    ) -> Result<reqwest::Response, error::Error> 
    where
        T: serde::de::DeserializeOwned + schemars::JsonSchema
    {
        let client = reqwest::Client::new();
        let response = client
            .post(self.get_chat_endpoint())
            .headers(self.get_headers()?)
            .json(&self.get_body_with_response_schema::<T>(messages, streaming)?)
            .send()
            .await?;

        Ok(response)
    }
}
