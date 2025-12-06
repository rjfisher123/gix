//! Kyber KEM (Key Encapsulation Mechanism) - Real implementation
//!
//! This module provides post-quantum key encapsulation using Kyber1024.
//! It wraps the pqcrypto-kyber library for use in GIX.

use pqcrypto_kyber::kyber1024;
use pqcrypto_traits::kem::{Ciphertext as CiphertextTrait, PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait, SharedSecret as SharedSecretTrait};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Cryptography errors
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key generation failed")]
    KeyGenerationFailed,
    #[error("Encapsulation failed")]
    EncapsulationFailed,
    #[error("Decapsulation failed")]
    DecapsulationFailed,
    #[error("Invalid key size: expected {expected}, got {actual}")]
    InvalidKeySize { expected: usize, actual: usize },
}

/// Kyber public key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KyberPublicKey {
    /// Public key bytes
    pub bytes: Vec<u8>,
}

impl KyberPublicKey {
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, CryptoError> {
        let expected_size = kyber1024::public_key_bytes();
        if bytes.len() != expected_size {
            return Err(CryptoError::InvalidKeySize {
                expected: expected_size,
                actual: bytes.len(),
            });
        }
        Ok(KyberPublicKey { bytes })
    }

    /// Get the bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to pqcrypto PublicKey type
    fn to_pqcrypto(&self) -> kyber1024::PublicKey {
        kyber1024::PublicKey::from_bytes(&self.bytes).expect("Valid public key bytes")
    }
}

/// Kyber secret key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KyberSecretKey {
    /// Secret key bytes
    pub bytes: Vec<u8>,
}

impl KyberSecretKey {
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, CryptoError> {
        let expected_size = kyber1024::secret_key_bytes();
        if bytes.len() != expected_size {
            return Err(CryptoError::InvalidKeySize {
                expected: expected_size,
                actual: bytes.len(),
            });
        }
        Ok(KyberSecretKey { bytes })
    }

    /// Get the bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to pqcrypto SecretKey type
    fn to_pqcrypto(&self) -> kyber1024::SecretKey {
        kyber1024::SecretKey::from_bytes(&self.bytes).expect("Valid secret key bytes")
    }
}

/// Kyber key pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KyberKeyPair {
    /// Public key
    pub public: KyberPublicKey,
    /// Secret key
    pub secret: KyberSecretKey,
}

impl KyberKeyPair {
    /// Generate a new key pair using Kyber1024
    pub fn generate() -> Self {
        let (pk, sk) = kyber1024::keypair();
        
        KyberKeyPair {
            public: KyberPublicKey {
                bytes: pk.as_bytes().to_vec(),
            },
            secret: KyberSecretKey {
                bytes: sk.as_bytes().to_vec(),
            },
        }
    }
}

/// Kyber ciphertext
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KyberCiphertext {
    /// Ciphertext bytes
    pub bytes: Vec<u8>,
}

impl KyberCiphertext {
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, CryptoError> {
        let expected_size = kyber1024::ciphertext_bytes();
        if bytes.len() != expected_size {
            return Err(CryptoError::InvalidKeySize {
                expected: expected_size,
                actual: bytes.len(),
            });
        }
        Ok(KyberCiphertext { bytes })
    }

    /// Get the bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to pqcrypto Ciphertext type
    fn to_pqcrypto(&self) -> kyber1024::Ciphertext {
        kyber1024::Ciphertext::from_bytes(&self.bytes).expect("Valid ciphertext bytes")
    }
}

/// Kyber shared secret
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KyberSharedSecret {
    /// Shared secret bytes
    pub bytes: Vec<u8>,
}

impl KyberSharedSecret {
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, CryptoError> {
        let expected_size = kyber1024::shared_secret_bytes();
        if bytes.len() != expected_size {
            return Err(CryptoError::InvalidKeySize {
                expected: expected_size,
                actual: bytes.len(),
            });
        }
        Ok(KyberSharedSecret { bytes })
    }

