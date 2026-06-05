use std::io::BufRead;
use serde_json::{json, Value};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

use crate::client::chat_config::ChatConfig;
use crate::client::chat::{ChatRequest, ChatResponse, ChatChunk};
use crate::types::{Message, Error};

/// Client for interacting with OpenAI-compatible chat completion APIs.
///
/// Provides both blocking and async methods for standard chat, SSE
/// streaming, and structured output (JSON Schema) parsing.
pub struct LLMClient {
    model: String,
    base_url: String,
    api_key: String,
    config: ChatConfig
}

impl LLMClient {
    /// Creates a new `LLMClient`.
    ///
    /// `oai_compat_base_url` should be the base URL of an OpenAI-compatible
    /// chat completions endpoint (e.g. `http://localhost:8080/v1`).
    pub fn new(
        model: impl Into<String>,
        oai_compat_base_url: impl Into<String>,
        api_key: impl Into<String>,
        chat_config: ChatConfig
    ) -> Self {
        Self {
            model: model.into(),
            base_url: oai_compat_base_url.into(),
            api_key: api_key.into(),
            config: chat_config
        }
    }

    /// Builds the JSON request body for a non-streaming, non-structured
    /// chat completion request.
    fn get_body(
        &self,
        messages: &[Message],
    ) -> Result<Value, Error> {
        Ok(serde_json::to_value(ChatRequest {
            model: &self.model,
            stream: false,
            messages: messages
                .iter()
                .map(Message::to_oai_json)
                .collect(),
            config: &self.config,
        })?)
    }

    /// Returns the full `/chat/completions` URL for this client's base URL.
    fn get_chat_endpoint(&self) -> String {
        format!(
            "{}/chat/completions",
            self.base_url.trim_end_matches('/')
        )
    }

    /// Builds the standard HTTP headers for the API request.
    ///
    /// Includes the `Authorization: Bearer <key>` header.
    fn get_headers(&self) -> Result<HeaderMap, Error> {
        let mut headers = HeaderMap::new();

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(
                &format!("Bearer {}", self.api_key)
            )?,
        );

