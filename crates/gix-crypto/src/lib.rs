pub mod hash;
pub mod pqc;
pub mod vdf;

// Re-export commonly used functions
pub use hash::hash as hash_blake3;

// VDF exports
pub use vdf::{evaluate as vdf_evaluate, prove as vdf_prove, verify as vdf_verify, VdfProof, VdfError};

// Kyber KEM exports
pub use pqc::kyber::{
    encapsulate as kyber_encapsulate, 
    decapsulate as kyber_decapsulate, 
    KyberCiphertext,
    KyberKeyPair, 
    KyberPublicKey, 
    KyberSecretKey, 
    KyberSharedSecret,
    CryptoError as KyberError,
};

// Dilithium signature exports
pub use pqc::dilithium::{
    sign_detached as dilithium_sign,
    verify_detached as dilithium_verify,
    KeyPair as DilithiumKeyPair,
    PublicKey as DilithiumPublicKey,
    SecretKey as DilithiumSecretKey,
    Signature as DilithiumSignature,
    SignatureError as DilithiumError,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_blake3_export() {
        let input = b"test";
        let hash = hash_blake3(input);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_vdf_integration() {
        let input = b"test input";
        let proof = vdf_prove(input, 1000).expect("VDF prove failed");
        assert!(vdf_verify(input, &proof));
    }

    #[test]
    fn test_kyber_integration() {
        let keypair = KyberKeyPair::generate();
        let (ciphertext, shared_secret1) = kyber_encapsulate(&keypair.public).expect("Encapsulation failed");
        let shared_secret2 = kyber_decapsulate(&keypair.secret, &ciphertext).expect("Decapsulation failed");
        // Real Kyber: secrets should match
        assert_eq!(shared_secret1.bytes, shared_secret2.bytes);
    }

    #[test]
    fn test_dilithium_integration() {
        let keypair = DilithiumKeyPair::generate();
        let message = b"Test message";
        let signature = dilithium_sign(message, &keypair.secret).expect("Signing failed");
        dilithium_verify(message, &signature, &keypair.public).expect("Verification failed");
    }
}
