use oai_rs::{LLMClient, ChatConfig};
use oai_rs::message::{Message, Role, ContentPart};
use serde::Deserialize;
use schemars::JsonSchema;

/// A structured response schema for recipe extraction.
///
/// The model will be forced to produce valid JSON matching this shape
/// via `response_format` / JSON Schema mode.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct Recipe {
    /// Name of the dish.
    pub name: String,
    /// Approximate cook time in minutes.
    pub cook_time_minutes: u32,
    /// List of ingredients with quantities.
    pub ingredients: Vec<String>,
    /// Step-by-step instructions.
    pub steps: Vec<String>,
}

/// Demonstrates structured output (JSON Schema) with `client.parse::<T>()`.
///
/// Run with:
///   MODEL=mistral-small API_KEY=openai BASE_URL=http://localhost:21465/v1 cargo run --example 03-structured-output
#[tokio::main]
async fn main() {
    let model = std::env::var("MODEL").unwrap_or_else(|_| "gpt-4o".into());
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "http://localhost:21465/v1".into());
    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| "openai".into());

    let client = LLMClient::new(model, base_url, api_key, ChatConfig::default());

    let messages = vec![
        Message {
            role: Role::System,
            parts: vec![ContentPart::Text {
                text: "Output valid JSON matching the requested schema.".into(),
            }],
        },
        Message {
            role: Role::User,
            parts: vec![ContentPart::Text {
                text: "Give me a recipe for chocolate chip cookies.".into(),
            }],
        },
    ];

    match client.parse::<Recipe>(&messages).await {
        Ok(result) => {
            println!("=== Parsed Recipe ===\n{:#?}\n", result.parsed);
            println!("=== Raw Response ===");
            println!("ID:       {:?}", result.raw.id);
            println!("Finish:   {:?}", result.raw.finish_reason);
            println!("Usage:    {:?}", result.raw.usage);
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