        Ok(headers)
    }

    /// Builds a JSON body with structured output (`response_format`) for
    /// the given type `T`. The schema is derived at compile time via
    /// `schemars`.
    fn get_structured_body<T: schemars::JsonSchema>(
        &self,
        messages: &[Message],
    ) -> Result<Value, Error> {
        let schema = schemars::schema_for!(T);
        let name = std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap_or("Response");

        let mut body = self.get_body(messages)?;
        body["response_format"] = json!({
            "type": "json_schema",
            "json_schema": {
                "name": name,
                "schema": serde_json::to_value(&schema)?,
                "strict": true,
            }
        });
        Ok(body)
    }

    /// Sends a blocking chat completion request and returns the assistant's
    /// response text.
    pub fn chat(&self, messages: &[Message]) -> Result<String, Error> {
        let client = reqwest::blocking::Client::new();

        let response = client
            .post(self.get_chat_endpoint())
            .headers(self.get_headers()?)
            .json(&self.get_body(messages)?)
            .send()?;

        let response: ChatResponse = response.json()?;

        let msg = &response
            .choices
            .first()
            .ok_or(Error::NoChoices)?
            .message;

        if let Some(refusal) = &msg.refusal {
            return Err(Error::Api(refusal.clone()));
        }

        let content = msg
            .content
            .clone()
            .ok_or_else(|| Error::Api(
                "assistant message contained no content".into()
            ))?;

        Ok(content)
    }

    /// Sends an async chat completion request and returns the assistant's
    /// response text.
    pub async fn async_chat(&self, messages: &[Message]) -> Result<String, Error> {
        let client = reqwest::Client::new();

        let response = client
            .post(self.get_chat_endpoint())
            .headers(self.get_headers()?)
            .json(&self.get_body(messages)?)
            .send()
            .await?;

        let response: ChatResponse = response.json().await?;

        let msg = &response
            .choices
            .first()
            .ok_or(Error::NoChoices)?
            .message;

        if let Some(refusal) = &msg.refusal {
            return Err(Error::Api(refusal.clone()));
        }

        let content = msg
            .content
            .clone()
            .ok_or_else(|| Error::Api(
                "assistant message contained no content".into(),
            ))?;

        Ok(content)
    }

    /// Sends a blocking streaming chat completion request.
    ///
    /// Returns a `Receiver` that yields content deltas as they arrive
    /// from the API via SSE.
    pub fn stream(
        &self,
        messages: &[Message],
    ) -> Result<std::sync::mpsc::Receiver<Result<String, Error>>, Error> {

        let mut body = self.get_body(messages)?;
        body["stream"] = json!(true);

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(self.get_chat_endpoint())
            .headers(self.get_headers()?)
            .json(&body)
            .send()?;

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let reader = std::io::BufReader::new(response);
            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        break;
                    }

                    if let Some(text) = serde_json::from_str::<ChatChunk>(data)
                        .ok()
                        .and_then(|c| c.content_delta().map(String::from)) {
                        let _ = tx.send(Ok(text));
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Sends an async streaming chat completion request.
    ///
    /// Returns a `Receiver` that yields content deltas as they arrive
    /// from the API via SSE.
    pub async fn async_stream(
        &self,
        messages: &[Message],
    ) -> Result<tokio::sync::mpsc::Receiver<Result<String, Error>>, Error> {

        let mut body = self.get_body(messages)?;
        body["stream"] = json!(true);

        let client = reqwest::Client::new();
        let response = client
            .post(self.get_chat_endpoint())
            .headers(self.get_headers()?)
            .json(&body)
            .send()
            .await?;

        let (tx, rx) = tokio::sync::mpsc::channel(16);

        tokio::spawn(async move {
            use futures::StreamExt;

            let mut stream = response.bytes_stream();
            let mut buf = Vec::new();

            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(_) => break,
                };
                buf.extend_from_slice(&chunk);

                while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                    let line = String::from_utf8_lossy(&buf[..pos])
                        .trim_end_matches('\r')
                        .to_string();
                    buf.drain(..=pos);

                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            return;
                        }
                        if let Some(text) = serde_json::from_str::<ChatChunk>(data)
                            .ok()
                            .and_then(|c| c.content_delta().map(String::from))
                        {
                            let _ = tx.send(Ok(text)).await.ok();
                        }
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Sends a blocking chat completion with structured output (JSON Schema).
    ///
    /// The response is parsed directly into `T`. The schema for `T` is
    /// generated at compile time via `schemars`.
    pub fn parse<T>(&self, messages: &[Message]) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned + schemars::JsonSchema,
    {
        let body = self.get_structured_body::<T>(messages)?;

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(self.get_chat_endpoint())
            .headers(self.get_headers()?)
            .json(&body)
            .send()?;

        let chat_resp: ChatResponse = response.json()?;

        let msg = &chat_resp.choices
            .first()
            .ok_or(Error::NoChoices)?
            .message;

        if let Some(refusal) = &msg.refusal {
            return Err(Error::Api(refusal.clone()));
        }

        let text = msg
            .content
            .clone()
            .ok_or_else(|| Error::Api("no content".into()))?;

        Ok(serde_json::from_str(&text)?)
    }

    /// Sends an async chat completion with structured output (JSON Schema).
    ///
    /// The response is parsed directly into `T`. The schema for `T` is
    /// generated at compile time via `schemars`.
    pub async fn async_parse<T>(&self, messages: &[Message]) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned + schemars::JsonSchema,
    {
        let body = self.get_structured_body::<T>(messages)?;

        let client = reqwest::Client::new();
        let response = client
            .post(self.get_chat_endpoint())
            .headers(self.get_headers()?)
            .json(&body)
            .send()
            .await?;

        let chat_resp: ChatResponse = response.json().await?;

        let msg = &chat_resp.choices
            .first()
            .ok_or(Error::NoChoices)?
            .message;

        if let Some(refusal) = &msg.refusal {
            return Err(Error::Api(refusal.clone()));
        }

        let text = msg
            .content
            .clone()
            .ok_or_else(|| Error::Api("no content".into()))?;

        Ok(serde_json::from_str(&text)?)
    }
}
