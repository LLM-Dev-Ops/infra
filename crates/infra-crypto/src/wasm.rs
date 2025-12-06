//! WASM bindings for crypto operations.

use crate::hash::{blake3_hex, sha256_hex, Blake3Hasher, Hasher, Sha256Hasher};
use crate::cipher::{Aes256GcmCipher, Cipher};
use wasm_bindgen::prelude::*;

/// Hash data with SHA-256 and return hex
#[wasm_bindgen(js_name = sha256Hex)]
pub fn js_sha256_hex(data: &[u8]) -> String {
    sha256_hex(data)
}

/// Hash string with SHA-256 and return hex
#[wasm_bindgen(js_name = sha256String)]
pub fn js_sha256_string(data: &str) -> String {
    sha256_hex(data.as_bytes())
}

/// Hash data with Blake3 and return hex
#[wasm_bindgen(js_name = blake3Hex)]
pub fn js_blake3_hex(data: &[u8]) -> String {
    blake3_hex(data)
}

/// Hash string with Blake3 and return hex
#[wasm_bindgen(js_name = blake3String)]
pub fn js_blake3_string(data: &str) -> String {
    blake3_hex(data.as_bytes())
}

/// AES-256-GCM cipher for WASM
#[wasm_bindgen]
pub struct JsAes256Gcm {
    cipher: Aes256GcmCipher,
}

#[wasm_bindgen]
impl JsAes256Gcm {
    /// Create a new cipher with a random key
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<JsAes256Gcm, JsValue> {
        Aes256GcmCipher::generate()
            .map(|cipher| JsAes256Gcm { cipher })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create from a 32-byte key
    #[wasm_bindgen(js_name = fromKey)]
    pub fn from_key(key: &[u8]) -> Result<JsAes256Gcm, JsValue> {
        Aes256GcmCipher::from_bytes(key)
            .map(|cipher| JsAes256Gcm { cipher })
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Encrypt data
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, JsValue> {
        self.cipher
            .encrypt(plaintext)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Decrypt data
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, JsValue> {
        self.cipher
            .decrypt(ciphertext)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get the key as base64
    #[wasm_bindgen(js_name = keyBase64)]
    pub fn key_base64(&self) -> String {
        self.cipher.key_base64()
    }
}

impl Default for JsAes256Gcm {
    fn default() -> Self {
        Self::new().expect("Failed to create cipher")
    }
}
