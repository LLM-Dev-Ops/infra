//! Unified error handling for LLM-Dev-Ops infrastructure.
//!
//! This crate provides:
//! - `InfraError`: The unified error enum for all infra operations
//! - Error conversion traits for external error types
//! - WASM-compatible error representation
//! - OpenTelemetry span recording utilities
//! - Retry logic helpers

mod error;
mod kinds;
mod context;
mod retry;

#[cfg(feature = "wasm")]
mod wasm;

pub mod testing;

pub use error::InfraError;
pub use kinds::{
    AuthErrorKind, CryptoOperation, IoOperation, MqOperation,
    SerializationFormat, VectorOperation,
};
pub use context::{ErrorContext, SourceLocation, TraceIds};
pub use retry::{RetryConfig, RetryStrategy};

/// Result type alias using InfraError
pub type InfraResult<T> = Result<T, InfraError>;

#[cfg(feature = "wasm")]
pub use wasm::JsInfraError;
