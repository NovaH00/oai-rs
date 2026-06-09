# oai-rs

A Rust client for OpenAI-compatible chat completion APIs (OpenAI, vLLM, Ollama, DeepSeek, etc.).

Supports standard chat, SSE streaming (including reasoning tokens), and structured output via JSON Schema.

**Status:** Work in progress. Tested against llama.cpp's OpenAI-compatible API endpoint.

## Usage

```rust
use oai_rs::{LLMClient, ChatConfig};
use oai_rs::message::{Message, Role, ContentPart};

#[tokio::main]
async fn main() {
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

    match client.chat(&messages).await {
        Ok(response) => println!("{}", response.content.unwrap_or_default()),
        Err(e) => eprintln!("{e}"),
    }
}
```

## Examples

Run any example with:

```bash
MODEL=gpt-4o API_KEY=sk-... BASE_URL=https://api.openai.com/v1 \
    cargo run --example 01-basic-chat
```

| Example | Method | Description |
|---|---|---|
| `01-basic-chat` | `chat` | Single Q&A, prints response metadata |
| `02-streaming` | `stream` | Streams tokens with reasoning/content display |
| `03-structured-output` | `parse` | JSON Schema-constrained response |
| `04-interactive-cli` | `stream` | Persistent conversation loop with CLI prompt |

### Interactive CLI

```
>>> Tell me a story about a robot.

[reasoning tokens appear in dim, then:]

Once upon a time, in a workshop of gears and wires...
>>> What happened next?
```

Exit with `exit`, `/exit`, or `Ctrl+D`. Set a system prompt via the `SYSTEM` env var.

## Methods

| Method | Returns | Description |
|---|---|---|
| `chat` | `Result<ChatResponse>` | Standard chat completion |
| `stream` | `Result<Receiver<Result<StreamChunk>>>` | SSE streaming (incl. reasoning, usage) |
| `parse` | `Result<ParsedResponse<T>>` | Structured output (JSON Schema) |

### Key types

- **`ChatResponse`** — `id`, `content`, `reasoning`, `refusal`, `finish_reason`, `usage`
- **`StreamChunk`** — per-delta `content`, `reasoning`, `refusal`, `finish_reason`, `usage`
- **`ParsedResponse<T>`** — `parsed: T` + `raw: ChatResponse`
- **`ChatConfig`** — `max_tokens`, `temperature`, `top_p`, `n`, `seed`, `response_format`, and extra provider params via `extra`

### Structured output

```rust
use serde::Deserialize;
use schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
struct Joke {
    setup: String,
    punchline: String,
}

let resp = client.parse::<Joke>(&messages).await?;
println!("{:?}", resp.parsed);  // the deserialized Joke
println!("{:?}", resp.raw);     // the full ChatResponse (usage, reasoning, etc.)
```
