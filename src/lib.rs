//! Kernel-level encryption framework for SigmaOS.
//!
//! Provides encryption key management, cipher operations, and secure data
//! handling. Simulates kernel encryption without actual hardware crypto.

use std::collections::HashMap;
use std::fmt;

/// Errors for encryption operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionError {
    InvalidKey(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
    KeyNotFound(String),
    InvalidAlgorithm(String),
}

impl fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey(m) => write!(f, "invalid key: {m}"),
            Self::EncryptionFailed(m) => write!(f, "encryption failed: {m}"),
            Self::DecryptionFailed(m) => write!(f, "decryption failed: {m}"),
            Self::KeyNotFound(k) => write!(f, "key not found: {k}"),
            Self::InvalidAlgorithm(a) => write!(f, "invalid algorithm: {a}"),
        }
    }
}

impl std::error::Error for EncryptionError {}

/// Encryption algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Algorithm {
    AES256,
    AES128,
    ChaCha20,
    XChaCha20,
}

impl Algorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AES256 => "AES-256",
            Self::AES128 => "AES-128",
            Self::ChaCha20 => "ChaCha20",
            Self::XChaCha20 => "XChaCha20",
        }
    }
}

/// Encryption key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionKey {
    pub id: String,
    pub algorithm: Algorithm,
    pub material: Vec<u8>,
    pub created_at: u64,
}

impl EncryptionKey {
    pub fn new(id: impl Into<String>, algorithm: Algorithm, material: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            algorithm,
            material,
            created_at: 0,
        }
    }

    pub fn validate(&self) -> Result<(), EncryptionError> {
        let expected_len = match self.algorithm {
            Algorithm::AES256 => 32,
            Algorithm::AES128 => 16,
            Algorithm::ChaCha20 => 32,
            Algorithm::XChaCha20 => 32,
        };
        if self.material.len() != expected_len {
            return Err(EncryptionError::InvalidKey(format!(
                "expected {} bytes for {:?}, got {}",
                expected_len, self.algorithm, self.material.len()
            )));
        }
        Ok(())
    }
}

/// Encrypted data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub key_id: String,
    pub algorithm: Algorithm,
}

/// Kernel encryption manager.
#[derive(Debug, Clone, Default)]
pub struct EncryptionManager {
    keys: HashMap<String, EncryptionKey>,
    next_id: u64,
}

impl EncryptionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_key(&mut self, mut key: EncryptionKey) -> Result<(), EncryptionError> {
        key.validate()?;
        if self.keys.contains_key(&key.id) {
            return Err(EncryptionError::InvalidKey(format!("duplicate key id: {}", key.id)));
        }
        self.next_id += 1;
        key.created_at = self.next_id;
        self.keys.insert(key.id.clone(), key);
        Ok(())
    }

    pub fn remove_key(&mut self, id: &str) -> Result<(), EncryptionError> {
        self.keys.remove(id)
            .map(|_| ())
            .ok_or_else(|| EncryptionError::KeyNotFound(id.to_string()))
    }

    pub fn get_key(&self, id: &str) -> Option<&EncryptionKey> {
        self.keys.get(id)
    }

    pub fn list_keys(&self) -> Vec<&EncryptionKey> {
        self.keys.values().collect()
    }

    /// Simulate encryption by XOR with key material.
    pub fn encrypt(&self, key_id: &str, plaintext: &[u8]) -> Result<EncryptedData, EncryptionError> {
        let key = self.keys.get(key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound(key_id.to_string()))?;

        let nonce = vec![0u8; 12];
        let ciphertext: Vec<u8> = plaintext.iter()
            .enumerate()
            .map(|(i, b)| b ^ key.material[i % key.material.len()])
            .collect();

        Ok(EncryptedData {
            ciphertext,
            nonce,
            key_id: key_id.to_string(),
            algorithm: key.algorithm,
        })
    }

    /// Simulate decryption by XOR with key material.
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, EncryptionError> {
        let key = self.keys.get(&encrypted.key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound(encrypted.key_id.clone()))?;

        if key.algorithm != encrypted.algorithm {
            return Err(EncryptionError::DecryptionFailed("algorithm mismatch".into()));
        }

        let plaintext: Vec<u8> = encrypted.ciphertext.iter()
            .enumerate()
            .map(|(i, b)| b ^ key.material[i % key.material.len()])
            .collect();

        Ok(plaintext)
    }

    /// Generate a random-looking key material (deterministic for testing).
    pub fn generate_key_material(&self, algorithm: Algorithm, seed: u64) -> Vec<u8> {
        let len = match algorithm {
            Algorithm::AES256 => 32,
            Algorithm::AES128 => 16,
            Algorithm::ChaCha20 => 32,
            Algorithm::XChaCha20 => 32,
        };
        (0..len).map(|i| ((seed.wrapping_mul(31).wrapping_add(i as u64)) & 0xFF) as u8).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_key_aaa() {
        // Arrange
        let mut mgr = EncryptionManager::new();
        let material = mgr.generate_key_material(Algorithm::AES256, 42);
        let key = EncryptionKey::new("key1", Algorithm::AES256, material);

        // Act
        let res = mgr.add_key(key);

        // Assert
        assert!(res.is_ok());
        assert!(mgr.get_key("key1").is_some());
    }

    #[test]
    fn test_encrypt_decrypt_aaa() {
        // Arrange
        let mut mgr = EncryptionManager::new();
        let material = mgr.generate_key_material(Algorithm::AES256, 42);
        let key = EncryptionKey::new("key1", Algorithm::AES256, material);
        mgr.add_key(key).unwrap();

        // Act
        let plaintext = b"secret data";
        let encrypted = mgr.encrypt("key1", plaintext).unwrap();
        let decrypted = mgr.decrypt(&encrypted).unwrap();

        // Assert
        assert_ne!(encrypted.ciphertext, plaintext.to_vec());
        assert_eq!(decrypted, plaintext.to_vec());
    }

    #[test]
    fn test_key_not_found_aaa() {
        // Arrange
        let mgr = EncryptionManager::new();

        // Act
        let res = mgr.encrypt("nonexistent", b"data");

        // Assert
        assert!(matches!(res, Err(EncryptionError::KeyNotFound(_))));
    }
}
