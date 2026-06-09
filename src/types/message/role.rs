//! The [`Role`] enum — who sent a message.

use serde::{Serialize, Deserialize};

/// The sender of a chat message.
///
/// Maps directly to the `role` field in the OpenAI message format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System prompt / instruction message.
    System,
    /// Model-generated response message.
    Assistant,
    /// End-user input message.
    User,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
        }
    }
}
