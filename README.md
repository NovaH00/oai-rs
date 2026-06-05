# rust-oai-client

A Rust client for OpenAI-compatible chat completion APIs (OpenAI, vLLM, Ollama, etc.).

## Usage

```rust
use rust_oai_client::{LLMClient, ChatConfig, Message, Role, ContentPart};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LLMClient::new(
        "gpt-4o",
        "https://api.openai.com/v1",
        "sk-...",
        ChatConfig::default(),
    );

    let messages = vec![
        Message {
            role: Role::User,
            parts: vec![ContentPart::Text { text: "Hello!".into() }],
        },
    ];

    let reply = client.chat(&messages)?;
    println!("{reply}");
    Ok(())
}
```

## Methods

### Sync

| Method | Returns | Description |
|---|---|---|
| `chat` | `Result<String>` | Standard chat completion |
| `stream` | `Result<Receiver<Result<String>>>` | SSE streaming chat |
| `parse` | `Result<T>` | Structured output (JSON Schema) |

### Async

| Method | Returns | Description |
|---|---|---|
| `async_chat` | `Result<String>` | Standard chat completion |
| `async_stream` | `Result<Receiver<Result<String>>>` | SSE streaming chat |
| `async_parse` | `Result<T>` | Structured output (JSON Schema) |
