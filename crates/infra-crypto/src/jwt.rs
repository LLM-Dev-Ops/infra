//! JWT (JSON Web Token) support.

use chrono::{Duration, Utc};
use infra_errors::{AuthErrorKind, CryptoOperation, InfraError, InfraResult};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// JWT algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwtAlgorithm {
    HS256,
    HS384,
    HS512,
}

impl JwtAlgorithm {
    fn to_jsonwebtoken(&self) -> jsonwebtoken::Algorithm {
        match self {
            Self::HS256 => jsonwebtoken::Algorithm::HS256,
            Self::HS384 => jsonwebtoken::Algorithm::HS384,
            Self::HS512 => jsonwebtoken::Algorithm::HS512,
        }
    }
}

/// Standard JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims<T = serde_json::Value> {
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Not before (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,
    /// Subject
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    /// Issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// Audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    /// JWT ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    /// Custom payload
    #[serde(flatten)]
    pub payload: T,
}

impl<T: Default> Claims<T> {
    /// Create new claims with expiration
    #[must_use]
    pub fn new(expiry: Duration) -> Self {
        let now = Utc::now();
        Self {
            exp: (now + expiry).timestamp(),
            iat: now.timestamp(),
            nbf: None,
            sub: None,
            iss: None,
            aud: None,
            jti: None,
            payload: T::default(),
        }
    }
}

impl<T> Claims<T> {
    /// Create with custom payload
    #[must_use]
    pub fn with_payload(payload: T, expiry: Duration) -> Self {
        let now = Utc::now();
        Self {
            exp: (now + expiry).timestamp(),
            iat: now.timestamp(),
            nbf: None,
            sub: None,
            iss: None,
            aud: None,
            jti: None,
            payload,
        }
    }

    /// Set subject
    #[must_use]
    pub fn with_subject(mut self, sub: impl Into<String>) -> Self {
        self.sub = Some(sub.into());
        self
    }

    /// Set issuer
    #[must_use]
    pub fn with_issuer(mut self, iss: impl Into<String>) -> Self {
        self.iss = Some(iss.into());
        self
    }

    /// Set audience
    #[must_use]
    pub fn with_audience(mut self, aud: impl Into<String>) -> Self {
        self.aud = Some(aud.into());
        self
    }

    /// Set JWT ID
    #[must_use]
    pub fn with_jti(mut self, jti: impl Into<String>) -> Self {
        self.jti = Some(jti.into());
        self
    }

    /// Check if the token is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// JWT signer and verifier
pub struct JwtSigner {
    algorithm: JwtAlgorithm,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtSigner {
    /// Create a new JWT signer with HS256
    #[must_use]
    pub fn hs256(secret: &[u8]) -> Self {
        Self {
            algorithm: JwtAlgorithm::HS256,
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    /// Create a new JWT signer with HS384
    #[must_use]
    pub fn hs384(secret: &[u8]) -> Self {
        Self {
            algorithm: JwtAlgorithm::HS384,
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    /// Create a new JWT signer with HS512
    #[must_use]
    pub fn hs512(secret: &[u8]) -> Self {
        Self {
            algorithm: JwtAlgorithm::HS512,
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    /// Sign claims and create a JWT
    pub fn sign<T: Serialize>(&self, claims: &Claims<T>) -> InfraResult<String> {
        let header = Header::new(self.algorithm.to_jsonwebtoken());

        encode(&header, claims, &self.encoding_key).map_err(|e| InfraError::Crypto {
            operation: CryptoOperation::Sign,
            message: e.to_string(),
            context: None,
        })
    }

    /// Verify and decode a JWT
    pub fn verify<T: DeserializeOwned>(&self, token: &str) -> InfraResult<Claims<T>> {
        let validation = Validation::new(self.algorithm.to_jsonwebtoken());

        decode::<Claims<T>>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                let kind = match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        AuthErrorKind::TokenExpired
                    }
                    jsonwebtoken::errors::ErrorKind::InvalidSignature
                    | jsonwebtoken::errors::ErrorKind::InvalidToken => AuthErrorKind::InvalidToken,
                    _ => AuthErrorKind::InvalidToken,
                };

                InfraError::Auth {
                    kind,
                    message: e.to_string(),
                    identity: None,
                    context: None,
                }
            })
    }

    /// Verify without validating expiration (useful for refresh tokens)
    pub fn verify_ignore_expiry<T: DeserializeOwned>(&self, token: &str) -> InfraResult<Claims<T>> {
        let mut validation = Validation::new(self.algorithm.to_jsonwebtoken());
        validation.validate_exp = false;

        decode::<Claims<T>>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| InfraError::Auth {
                kind: AuthErrorKind::InvalidToken,
                message: e.to_string(),
                identity: None,
                context: None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
    struct TestPayload {
        user_id: String,
        role: String,
    }

    #[test]
    fn test_jwt_sign_verify() {
        let signer = JwtSigner::hs256(b"super_secret_key_at_least_32_bytes!");

        let payload = TestPayload {
            user_id: "user123".to_string(),
            role: "admin".to_string(),
        };

        let claims = Claims::with_payload(payload.clone(), Duration::hours(1))
            .with_subject("test")
            .with_issuer("infra");

        let token = signer.sign(&claims).unwrap();
        let verified: Claims<TestPayload> = signer.verify(&token).unwrap();

        assert_eq!(verified.payload, payload);
        assert_eq!(verified.sub, Some("test".to_string()));
        assert_eq!(verified.iss, Some("infra".to_string()));
    }

    #[test]
    fn test_expired_token() {
        let signer = JwtSigner::hs256(b"super_secret_key_at_least_32_bytes!");

        // Use a token that expired 120 seconds ago to ensure it's definitely expired
        // (jsonwebtoken has a default leeway of 60 seconds)
        let claims: Claims<()> = Claims::with_payload((), Duration::seconds(-120));

        let token = signer.sign(&claims).unwrap();
        let result: Result<Claims<()>, _> = signer.verify(&token);

        assert!(result.is_err());
        if let Err(InfraError::Auth { kind, .. }) = result {
            assert_eq!(kind, AuthErrorKind::TokenExpired);
        }
    }

    #[test]
    fn test_invalid_signature() {
        let signer1 = JwtSigner::hs256(b"secret_key_1_at_least_32_bytes!!");
        let signer2 = JwtSigner::hs256(b"secret_key_2_at_least_32_bytes!!");

        let claims: Claims<()> = Claims::new(Duration::hours(1));
        let token = signer1.sign(&claims).unwrap();

        let result: Result<Claims<()>, _> = signer2.verify(&token);
        assert!(result.is_err());
    }
}
