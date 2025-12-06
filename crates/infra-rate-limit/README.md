# infra-rate-limit

Rate limiting utilities for LLM-Dev-Ops infrastructure.

## Overview

This crate provides various rate limiting strategies for controlling request rates in distributed systems. It includes implementations of common rate limiting algorithms with async support via tokio.

## Features

- **Token Bucket**: Allows bursts while maintaining a steady rate
- **Sliding Window**: Accurate request tracking with sliding time windows
- **Fixed Window**: Simple and efficient with fixed time windows
- **Async/Await**: Full support for async operations
- **Thread-Safe**: All implementations are thread-safe

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
infra-rate-limit = { path = "../infra-rate-limit" }
```

### Token Bucket Example

```rust
use infra_rate_limit::{RateLimitConfig, TokenBucket, RateLimiter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RateLimitConfig::per_second(100.0)?;
    let limiter = TokenBucket::new(config);

    // Acquire a permit (waits if necessary)
    limiter.acquire().await?;

    // Try to acquire without waiting
    let result = limiter.try_acquire().await;
    if result.is_allowed() {
        // Process request
    }

    Ok(())
}
```

### Sliding Window Example

```rust
use infra_rate_limit::{RateLimitConfig, SlidingWindowLimiter, RateLimiter};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RateLimitConfig::new(10.0, 10, Duration::from_secs(1))?;
    let limiter = SlidingWindowLimiter::new(config);

    limiter.acquire().await?;

    Ok(())
}
```

### Fixed Window Example

```rust
use infra_rate_limit::{RateLimitConfig, FixedWindowLimiter, RateLimiter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RateLimitConfig::per_minute(1000.0)?;
    let limiter = FixedWindowLimiter::new(config);

    // Check available permits
    let available = limiter.available().await;
    println!("Available permits: {}", available);

    Ok(())
}
```

## Configuration

The `RateLimitConfig` type provides several convenience methods:

- `RateLimitConfig::per_second(rate)` - Configure requests per second
- `RateLimitConfig::per_minute(rate)` - Configure requests per minute
- `RateLimitConfig::per_hour(rate)` - Configure requests per hour
- `RateLimitConfig::new(rate, burst, window)` - Custom configuration

## Rate Limiting Strategies

### Token Bucket

The token bucket algorithm maintains a bucket of tokens that are consumed by requests. Tokens are added at a constant rate, and requests can consume tokens in bursts up to the bucket capacity. This algorithm provides good burst handling while maintaining a steady rate.

**Pros:**
- Allows controlled bursts
- Smooth rate limiting
- Memory efficient

**Cons:**
- Can allow bursts after idle periods

### Sliding Window

The sliding window algorithm tracks individual request timestamps within a moving time window. This provides more accurate rate limiting than fixed windows but requires more memory.

**Pros:**
- Accurate rate limiting
- No burst at window boundaries

**Cons:**
- Higher memory usage
- More complex implementation

### Fixed Window

The fixed window algorithm divides time into fixed intervals and counts requests in each window. Simple and efficient, but can allow double the rate at window boundaries.

**Pros:**
- Simple implementation
- Low memory usage
- Fast

**Cons:**
- Can allow bursts at window boundaries
- Less accurate

## License

See the workspace license.
