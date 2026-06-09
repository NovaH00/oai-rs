//! Message types for chat conversations.
//!
//! Defines [`Message`], [`Role`], and [`ContentPart`] for building
//! chat histories sent to the API.

mod role;
mod parts;

pub use role::Role;
pub use parts::{ContentPart, ImageFormat};

use serde_json::json;

/// A chat message exchanged with an OpenAI-compatible model.
///
/// Each message has a role (such as user, assistant, or system) and
/// contains one or more content parts. Content parts may include text,
/// images, or other supported content types.
///
/// # Example
///
/// ```rust
/// let message = Message {
///     role: Role::User,
///     parts: vec![
///         ContentPart::Text {
///             text: "Hello!".into(),
///         }
///     ],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Message {
    /// The role of the message sender.
    pub role: Role,
    /// The content parts that make up the message.
    pub parts: Vec<ContentPart>
}

impl Message {
    /// Converts this message into the JSON format expected by
    /// OpenAI-compatible chat completion APIs.
    ///
    /// Each content part is converted using
    /// [`ContentPart::to_oai_json`], and the message role is serialized
    /// as its corresponding API string value.
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` in the following form:
    ///
    /// ```json
    /// {
    ///   "role": "user",
    ///   "content": [
    ///     {
    ///       "type": "text",
    ///       "text": "Hello!"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn to_oai_json(&self) -> serde_json::Value {
        let content: Vec<serde_json::Value> = self
            .parts
            .iter()
            .map(ContentPart::to_oai_json)
            .collect();

        json!({
            "role": self.role.to_string(),
            "content": content,
        })
    }
}
