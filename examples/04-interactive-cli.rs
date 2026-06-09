use std::io::{self, Write, BufRead};
use oai_rs::{LLMClient, ChatConfig};
use oai_rs::message::{Message, Role, ContentPart};

/// Interactive chat CLI with streaming and reasoning support.
///
/// Run with:
///   MODEL=mistral-small API_KEY=openai BASE_URL=http://localhost:21465/v1 cargo run --example 04-interactive-cli
#[tokio::main]
async fn main() {
    let model = std::env::var("MODEL").unwrap_or_else(|_| "gpt-4o".into());
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| "http://localhost:21465/v1".into());
    let api_key = std::env::var("API_KEY").unwrap_or_else(|_| "openai".into());

    let client = LLMClient::new(model, base_url, api_key, ChatConfig::default());
    let mut history: Vec<Message> = Vec::new();
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let system_prompt = std::env::var("SYSTEM").unwrap_or_else(|_| String::new());

    if !system_prompt.is_empty() {
        history.push(Message {
            role: Role::System,
            parts: vec![ContentPart::Text { text: system_prompt }],
        });
    }

    loop {
        print!(">>> ");
        io::stdout().flush().ok();

        let mut input = String::new();
        let bytes_read = reader.read_line(&mut input).ok();

        match bytes_read {
            Some(0) | None => break, // EOF (Ctrl+D)
            _ => {}
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "/exit" {
            break;
        }

        history.push(Message {
            role: Role::User,
            parts: vec![ContentPart::Text { text: input.into() }],
        });

        let mut rx = match client.stream(&history).await {
            Ok(rx) => rx,
            Err(e) => {
                eprintln!("Error: {e}");
                history.pop();
                continue;
            }
        };

        let mut dimmed = false;
        let mut has_reasoned = false;
        let mut assistant_content = String::new();

        while let Some(result) = rx.recv().await {
            let chunk = match result {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("\nStream error: {e}");
                    break;
                }
            };

            if let Some(ref reasoning) = chunk.reasoning {
                if !has_reasoned {
                    print!("\x1b[2mThinking: \x1b[0m\x1b[2m");
                    has_reasoned = true;
                    dimmed = true;
                }
                print!("{reasoning}");
            }

            if let Some(ref content) = chunk.content {
                if dimmed {
                    print!("\x1b[0m\n");
                    dimmed = false;
                }
                print!("{content}");
                assistant_content.push_str(content);
            }

            if chunk.reasoning.is_none() && dimmed {
                print!("\x1b[0m");
                dimmed = false;
            }

            io::stdout().flush().ok();
        }

        println!();

        history.push(Message {
            role: Role::Assistant,
            parts: vec![ContentPart::Text { text: assistant_content }],
        });
    }
}
