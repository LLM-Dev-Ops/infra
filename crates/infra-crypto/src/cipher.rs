//! Symmetric encryption implementations.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use infra_errors::{CryptoOperation, InfraError, InfraResult};
use rand::RngCore;

/// Trait for symmetric ciphers
pub trait Cipher: Send + Sync {
    /// Encrypt plaintext
    fn encrypt(&self, plaintext: &[u8]) -> InfraResult<Vec<u8>>;

    /// Decrypt ciphertext
    fn decrypt(&self, ciphertext: &[u8]) -> InfraResult<Vec<u8>>;
}

/// AES-256-GCM cipher
pub struct Aes256GcmCipher {
    key: [u8; 32],
}

impl Aes256GcmCipher {
    /// Create a new cipher with the given key
    #[must_use]
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Generate a new cipher with a random key
    pub fn generate() -> InfraResult<Self> {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Ok(Self { key })
    }

    /// Create from a byte slice (must be 32 bytes)
    pub fn from_bytes(bytes: &[u8]) -> InfraResult<Self> {
        if bytes.len() != 32 {
            return Err(InfraError::Crypto {
                operation: CryptoOperation::KeyGeneration,
                message: format!("Key must be 32 bytes, got {}", bytes.len()),
                context: None,
            });
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);
        Ok(Self { key })
    }

    /// Derive a key from a passphrase using Argon2
    pub fn from_passphrase(passphrase: &str, salt: &[u8]) -> InfraResult<Self> {
        use argon2::Argon2;

        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(passphrase.as_bytes(), salt, &mut key)
            .map_err(|e| InfraError::Crypto {
                operation: CryptoOperation::KeyDerivation,
                message: e.to_string(),
                context: None,
            })?;

        Ok(Self { key })
    }

    /// Get the key (use carefully)
    #[must_use]
    pub fn key(&self) -> &[u8; 32] {
        &self.key
    }

    /// Export key as base64
    #[must_use]
    pub fn key_base64(&self) -> String {
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &self.key)
    }

    /// Import key from base64
    pub fn from_base64(encoded: &str) -> InfraResult<Self> {
        let key_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            .map_err(|e| InfraError::Crypto {
                operation: CryptoOperation::KeyGeneration,
                message: format!("Invalid base64: {e}"),
                context: None,
            })?;
        Self::from_bytes(&key_bytes)
    }
}

impl Cipher for Aes256GcmCipher {
    fn encrypt(&self, plaintext: &[u8]) -> InfraResult<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key).map_err(|e| InfraError::Crypto {
            operation: CryptoOperation::Encrypt,
            message: e.to_string(),
            context: None,
        })?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|e| InfraError::Crypto {
            operation: CryptoOperation::Encrypt,
            message: e.to_string(),
            context: None,
        })?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Ok(result)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> InfraResult<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(InfraError::Crypto {
                operation: CryptoOperation::Decrypt,
                message: "Ciphertext too short (missing nonce)".to_string(),
                context: None,
            });
        }

        let cipher = Aes256Gcm::new_from_slice(&self.key).map_err(|e| InfraError::Crypto {
            operation: CryptoOperation::Decrypt,
            message: e.to_string(),
            context: None,
        })?;

        let nonce = Nonce::from_slice(&ciphertext[..12]);

        cipher
            .decrypt(nonce, &ciphertext[12..])
            .map_err(|e| InfraError::Crypto {
                operation: CryptoOperation::Decrypt,
                message: e.to_string(),
                context: None,
            })
    }
}

impl Clone for Aes256GcmCipher {
    fn clone(&self) -> Self {
        Self { key: self.key }
    }
}

// Implement zeroize on drop for security
impl Drop for Aes256GcmCipher {
    fn drop(&mut self) {
        // Zero out the key
        self.key.iter_mut().for_each(|b| *b = 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let cipher = Aes256GcmCipher::generate().unwrap();
        let plaintext = b"Hello, World!";

        let ciphertext = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_different_ciphertexts() {
        let cipher = Aes256GcmCipher::generate().unwrap();
        let plaintext = b"Hello, World!";

        let ct1 = cipher.encrypt(plaintext).unwrap();
        let ct2 = cipher.encrypt(plaintext).unwrap();

        // Should produce different ciphertexts due to random nonce
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_from_passphrase() {
        let salt = b"random_salt_value_here!";
        let cipher = Aes256GcmCipher::from_passphrase("my_password", salt).unwrap();

        let plaintext = b"secret data";
        let ciphertext = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_invalid_ciphertext() {
        let cipher = Aes256GcmCipher::generate().unwrap();

        // Too short
        assert!(cipher.decrypt(b"short").is_err());

        // Invalid data
        let invalid = vec![0u8; 100];
        assert!(cipher.decrypt(&invalid).is_err());
    }
}
