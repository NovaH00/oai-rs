//! The [`LLMClient`] struct and its public methods.
//!
//! Public API methods are split across sibling modules:
//! - [`chat`](self::chat) — standard chat completion
//! - [`stream`](self::stream) — SSE streaming (incl. reasoning, usage)
//! - [`parse`](self::parse) — structured output (JSON Schema)
//!
//! Internal HTTP/transport logic lives in [`transport`](self::transport).

mod transport;
mod chat;
mod stream;
mod parse;

use crate::core::chat_config::ChatConfig;

/// Client for interacting with OpenAI-compatible chat completion APIs.
///
/// Provides async methods for standard chat, SSE streaming, and
/// structured output (JSON Schema) parsing.
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
}
