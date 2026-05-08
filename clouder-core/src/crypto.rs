//! AES-256-GCM helpers for encrypting sensitive blobs at rest.
//!
//! Storage format: a fresh 12-byte nonce per encryption, prepended to the
//! ciphertext+tag, then hex-encoded for safe SQLite TEXT storage. Reusing a
//! (key, nonce) pair under GCM is catastrophic, so the nonce is generated
//! from the OS CSPRNG every call and never written by the caller.

// `Key::from_slice` and `Nonce::from_slice` are flagged deprecated in aes-gcm
// 0.10 because they reference generic-array 0.x; 0.11 (with generic-array 1.x)
// isn't released yet. Behavior is unchanged, so we silence the noise here.
#![allow(deprecated)]

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{Result, anyhow};
use rand::Rng;

const NONCE_LEN: usize = 12;

fn cipher(key_bytes: &[u8; 32]) -> Aes256Gcm {
    Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key_bytes))
}

fn random_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce);
    nonce
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn from_hex(hex: &str) -> Result<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return Err(anyhow!("invalid hex length"));
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| anyhow!(e)))
        .collect()
}

/// Encrypts `plaintext` with the given 32-byte key. Output is hex-encoded
/// `nonce || ciphertext_with_tag`, suitable for SQLite TEXT storage.
pub fn encrypt(key_bytes: &[u8; 32], plaintext: &[u8]) -> Result<String> {
    let nonce_bytes = random_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher(key_bytes)
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow!("aes-gcm encrypt: {}", e))?;
    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(to_hex(&out))
}

/// Decrypts the hex-encoded `nonce || ciphertext_with_tag` produced by
/// [`encrypt`]. Returns an error on tampering, key mismatch, or malformed input.
pub fn decrypt(key_bytes: &[u8; 32], hex_blob: &str) -> Result<Vec<u8>> {
    let bytes = from_hex(hex_blob)?;
    if bytes.len() < NONCE_LEN {
        return Err(anyhow!("ciphertext too short"));
    }
    let (nonce_bytes, ciphertext) = bytes.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher(key_bytes)
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("aes-gcm decrypt: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = [7u8; 32];
        let ct = encrypt(&key, b"hello world").unwrap();
        assert_eq!(decrypt(&key, &ct).unwrap(), b"hello world");
    }

    #[test]
    fn wrong_key_fails() {
        let ct = encrypt(&[1u8; 32], b"secret").unwrap();
        assert!(decrypt(&[2u8; 32], &ct).is_err());
    }

    #[test]
    fn tampered_fails() {
        let key = [3u8; 32];
        let mut ct = encrypt(&key, b"secret").unwrap();
        // Flip a byte in the ciphertext (after the nonce).
        ct.replace_range(40..42, "00");
        assert!(decrypt(&key, &ct).is_err());
    }

    #[test]
    fn distinct_nonces() {
        let key = [9u8; 32];
        let a = encrypt(&key, b"same").unwrap();
        let b = encrypt(&key, b"same").unwrap();
        assert_ne!(a, b, "nonce must vary per call");
    }
}
