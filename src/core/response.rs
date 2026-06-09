//! Response models for the OpenAI chat completion API.
//!
//! Provides types for non-streaming responses ([`ChatResponse`]), streaming
//! chunks ([`StreamChunk`]), and structured-output responses
//! ([`ParsedResponse`]).

use serde::{self, Deserialize};
use crate::types::error;

/// Why the model stopped generating.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// The model reached a natural stop point.
    Stop,
    /// The model hit the configured token limit.
    Length,
    /// The model called a tool/function.
    ToolCalls,
    /// Content was filtered by the API provider.
    ContentFilter,
    /// Any other / unrecognised finish reason.
    #[serde(other)]
    Unknown,
}

/// Token usage statistics returned by the API.
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    /// Number of tokens in the prompt.
    #[serde(default)]
    pub prompt_tokens: u32,

    /// Number of tokens in the generated completion.
    #[serde(default)]
    pub completion_tokens: u32,

    #[serde(default)]
    pub total_tokens: u32,
}

/// A non-streaming chat completion response.
///
/// Fields are optional because the API may omit them depending on the
/// model or provider.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    /// Unique identifier for the completion.
    pub id: Option<String>,
    /// The assistant's response text.
    pub content: Option<String>,
    /// Reasoning / chain-of-thought tokens (provider-specific).
    pub reasoning: Option<String>,
    /// Refusal message when the model declines to respond.
    pub refusal: Option<String>,
    /// Why generation finished.
    pub finish_reason: Option<FinishReason>,
    /// Token usage, if reported.
    pub usage: Option<Usage>
}

impl ChatResponse {
    /// Parse a [`ChatResponse`] from a raw OpenAI `/chat/completions`
    /// JSON value.
    pub fn from_oai_chat_completion(
        data: serde_json::Value,
    ) -> Result<ChatResponse, error::Error> {
        let choices = data["choices"]
            .as_array()
            .ok_or(error::Error::NoChoices)?;

        let choice = choices
            .first()
            .ok_or(error::Error::NoChoices)?;

        let finish_reason = choice["finish_reason"]
            .as_str()
            .map(|s| {
                serde_json::from_value(
                    serde_json::Value::String(s.to_owned())
                )
            })
            .transpose()?;

        let usage = match data.get("usage") {
            Some(usage) => Some(serde_json::from_value(usage.clone())?),
            None => None,
        };

        let message = &choice["message"];

        Ok(ChatResponse {
            id: data["id"]
                .as_str()
                .map(str::to_owned),

            content: message["content"]
                .as_str()
                .map(str::to_owned),

            reasoning: message["reasoning_content"]
                .as_str()
                .map(str::to_owned),

            refusal: message["refusal"]
                .as_str()
                .map(str::to_owned),

            finish_reason,

            usage,
        })
    }
}

/// A single SSE delta from a streaming chat completion.
///
/// Each chunk represents one token (or partial token) emitted by the
/// model, together with any metadata such as usage or finish reason
/// that may appear on the final chunk.
///
/// The API emits a penultimate event containing only token usage and no
/// content.  You can detect this event by checking whether `content`,
/// `reasoning`, and `finish_reason` are all `None` while `usage` is
/// `Some`.  In practice, simply capturing the last `usage` value
/// encountered works because the usage-only chunk carries the
/// authoritative totals.
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Request-level ID (may only appear on the first chunk).
    pub id: Option<String>,
    /// Content delta for this chunk.
    pub content: Option<String>,
    /// Reasoning / chain-of-thought delta (provider-specific).
    pub reasoning: Option<String>,
    /// Refusal delta.
    pub refusal: Option<String>,
    /// Finish reason (typically only set on the final chunk).
    pub finish_reason: Option<FinishReason>,
    /// Token usage (typically only set on the final chunk).
    pub usage: Option<Usage>,
}

impl StreamChunk {
    /// Parse a [`StreamChunk`] from a raw OpenAI SSE `data:` line.
    pub fn from_oai_chat_chunk(
        data: serde_json::Value,
    ) -> Result<StreamChunk, error::Error> {
        let choices = data["choices"]
            .as_array()
            .ok_or(error::Error::NoChoices)?;

        let usage = match data.get("usage") {
            Some(usage) => Some(serde_json::from_value(usage.clone())?),
            None => None,
        };

        // OpenAI emits a penultimate event with empty choices and final
        // usage when stream_options.include_usage is true.
        if choices.is_empty() {
            return usage
                .map(|u| StreamChunk {
                    id: None,
                    content: None,
                    reasoning: None,
                    refusal: None,
                    finish_reason: None,
                    usage: Some(u),
                })
                .ok_or(error::Error::NoChoices);
        }

        let choice = choices
            .first()
            .ok_or(error::Error::NoChoices)?;

        let finish_reason = choice["finish_reason"]
            .as_str()
            .map(|s| {
                serde_json::from_value(
                    serde_json::Value::String(s.to_owned())
                )
            })
            .transpose()?;

        let delta = &choice["delta"];

        Ok(StreamChunk {
            id: data["id"]
                .as_str()
                .map(str::to_owned),

            content: delta["content"]
                .as_str()
                .map(str::to_owned),

            reasoning: delta["reasoning_content"]
                .as_str()
                .map(str::to_owned),

            refusal: delta["refusal"]
                .as_str()
                .map(str::to_owned),

            finish_reason,

            usage,
        })
    }
}

/// The result of a structured-output (JSON Schema) request.
///
/// Contains both the deserialized `parsed` value and the underlying
/// [`ChatResponse`] so callers can inspect usage, finish reason, etc.
#[derive(Debug, Clone, Deserialize)]
pub struct ParsedResponse<T> {
    /// The parsed structured output.
    pub parsed: T,
    /// The full raw response (includes usage, finish reason, etc.).
    pub raw: ChatResponse
}
