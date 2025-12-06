//! Dilithium Digital Signature - Real implementation
//!
//! This module provides post-quantum digital signatures using Dilithium3.
//! It wraps the pqcrypto-dilithium library for use in GIX.

use pqcrypto_dilithium::dilithium3;
use pqcrypto_traits::sign::{DetachedSignature as DetachedSignatureTrait, PublicKey as PublicKeyTrait, SecretKey as SecretKeyTrait};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Signature errors
#[derive(Error, Debug)]
pub enum SignatureError {
    #[error("Key generation failed")]
    KeyGenerationFailed,
    #[error("Signing failed")]
    SigningFailed,
    #[error("Verification failed")]
    VerificationFailed,
    #[error("Invalid key size: expected {expected}, got {actual}")]
    InvalidKeySize { expected: usize, actual: usize },
    #[error("Invalid signature size: expected {expected}, got {actual}")]
    InvalidSignatureSize { expected: usize, actual: usize },
}

/// Dilithium public key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    /// Public key bytes
    pub bytes: Vec<u8>,
}

impl PublicKey {
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, SignatureError> {
        let expected_size = dilithium3::public_key_bytes();
        if bytes.len() != expected_size {
            return Err(SignatureError::InvalidKeySize {
                expected: expected_size,
                actual: bytes.len(),
            });
        }
        Ok(PublicKey { bytes })
    }

    /// Get the bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to pqcrypto PublicKey type
    fn to_pqcrypto(&self) -> dilithium3::PublicKey {
        dilithium3::PublicKey::from_bytes(&self.bytes).expect("Valid public key bytes")
    }
}

/// Dilithium secret key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretKey {
    /// Secret key bytes
    pub bytes: Vec<u8>,
}

impl SecretKey {
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, SignatureError> {
        let expected_size = dilithium3::secret_key_bytes();
        if bytes.len() != expected_size {
            return Err(SignatureError::InvalidKeySize {
                expected: expected_size,
                actual: bytes.len(),
            });
        }
        Ok(SecretKey { bytes })
    }

    /// Get the bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to pqcrypto SecretKey type
    fn to_pqcrypto(&self) -> dilithium3::SecretKey {
        dilithium3::SecretKey::from_bytes(&self.bytes).expect("Valid secret key bytes")
    }
}

/// Dilithium key pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    /// Public key
    pub public: PublicKey,
    /// Secret key
    pub secret: SecretKey,
}

impl KeyPair {
    /// Generate a new key pair using Dilithium3
    pub fn generate() -> Self {
        let (pk, sk) = dilithium3::keypair();
        
        KeyPair {
            public: PublicKey {
                bytes: pk.as_bytes().to_vec(),
            },
            secret: SecretKey {
                bytes: sk.as_bytes().to_vec(),
            },
        }
    }
}

/// Dilithium signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// Signature bytes
    pub bytes: Vec<u8>,
}

impl Signature {
    /// Create from bytes
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, SignatureError> {
        let expected_size = dilithium3::signature_bytes();
        if bytes.len() != expected_size {
            return Err(SignatureError::InvalidSignatureSize {
                expected: expected_size,
                actual: bytes.len(),
            });
        }
        Ok(Signature { bytes })
    }

    /// Get the bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Convert to pqcrypto DetachedSignature type
    fn to_pqcrypto(&self) -> dilithium3::DetachedSignature {
        dilithium3::DetachedSignature::from_bytes(&self.bytes).expect("Valid signature bytes")
    }
}

/// Sign a message using a secret key
///
/// # Arguments
/// * `message` - The message to sign
/// * `secret_key` - The signer's secret key
///
/// # Returns
/// A detached signature on success
pub fn sign_detached(message: &[u8], secret_key: &SecretKey) -> Result<Signature, SignatureError> {
    let sk = secret_key.to_pqcrypto();
    let sig = dilithium3::detached_sign(message, &sk);
    
    Ok(Signature {
        bytes: sig.as_bytes().to_vec(),
    })
}

/// Verify a detached signature
///
/// # Arguments
/// * `message` - The message that was signed
/// * `signature` - The signature to verify
/// * `public_key` - The signer's public key
///
/// # Returns
/// `Ok(())` if the signature is valid, `Err` otherwise
pub fn verify_detached(
    message: &[u8],
    signature: &Signature,
    public_key: &PublicKey,
) -> Result<(), SignatureError> {
    let pk = public_key.to_pqcrypto();
    let sig = signature.to_pqcrypto();
    
    dilithium3::verify_detached_signature(&sig, message, &pk)
        .map_err(|_| SignatureError::VerificationFailed)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dilithium_keypair_generation() {
        let keypair = KeyPair::generate();
        assert_eq!(keypair.public.bytes.len(), dilithium3::public_key_bytes());
        assert_eq!(keypair.secret.bytes.len(), dilithium3::secret_key_bytes());
    }

    #[test]
    fn test_dilithium_sign_and_verify() {
        let keypair = KeyPair::generate();
        let message = b"Test message for signing";
        
        // Sign
        let signature = sign_detached(message, &keypair.secret).expect("Signing failed");
        assert_eq!(signature.bytes.len(), dilithium3::signature_bytes());
        
        // Verify
        verify_detached(message, &signature, &keypair.public).expect("Verification failed");
    }

    #[test]
    fn test_dilithium_verify_wrong_message_fails() {
        let keypair = KeyPair::generate();
        let message = b"Original message";
        let wrong_message = b"Tampered message";
        
        let signature = sign_detached(message, &keypair.secret).unwrap();
        
        // Verification with wrong message should fail
        assert!(verify_detached(wrong_message, &signature, &keypair.public).is_err());
    }

    #[test]
    fn test_dilithium_verify_wrong_key_fails() {
        let keypair1 = KeyPair::generate();
        let keypair2 = KeyPair::generate();
        let message = b"Test message";
        
        let signature = sign_detached(message, &keypair1.secret).unwrap();
        
        // Verification with wrong public key should fail
        assert!(verify_detached(message, &signature, &keypair2.public).is_err());
    }

    #[test]
    fn test_dilithium_serialization() {
        let keypair = KeyPair::generate();
        let serialized = serde_json::to_string(&keypair).unwrap();
        let deserialized: KeyPair = serde_json::from_str(&serialized).unwrap();
        assert_eq!(keypair.public.bytes, deserialized.public.bytes);
        assert_eq!(keypair.secret.bytes, deserialized.secret.bytes);
    }

    #[test]
    fn test_dilithium_signature_serialization() {
        let keypair = KeyPair::generate();
        let message = b"Test message";
        let signature = sign_detached(message, &keypair.secret).unwrap();
        
        let serialized = serde_json::to_string(&signature).unwrap();
        let deserialized: Signature = serde_json::from_str(&serialized).unwrap();
        assert_eq!(signature.bytes, deserialized.bytes);
        
        // Deserialized signature should still verify
        verify_detached(message, &deserialized, &keypair.public).expect("Verification failed");
    }
}

