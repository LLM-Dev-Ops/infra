//! Hashing implementations.

use infra_errors::{CryptoOperation, InfraError, InfraResult};
use sha2::{Digest, Sha256};

/// Trait for hash functions
pub trait Hasher: Send + Sync {
    /// Hash data and return raw bytes
    fn hash(&self, data: &[u8]) -> Vec<u8>;

    /// Hash data and return hex-encoded string
    fn hash_hex(&self, data: &[u8]) -> String {
        hex::encode(self.hash(data))
    }

    /// Verify that data matches an expected hash
    fn verify(&self, data: &[u8], expected: &[u8]) -> bool {
        constant_time_eq::constant_time_eq(&self.hash(data), expected)
    }
}

/// SHA-256 hasher
#[derive(Debug, Clone, Default)]
pub struct Sha256Hasher;

impl Sha256Hasher {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Hasher for Sha256Hasher {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}

/// Blake3 hasher (fast, modern)
#[derive(Debug, Clone, Default)]
pub struct Blake3Hasher;

impl Blake3Hasher {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Hasher for Blake3Hasher {
    fn hash(&self, data: &[u8]) -> Vec<u8> {
        blake3::hash(data).as_bytes().to_vec()
    }
}

/// Password hashing algorithm
#[derive(Debug, Clone, Copy)]
pub enum PasswordAlgorithm {
    /// Argon2id with configurable parameters
    Argon2id {
        memory_cost: u32,
        time_cost: u32,
        parallelism: u32,
    },
}

impl Default for PasswordAlgorithm {
    fn default() -> Self {
        Self::Argon2id {
            memory_cost: 65536, // 64 MB
            time_cost: 3,
            parallelism: 4,
        }
    }
}

/// Password hasher for secure credential storage
#[derive(Debug, Clone)]
pub struct PasswordHasher {
    algorithm: PasswordAlgorithm,
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher {
    /// Create a new password hasher with default Argon2id settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            algorithm: PasswordAlgorithm::default(),
        }
    }

    /// Create with custom algorithm
    #[must_use]
    pub fn with_algorithm(algorithm: PasswordAlgorithm) -> Self {
        Self { algorithm }
    }

    /// Hash a password
    pub fn hash(&self, password: &str) -> InfraResult<String> {
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHasher as _, SaltString},
            Argon2, Params,
        };

        match self.algorithm {
            PasswordAlgorithm::Argon2id {
                memory_cost,
                time_cost,
                parallelism,
            } => {
                let salt = SaltString::generate(&mut OsRng);
                let params = Params::new(memory_cost, time_cost, parallelism, None).map_err(
                    |e| InfraError::Crypto {
                        operation: CryptoOperation::Hash,
                        message: e.to_string(),
                        context: None,
                    },
                )?;

                let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

                argon2
                    .hash_password(password.as_bytes(), &salt)
                    .map(|h| h.to_string())
                    .map_err(|e| InfraError::Crypto {
                        operation: CryptoOperation::Hash,
                        message: e.to_string(),
                        context: None,
                    })
            }
        }
    }

    /// Verify a password against a hash
    pub fn verify(&self, password: &str, hash: &str) -> InfraResult<bool> {
        use argon2::{password_hash::PasswordVerifier, Argon2, PasswordHash};

        let parsed_hash = PasswordHash::new(hash).map_err(|e| InfraError::Crypto {
            operation: CryptoOperation::Verify,
            message: e.to_string(),
            context: None,
        })?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

/// Convenience function to hash with SHA-256
#[must_use]
pub fn sha256(data: &[u8]) -> Vec<u8> {
    Sha256Hasher::new().hash(data)
}

/// Convenience function to hash with SHA-256 and return hex
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    Sha256Hasher::new().hash_hex(data)
}

/// Convenience function to hash with Blake3
#[must_use]
pub fn blake3(data: &[u8]) -> Vec<u8> {
    Blake3Hasher::new().hash(data)
}

/// Convenience function to hash with Blake3 and return hex
#[must_use]
pub fn blake3_hex(data: &[u8]) -> String {
    Blake3Hasher::new().hash_hex(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hasher = Sha256Hasher::new();
        let hash = hasher.hash_hex(b"hello world");
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_blake3() {
        let hasher = Blake3Hasher::new();
        let hash = hasher.hash(b"hello world");
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_password_hash_verify() {
        let hasher = PasswordHasher::new();
        let password = "super_secret_password";

        let hash = hasher.hash(password).unwrap();
        assert!(hasher.verify(password, &hash).unwrap());
        assert!(!hasher.verify("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_hasher_verify() {
        let hasher = Sha256Hasher::new();
        let data = b"test data";
        let hash = hasher.hash(data);
        assert!(hasher.verify(data, &hash));
        assert!(!hasher.verify(b"other data", &hash));
    }
}
