//! Cryptographic utilities for LLM-Dev-Ops infrastructure.
//!
//! Provides:
//! - Hashing (SHA256, Blake3)
//! - Password hashing (Argon2id)
//! - Symmetric encryption (AES-256-GCM)
//! - Digital signatures (Ed25519)
//! - JWT support

mod hash;
mod cipher;
mod sign;
pub mod jwt;

pub use hash::{Hasher, Sha256Hasher, Blake3Hasher, PasswordHasher, PasswordAlgorithm};
pub use cipher::{Cipher, Aes256GcmCipher};
pub use sign::{Signer, Verifier, Ed25519Signer, Ed25519Verifier, Signature, PublicKey, Keypair};
pub use jwt::{JwtSigner, JwtAlgorithm, Claims};

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(feature = "wasm")]
pub use wasm::*;