    /// Get the bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Encapsulate a shared secret using a public key
///
/// # Arguments
/// * `public_key` - The recipient's public key
///
/// # Returns
/// A tuple of (ciphertext, shared_secret) on success
pub fn encapsulate(public_key: &KyberPublicKey) -> Result<(KyberCiphertext, KyberSharedSecret), CryptoError> {
    let pk = public_key.to_pqcrypto();
    let (ss, ct) = kyber1024::encapsulate(&pk);
    
    Ok((
        KyberCiphertext {
            bytes: ct.as_bytes().to_vec(),
        },
        KyberSharedSecret {
            bytes: ss.as_bytes().to_vec(),
        },
    ))
}

/// Decapsulate a shared secret using a secret key and ciphertext
///
/// # Arguments
/// * `secret_key` - The recipient's secret key
/// * `ciphertext` - The encapsulated ciphertext
///
/// # Returns
/// The shared secret on success
pub fn decapsulate(
    secret_key: &KyberSecretKey,
    ciphertext: &KyberCiphertext,
) -> Result<KyberSharedSecret, CryptoError> {
    let sk = secret_key.to_pqcrypto();
    let ct = ciphertext.to_pqcrypto();
    
    let ss = kyber1024::decapsulate(&ct, &sk);
    
    Ok(KyberSharedSecret {
        bytes: ss.as_bytes().to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kyber_keypair_generation() {
        let keypair = KyberKeyPair::generate();
        assert_eq!(keypair.public.bytes.len(), kyber1024::public_key_bytes());
        assert_eq!(keypair.secret.bytes.len(), kyber1024::secret_key_bytes());
    }

    #[test]
    fn test_kyber_encapsulate_decapsulate() {
        let keypair = KyberKeyPair::generate();
        
        // Encapsulate
        let (ciphertext, shared_secret1) = encapsulate(&keypair.public).expect("Encapsulation failed");
        assert_eq!(ciphertext.bytes.len(), kyber1024::ciphertext_bytes());
        assert_eq!(shared_secret1.bytes.len(), kyber1024::shared_secret_bytes());
        
        // Decapsulate
        let shared_secret2 = decapsulate(&keypair.secret, &ciphertext).expect("Decapsulation failed");
        assert_eq!(shared_secret2.bytes.len(), kyber1024::shared_secret_bytes());
        
        // Shared secrets should match
        assert_eq!(shared_secret1.bytes, shared_secret2.bytes);
    }

    #[test]
    fn test_kyber_serialization() {
        let keypair = KyberKeyPair::generate();
        let serialized = serde_json::to_string(&keypair).unwrap();
        let deserialized: KyberKeyPair = serde_json::from_str(&serialized).unwrap();
        assert_eq!(keypair.public.bytes, deserialized.public.bytes);
        assert_eq!(keypair.secret.bytes, deserialized.secret.bytes);
    }

    #[test]
    fn test_kyber_different_keypairs_different_secrets() {
        let keypair1 = KyberKeyPair::generate();
        let keypair2 = KyberKeyPair::generate();
        
        let (ct1, ss1) = encapsulate(&keypair1.public).unwrap();
        let (ct2, ss2) = encapsulate(&keypair2.public).unwrap();
        
        // Different public keys should produce different ciphertexts and secrets
        assert_ne!(ct1.bytes, ct2.bytes);
        assert_ne!(ss1.bytes, ss2.bytes);
    }

    #[test]
    fn test_kyber_wrong_key_different_secret() {
        let keypair1 = KyberKeyPair::generate();
        let keypair2 = KyberKeyPair::generate();
        
        let (ciphertext, shared_secret1) = encapsulate(&keypair1.public).unwrap();
        
        // Decapsulating with wrong key should give different secret
        let shared_secret2 = decapsulate(&keypair2.secret, &ciphertext).unwrap();
        assert_ne!(shared_secret1.bytes, shared_secret2.bytes);
    }
}
