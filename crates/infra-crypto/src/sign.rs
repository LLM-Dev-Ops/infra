//! Digital signature implementations.

use ed25519_dalek::{Signer as DalekSigner, SigningKey, Verifier as DalekVerifier, VerifyingKey};
use infra_errors::{CryptoOperation, InfraError, InfraResult};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

/// Trait for digital signature creation
pub trait Signer: Send + Sync {
    /// Sign data
    fn sign(&self, data: &[u8]) -> InfraResult<Signature>;

    /// Get the public key
    fn public_key(&self) -> PublicKey;
}

/// Trait for signature verification
pub trait Verifier: Send + Sync {
    /// Verify a signature
    fn verify(&self, data: &[u8], signature: &Signature) -> InfraResult<bool>;
}

/// Digital signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature(#[serde(with = "hex_serde")] Vec<u8>);

impl Signature {
    /// Create from bytes
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Get as bytes
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to hex string
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Parse from hex string
    pub fn from_hex(hex_str: &str) -> InfraResult<Self> {
        hex::decode(hex_str)
            .map(Self)
            .map_err(|e| InfraError::Crypto {
                operation: CryptoOperation::Verify,
                message: format!("Invalid hex: {e}"),
                context: None,
            })
    }
}

/// Public key for signature verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey(#[serde(with = "hex_serde")] Vec<u8>);

impl PublicKey {
    /// Create from bytes
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Get as bytes
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to hex string
    #[must_use]
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Parse from hex string
    pub fn from_hex(hex_str: &str) -> InfraResult<Self> {
        hex::decode(hex_str)
            .map(Self)
            .map_err(|e| InfraError::Crypto {
                operation: CryptoOperation::Verify,
                message: format!("Invalid hex: {e}"),
                context: None,
            })
    }
}

mod hex_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex::decode(s).map_err(serde::de::Error::custom)
    }
}

/// Ed25519 keypair
#[derive(Clone)]
pub struct Keypair {
    signing_key: SigningKey,
}

impl Keypair {
    /// Generate a new random keypair
    #[must_use]
    pub fn generate() -> Self {
        Self {
            signing_key: SigningKey::generate(&mut OsRng),
        }
    }

    /// Create from secret key bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> InfraResult<Self> {
        Ok(Self {
            signing_key: SigningKey::from_bytes(bytes),
        })
    }

    /// Get the secret key bytes
    #[must_use]
    pub fn secret_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }

    /// Get the public key
    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.signing_key.verifying_key().to_bytes().to_vec())
    }

    /// Create a signer from this keypair
    #[must_use]
    pub fn signer(&self) -> Ed25519Signer {
        Ed25519Signer {
            signing_key: self.signing_key.clone(),
        }
    }

    /// Create a verifier from this keypair
    pub fn verifier(&self) -> InfraResult<Ed25519Verifier> {
        Ed25519Verifier::from_public_key(&self.public_key())
    }
}

/// Ed25519 signer
pub struct Ed25519Signer {
    signing_key: SigningKey,
}

impl Ed25519Signer {
    /// Generate a new signer with random key
    #[must_use]
    pub fn generate() -> Self {
        Self {
            signing_key: SigningKey::generate(&mut OsRng),
        }
    }

    /// Create from secret key bytes
    pub fn from_bytes(bytes: &[u8; 32]) -> InfraResult<Self> {
        Ok(Self {
            signing_key: SigningKey::from_bytes(bytes),
        })
    }
}

impl Signer for Ed25519Signer {
    fn sign(&self, data: &[u8]) -> InfraResult<Signature> {
        let sig = self.signing_key.sign(data);
        Ok(Signature(sig.to_bytes().to_vec()))
    }

    fn public_key(&self) -> PublicKey {
        PublicKey(self.signing_key.verifying_key().to_bytes().to_vec())
    }
}

/// Ed25519 verifier
pub struct Ed25519Verifier {
    verifying_key: VerifyingKey,
}

impl Ed25519Verifier {
    /// Create from public key
    pub fn from_public_key(public_key: &PublicKey) -> InfraResult<Self> {
        let bytes: [u8; 32] = public_key
            .as_bytes()
            .try_into()
            .map_err(|_| InfraError::Crypto {
                operation: CryptoOperation::Verify,
                message: "Invalid public key length".to_string(),
                context: None,
            })?;

        let verifying_key = VerifyingKey::from_bytes(&bytes).map_err(|e| InfraError::Crypto {
            operation: CryptoOperation::Verify,
            message: e.to_string(),
            context: None,
        })?;

        Ok(Self { verifying_key })
    }
}

impl Verifier for Ed25519Verifier {
    fn verify(&self, data: &[u8], signature: &Signature) -> InfraResult<bool> {
        let sig_bytes: [u8; 64] =
            signature
                .as_bytes()
                .try_into()
                .map_err(|_| InfraError::Crypto {
                    operation: CryptoOperation::Verify,
                    message: "Invalid signature length".to_string(),
                    context: None,
                })?;

        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);

        Ok(self.verifying_key.verify(data, &sig).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_verify() {
        let keypair = Keypair::generate();
        let signer = keypair.signer();
        let verifier = keypair.verifier().unwrap();

        let data = b"Hello, World!";
        let signature = signer.sign(data).unwrap();

        assert!(verifier.verify(data, &signature).unwrap());
        assert!(!verifier.verify(b"Different data", &signature).unwrap());
    }

    #[test]
    fn test_keypair_roundtrip() {
        let keypair = Keypair::generate();
        let secret_bytes = keypair.secret_bytes();

        let restored = Keypair::from_bytes(&secret_bytes).unwrap();
        assert_eq!(keypair.public_key(), restored.public_key());
    }

    #[test]
    fn test_signature_hex_roundtrip() {
        let keypair = Keypair::generate();
        let signer = keypair.signer();

        let signature = signer.sign(b"test").unwrap();
        let hex = signature.to_hex();
        let restored = Signature::from_hex(&hex).unwrap();

        assert_eq!(signature, restored);
    }
}
