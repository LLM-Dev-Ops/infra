# infra-llm-client

Unified LLM provider abstraction for LLM-Dev-Ops infrastructure.

## Overview

This crate provides a common interface for interacting with various LLM providers (OpenAI, Anthropic, etc.) through a unified `LlmProvider` trait. It includes:

- **Core `LlmProvider` trait**: Defines the interface for implementing provider-specific adapters
- **Common types**: Standardized types for LLM requests, responses, and messages
- **Error handling**: Comprehensive error types for LLM operations
- **Placeholder adapters**: Stub implementations for future provider integrations

## Features

- `std` (default): Enable standard library support

## Architecture

The crate is organized into the following modules:

- `provider`: Core `LlmProvider` trait definition
- `types`: Common types for requests, responses, and messages
- `error`: Error types and result aliases
- `adapters`: Provider-specific adapter implementations (OpenAI, Anthropic, etc.)

## Status

This is a stub module with placeholder implementations. The adapters currently return `Unsupported` errors and will be fully implemented in future iterations.

## Usage

```rust
use infra_llm_client::{LlmProvider, LlmRequest, Message, Role};

async fn example(provider: impl LlmProvider) -> Result<(), Box<dyn std::error::Error>> {
    let request = LlmRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            Message {
                role: Role::User,
                content: "Hello, world!".to_string(),
            }
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        top_p: None,
        n: None,
        stream: None,
        stop: None,
    };

    let response = provider.complete(request).await?;
    println!("Response: {}", response.content);
    Ok(())
}
```

## Future Work

- Implement OpenAI adapter
- Implement Anthropic adapter
- Add retry logic and rate limiting
- Add request/response caching
- Add telemetry and observability
- Support additional providers (Cohere, Hugging Face, etc.)

## License

Dual-licensed under MIT or Apache-2.0.
