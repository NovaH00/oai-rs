//! Streaming chat completion (SSE).
//!
//! Contains the [`LLMClient::stream`] method which returns a
//! `Receiver<Result<StreamChunk>>` of server-sent events.

use tokio::sync::mpsc::Receiver;

use crate::core::response::StreamChunk;
use crate::types::{message, error};

use super::LLMClient;

impl LLMClient {
    /// Sends a streaming chat completion request and returns an async
    /// channel of SSE deltas.
    ///
    /// Each [`StreamChunk`] contains the content or reasoning delta for
    /// that chunk.  The final content-bearing chunk may include
    /// `finish_reason` and `usage`.
    ///
    /// The API emits an additional penultimate event containing
    /// authoritative token usage and no content.  To capture it, check
    /// for a chunk where all fields except `usage` are `None`, or simply
    /// track the last `usage` value received.
    pub async fn stream(
        &self,
        messages: &[message::Message],
    ) -> Result<Receiver<Result<StreamChunk, error::Error>>, error::Error> {

        let response = self.send_request(messages, true).await?;

        let (tx, rx) = tokio::sync::mpsc::channel(16);

        tokio::spawn(async move {
            use futures::StreamExt;
            use eventsource_stream::Eventsource;

            let mut stream = response
                .bytes_stream()
                .eventsource();

            while let Some(event) = stream.next().await {
                let data = match event {
                    Ok(v) => v.data,
                    Err(e) => {
                        let _ = tx.send(Err(error::Error::Api(e.to_string()))).await;
                        break;
                    },
                };

                if data == "[DONE]" {
                    return;
                }

                match serde_json::from_str::<serde_json::Value>(&data) {
                    Err(e) => {
                        let _ = tx.send(Err(error::Error::Serialization(e))).await;
                        break;
                    },
                    Ok(json_data) => {
                        let stream_chunk = StreamChunk::from_oai_chat_chunk(json_data);
                        let _ = tx.send(stream_chunk).await;
                    }
                }
            }
        });

        Ok(rx)
    }
}
