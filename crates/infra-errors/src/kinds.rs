//! Error kind enumerations for categorizing errors.

use serde::{Deserialize, Serialize};

/// Vector database operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorOperation {
    Insert,
    Search,
    Delete,
    Update,
    Index,
    Compress,
    BatchInsert,
    BatchDelete,
}

impl std::fmt::Display for VectorOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Insert => write!(f, "insert"),
            Self::Search => write!(f, "search"),
            Self::Delete => write!(f, "delete"),
            Self::Update => write!(f, "update"),
            Self::Index => write!(f, "index"),
            Self::Compress => write!(f, "compress"),
            Self::BatchInsert => write!(f, "batch_insert"),
            Self::BatchDelete => write!(f, "batch_delete"),
        }
    }
}

/// Authentication/Authorization error kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthErrorKind {
    InvalidCredentials,
    TokenExpired,
    InsufficientPermissions,
    InvalidToken,
    MissingCredentials,
    RateLimited,
    AccountLocked,
    SessionExpired,
}

impl std::fmt::Display for AuthErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "invalid_credentials"),
            Self::TokenExpired => write!(f, "token_expired"),
            Self::InsufficientPermissions => write!(f, "insufficient_permissions"),
            Self::InvalidToken => write!(f, "invalid_token"),
            Self::MissingCredentials => write!(f, "missing_credentials"),
            Self::RateLimited => write!(f, "rate_limited"),
            Self::AccountLocked => write!(f, "account_locked"),
            Self::SessionExpired => write!(f, "session_expired"),
        }
    }
}

/// Cryptographic operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CryptoOperation {
    Encrypt,
    Decrypt,
    Sign,
    Verify,
    Hash,
    KeyGeneration,
    KeyDerivation,
}

impl std::fmt::Display for CryptoOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Encrypt => write!(f, "encrypt"),
            Self::Decrypt => write!(f, "decrypt"),
            Self::Sign => write!(f, "sign"),
            Self::Verify => write!(f, "verify"),
            Self::Hash => write!(f, "hash"),
            Self::KeyGeneration => write!(f, "key_generation"),
            Self::KeyDerivation => write!(f, "key_derivation"),
        }
    }
}

/// I/O operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoOperation {
    Read,
    Write,
    Delete,
    Create,
    List,
    Watch,
    Copy,
    Move,
}

impl std::fmt::Display for IoOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Delete => write!(f, "delete"),
            Self::Create => write!(f, "create"),
            Self::List => write!(f, "list"),
            Self::Watch => write!(f, "watch"),
            Self::Copy => write!(f, "copy"),
            Self::Move => write!(f, "move"),
        }
    }
}

/// Serialization format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerializationFormat {
    Json,
    Toml,
    Yaml,
    MessagePack,
    Protobuf,
}

impl std::fmt::Display for SerializationFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Toml => write!(f, "toml"),
            Self::Yaml => write!(f, "yaml"),
            Self::MessagePack => write!(f, "messagepack"),
            Self::Protobuf => write!(f, "protobuf"),
        }
    }
}

/// Message queue operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MqOperation {
    Publish,
    Subscribe,
    Acknowledge,
    Reject,
    Connect,
    Disconnect,
}

impl std::fmt::Display for MqOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Publish => write!(f, "publish"),
            Self::Subscribe => write!(f, "subscribe"),
            Self::Acknowledge => write!(f, "acknowledge"),
            Self::Reject => write!(f, "reject"),
            Self::Connect => write!(f, "connect"),
            Self::Disconnect => write!(f, "disconnect"),
        }
    }
}
