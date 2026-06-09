use oai_rs::{LLMClient, ChatConfig};
use oai_rs::message::{Message, Role, ContentPart};

/// Streams a response with reasoning/thinking token support.
///
/// Run with:
///   MODEL=mistral-small API_KEY=openai BASE_URL=http://localhost:21465/v1 cargo run --example 02-streaming
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
                text: "You are a thoughtful assistant. Reason step by step.".into(),
            }],
        },
        Message {
            role: Role::User,
            parts: vec![ContentPart::Text {
                text: "Solve: 23 × 47".into(),
            }],
        },
    ];

    let mut rx = client.stream(&messages).await.expect("failed to start stream");

    use std::io::Write;
    let mut dimmed = false;
    let mut has_reasoned = false;

    while let Some(result) = rx.recv().await {
        match result {
            Ok(chunk) => {
                // Print reasoning tokens in dim style
                if let Some(ref reasoning) = chunk.reasoning {
                    if !has_reasoned {
                        print!("\x1b[2mThinking: \x1b[0m\x1b[2m");
                        has_reasoned = true;
                        dimmed = true;
                    }
                    print!("{reasoning}");
                }

                // Print visible content (resets dim if reasoning just ended)
                if let Some(content) = chunk.content {
                    if dimmed {
                        print!("\x1b[0m");
                        println!();
                        dimmed = false;
                    }
                    print!("{content}");
                }

                // Reset dim if chunk has no reasoning and dim is still on
                if chunk.reasoning.is_none() && dimmed {
                    print!("\x1b[0m");
                    dimmed = false;
                }
                if let Some(reason) = chunk.finish_reason {
                    println!("\n\nFinish reason: {reason:?}");
                }
                if let Some(usage) = chunk.usage {
                    println!("Usage: {usage:?}");
                }

                std::io::stdout().flush().ok();
            }
            Err(e) => {
                eprintln!("\nStream error: {e}");
                break;
            }
        }
    }
}
