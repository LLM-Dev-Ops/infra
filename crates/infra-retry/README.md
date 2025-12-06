# infra-retry

Advanced retry policies for LLM-Dev-Ops infrastructure.

## Features

- **Flexible Retry Policies**: Define custom retry strategies with the `RetryPolicy` trait
- **Built-in Strategies**:
  - `ExponentialBackoff`: Exponentially increasing delays between retries
  - `FixedDelay`: Constant delay between retries
  - `WithJitter`: Add randomization to any policy to prevent thundering herd
- **Async-first**: Built on `tokio` for seamless async/await integration
- **Composable**: Combine and wrap policies for complex retry logic

## Usage

```rust
use infra_retry::{retry_with_policy, ExponentialBackoff};
use std::io;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let policy = ExponentialBackoff::default()
        .with_max_attempts(5)
        .with_initial_delay(std::time::Duration::from_millis(100));

    let mut count = 0;
    let result = retry_with_policy(
        || async {
            count += 1;
            if count < 3 {
                Err(io::Error::new(io::ErrorKind::Other, "temporary error"))
            } else {
                Ok("success")
            }
        },
        &policy,
    ).await?;

    println!("Result: {}", result);
    Ok(())
}
```

## Retry Strategies

### Exponential Backoff

```rust
use infra_retry::ExponentialBackoff;
use std::time::Duration;

let policy = ExponentialBackoff::new()
    .with_initial_delay(Duration::from_millis(100))
    .with_max_delay(Duration::from_secs(60))
    .with_multiplier(2.0)
    .with_max_attempts(5);
```

### Fixed Delay

```rust
use infra_retry::FixedDelay;
use std::time::Duration;

let policy = FixedDelay::new(Duration::from_secs(1), 3);
```

### With Jitter

```rust
use infra_retry::{ExponentialBackoff, WithJitter};

let base_policy = ExponentialBackoff::default();
let jittered = WithJitter::new(base_policy, 0.3); // 30% jitter
```

## Custom Retry Policies

Implement the `RetryPolicy` trait to create custom retry strategies:

```rust
use infra_retry::{RetryPolicy, RetryDecision};
use std::time::Duration;

struct CustomPolicy;

impl RetryPolicy for CustomPolicy {
    fn should_retry(&self, attempt: u32, error: &dyn std::error::Error) -> RetryDecision {
        if attempt < 3 {
            RetryDecision::Retry(Duration::from_secs(1))
        } else {
            RetryDecision::Stop
        }
    }

    fn delay_for(&self, attempt: u32) -> Option<Duration> {
        if attempt < 3 {
            Some(Duration::from_secs(1))
        } else {
            None
        }
    }

    fn max_attempts(&self) -> u32 {
        3
    }
}
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
