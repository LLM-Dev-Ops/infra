/**
 * Cryptographic utilities.
 */

import { InfraError } from '../errors.js';

/**
 * Hash algorithm options.
 */
export type HashAlgorithm = 'sha256' | 'sha384' | 'sha512' | 'blake3';

/**
 * Cipher configuration.
 */
export interface CipherConfig {
  algorithm: 'aes-256-gcm';
  key: Uint8Array;
}

/**
 * Hash data using the specified algorithm.
 */
export async function hash(
  data: string | Uint8Array,
  algorithm: HashAlgorithm = 'sha256'
): Promise<Uint8Array> {
  const encoder = new TextEncoder();
  const buffer = typeof data === 'string' ? encoder.encode(data) : data;

  if (algorithm === 'blake3') {
    // Blake3 requires WASM or a JS implementation
    throw InfraError.crypto('Blake3 requires WASM module', 'hash');
  }

  const hashName = algorithm.toUpperCase().replace('SHA', 'SHA-');
  const hashBuffer = await crypto.subtle.digest(hashName, buffer.slice().buffer);
  return new Uint8Array(hashBuffer);
}

/**
 * Hash data and return as hex string.
 */
export async function hashHex(
  data: string | Uint8Array,
  algorithm: HashAlgorithm = 'sha256'
): Promise<string> {
  const hashBytes = await hash(data, algorithm);
  return bytesToHex(hashBytes);
}

/**
 * Generate a random key for encryption.
 */
export async function generateKey(): Promise<CryptoKey> {
  return crypto.subtle.generateKey(
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  );
}

/**
 * Export a CryptoKey to raw bytes.
 */
export async function exportKey(key: CryptoKey): Promise<Uint8Array> {
  const exported = await crypto.subtle.exportKey('raw', key);
  return new Uint8Array(exported);
}

/**
 * Import raw bytes as a CryptoKey.
 */
export async function importKey(keyBytes: Uint8Array): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    'raw',
    keyBytes.slice().buffer,
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  );
}

/**
 * Encrypt data with AES-256-GCM.
 */
export async function encrypt(
  data: Uint8Array,
  key: CryptoKey
): Promise<Uint8Array> {
  // Generate random IV
  const iv = crypto.getRandomValues(new Uint8Array(12));

  const ciphertext = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    key,
    data.slice().buffer
  );

  // Prepend IV to ciphertext
  const result = new Uint8Array(iv.length + ciphertext.byteLength);
  result.set(iv, 0);
  result.set(new Uint8Array(ciphertext), iv.length);

  return result;
}

/**
 * Decrypt data with AES-256-GCM.
 */
export async function decrypt(
  data: Uint8Array,
  key: CryptoKey
): Promise<Uint8Array> {
  if (data.length < 12) {
    throw InfraError.crypto('Ciphertext too short', 'decrypt');
  }

  const iv = data.slice(0, 12);
  const ciphertext = data.slice(12);

  const plaintext = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv },
    key,
    ciphertext
  );

  return new Uint8Array(plaintext);
}

/**
 * Convert bytes to hex string.
 */
export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * Convert hex string to bytes.
 */
export function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return bytes;
}

/**
 * Convert bytes to base64 string.
 */
export function bytesToBase64(bytes: Uint8Array): string {
  return btoa(String.fromCharCode(...bytes));
}

/**
 * Convert base64 string to bytes.
 */
export function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
