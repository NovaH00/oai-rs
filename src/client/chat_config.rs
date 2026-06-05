use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Generation parameters for a chat completion request.
///
/// Any provider-specific or unsupported parameters can be supplied
/// through `extra`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    /// Maximum number of tokens to generate.
    pub max_tokens: Option<u32>,

    /// Sampling temperature.
    pub temperature: Option<f32>,

    /// Nucleus sampling parameter.
    pub top_p: Option<f32>,

    /// Number of completions to generate.
    pub n: Option<u32>,

    /// Random seed for reproducibility.
    pub seed: Option<u64>,

    /// Additional provider-specific parameters.
    ///
    /// Examples:
    /// - reasoning_effort
    /// - frequency_penalty
    /// - presence_penalty
    /// - repetition_penalty
    /// - top_k
    /// - min_p
    /// - custom vLLM parameters

    #[serde(default)]
    pub extra: Value,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            seed: None,
            extra: json!({}),
        }
    }
}
