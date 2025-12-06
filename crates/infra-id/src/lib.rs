//! ID generation utilities for LLM-Dev-Ops infrastructure.
//!
//! Provides multiple ID generation strategies:
//! - UUID v4 (random)
//! - UUID v7 (time-ordered)
//! - ULID (lexicographically sortable)
//! - NanoID (URL-safe short IDs)

use infra_errors::{InfraError, InfraResult};
use serde::{Deserialize, Serialize};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// Trait for ID generators
pub trait IdGenerator: Send + Sync {
    /// Generate a new ID
    fn generate(&self) -> String;

    /// Generate multiple IDs
    fn generate_batch(&self, count: usize) -> Vec<String> {
        (0..count).map(|_| self.generate()).collect()
    }
}

/// UUID v4 generator (random)
#[derive(Debug, Clone, Default)]
pub struct UuidV4Generator;

impl UuidV4Generator {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for UuidV4Generator {
    fn generate(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

/// UUID v7 generator (time-ordered)
#[derive(Debug, Clone, Default)]
pub struct UuidV7Generator;

impl UuidV7Generator {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for UuidV7Generator {
    fn generate(&self) -> String {
        uuid::Uuid::now_v7().to_string()
    }
}

/// ULID generator (lexicographically sortable)
#[derive(Debug, Clone, Default)]
pub struct UlidGenerator;

impl UlidGenerator {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator for UlidGenerator {
    fn generate(&self) -> String {
        ulid::Ulid::new().to_string()
    }
}

/// NanoID generator (URL-safe short IDs)
#[derive(Debug, Clone)]
pub struct NanoIdGenerator {
    alphabet: Vec<char>,
    length: usize,
}

impl Default for NanoIdGenerator {
    fn default() -> Self {
        Self::new(21)
    }
}

impl NanoIdGenerator {
    /// Default URL-safe alphabet
    const DEFAULT_ALPHABET: &'static str =
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_";

    #[must_use]
    pub fn new(length: usize) -> Self {
        Self {
            alphabet: Self::DEFAULT_ALPHABET.chars().collect(),
            length,
        }
    }

    #[must_use]
    pub fn with_alphabet(alphabet: &str, length: usize) -> Self {
        Self {
            alphabet: alphabet.chars().collect(),
            length,
        }
    }
}

impl IdGenerator for NanoIdGenerator {
    fn generate(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..self.length)
            .map(|_| {
                let idx = rng.gen_range(0..self.alphabet.len());
                self.alphabet[idx]
            })
            .collect()
    }
}

/// Snowflake-like ID generator for distributed systems
#[derive(Debug)]
pub struct SnowflakeGenerator {
    machine_id: u16,
    sequence: std::sync::atomic::AtomicU16,
    epoch: i64,
}

impl Clone for SnowflakeGenerator {
    fn clone(&self) -> Self {
        Self {
            machine_id: self.machine_id,
            sequence: std::sync::atomic::AtomicU16::new(
                self.sequence.load(std::sync::atomic::Ordering::SeqCst),
            ),
            epoch: self.epoch,
        }
    }
}

impl SnowflakeGenerator {
    /// Create a new Snowflake generator
    ///
    /// # Arguments
    /// * `machine_id` - Unique identifier for this machine (0-1023)
    #[must_use]
    pub fn new(machine_id: u16) -> Self {
        Self {
            machine_id: machine_id & 0x3FF, // 10 bits
            sequence: std::sync::atomic::AtomicU16::new(0),
            epoch: 1704067200000, // 2024-01-01 00:00:00 UTC
        }
    }

    fn next_sequence(&self) -> u16 {
        self.sequence
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            & 0xFFF // 12 bits
    }
}

impl IdGenerator for SnowflakeGenerator {
    fn generate(&self) -> String {
        let timestamp = chrono::Utc::now().timestamp_millis() - self.epoch;
        let sequence = self.next_sequence();

        // 41 bits timestamp | 10 bits machine_id | 12 bits sequence
        let id = ((timestamp as u64 & 0x1FFFFFFFFFF) << 22)
            | ((self.machine_id as u64) << 12)
            | (sequence as u64);

        id.to_string()
    }
}

/// Generate an error ID (UUID v4)
#[must_use]
pub fn generate_error_id() -> String {
    UuidV4Generator::new().generate()
}

/// Generate a request ID (UUID v7 for time ordering)
#[must_use]
pub fn generate_request_id() -> String {
    UuidV7Generator::new().generate()
}

/// Generate a session ID (ULID for sortability)
#[must_use]
pub fn generate_session_id() -> String {
    UlidGenerator::new().generate()
}

/// Generate a short ID (NanoID)
#[must_use]
pub fn generate_short_id() -> String {
    NanoIdGenerator::default().generate()
}

/// Validated ID wrapper
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(String);

impl Id {
    /// Create a new ID from a string
    pub fn new(id: impl Into<String>) -> InfraResult<Self> {
        let id = id.into();
        if id.is_empty() {
            return Err(InfraError::validation("ID cannot be empty"));
        }
        Ok(Self(id))
    }

    /// Create without validation (use carefully)
    #[must_use]
    pub fn from_trusted(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a new UUID v4 ID
    #[must_use]
    pub fn generate() -> Self {
        Self(UuidV4Generator::new().generate())
    }

    /// Generate a new time-ordered ID (UUID v7)
    #[must_use]
    pub fn generate_ordered() -> Self {
        Self(UuidV7Generator::new().generate())
    }

    /// Get the ID as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned String
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<Id> for String {
    fn from(id: Id) -> Self {
        id.0
    }
}

// WASM bindings
#[cfg(feature = "wasm")]
mod wasm {
    use super::*;

    #[wasm_bindgen(js_name = generateUuidV4)]
    pub fn generate_uuid_v4() -> String {
        UuidV4Generator::new().generate()
    }

    #[wasm_bindgen(js_name = generateUuidV7)]
    pub fn generate_uuid_v7() -> String {
        UuidV7Generator::new().generate()
    }

    #[wasm_bindgen(js_name = generateUlid)]
    pub fn generate_ulid() -> String {
        UlidGenerator::new().generate()
    }

    #[wasm_bindgen(js_name = generateNanoId)]
    pub fn generate_nano_id(length: Option<usize>) -> String {
        let len = length.unwrap_or(21);
        NanoIdGenerator::new(len).generate()
    }

    #[wasm_bindgen(js_name = generateShortId)]
    pub fn js_generate_short_id() -> String {
        generate_short_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_uuid_v4_uniqueness() {
        let gen = UuidV4Generator::new();
        let ids: HashSet<_> = (0..1000).map(|_| gen.generate()).collect();
        assert_eq!(ids.len(), 1000, "All UUIDs should be unique");
    }

    #[test]
    fn test_uuid_v7_ordering() {
        let gen = UuidV7Generator::new();
        let id1 = gen.generate();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = gen.generate();
        assert!(id1 < id2, "UUID v7 should be time-ordered");
    }

    #[test]
    fn test_ulid_format() {
        let gen = UlidGenerator::new();
        let id = gen.generate();
        assert_eq!(id.len(), 26, "ULID should be 26 characters");
    }

    #[test]
    fn test_nanoid_length() {
        let gen = NanoIdGenerator::new(10);
        let id = gen.generate();
        assert_eq!(id.len(), 10);
    }

    #[test]
    fn test_snowflake_uniqueness() {
        let gen = SnowflakeGenerator::new(1);
        let ids: HashSet<_> = (0..1000).map(|_| gen.generate()).collect();
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn test_id_wrapper() {
        let id = Id::generate();
        assert!(!id.as_str().is_empty());

        let empty_result = Id::new("");
        assert!(empty_result.is_err());
    }
}
