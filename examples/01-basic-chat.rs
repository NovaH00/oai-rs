use oai_rs::{LLMClient, ChatConfig};
use oai_rs::message::{Message, Role, ContentPart};

/// Run with:
///   MODEL=mistral-small API_KEY=openai BASE_URL=http://localhost:21465/v1 cargo run --example 01-basic-chat
#[tokio::main]
async fn main() {
    let model = std::env::var("MODEL").unwrap_or_else(|_| "gpt-4o".into());
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "http://localhost:21465/v1".into());
    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| "openai".into());

    let config = ChatConfig {
        max_tokens: Some(512),
        temperature: Some(0.7),
        ..ChatConfig::default()
    };

    let client = LLMClient::new(model, base_url, api_key, config);

    let messages = vec![
        Message {
            role: Role::System,
            parts: vec![ContentPart::Text {
                text: "You are a helpful assistant.".into(),
            }],
        },
        Message {
            role: Role::User,
            parts: vec![ContentPart::Text {
                text: "What is the capital of France?".into(),
            }],
        },
    ];

    match client.chat(&messages).await {
        Ok(response) => {
            println!("ID:    {:?}", response.id);
            println!("Content: {}", response.content.unwrap_or_default());
            println!("Reasoning: {:?}", response.reasoning);
            println!("Refusal: {:?}", response.refusal);
            println!("Finish reason: {:?}", response.finish_reason);
            println!("Usage: {:?}", response.usage);
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
