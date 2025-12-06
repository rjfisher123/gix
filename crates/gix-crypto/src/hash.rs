//! Cryptographic hashing using Blake3

use blake3;

/// Hash input data using Blake3, returning a 32-byte hash
pub fn hash(input: &[u8]) -> [u8; 32] {
    *blake3::hash(input).as_bytes()
}

/// Hash input data using Blake3 with a key
pub fn hash_keyed(key: &[u8; 32], input: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_keyed(key);
    hasher.update(input);
    *hasher.finalize().as_bytes()
}

/// Derive a key from input using Blake3 key derivation
///
/// The context should be a human-readable, application-specific string identifier.
/// The input is the key material from which the key will be derived.
///
/// # Arguments
/// * `context` - A string slice representing the application-specific context
/// * `input` - The key material (byte slice) from which to derive the key
///
/// # Returns
/// A 32-byte array representing the derived key
pub fn derive_key(context: &str, input: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_derive_key(context);
    hasher.update(input);
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        let input = b"test input";
        let hash1 = hash(input);
        let hash2 = hash(input);
        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        // Hash should be 32 bytes
        assert_eq!(hash1.len(), 32);
    }

    #[test]
    fn test_hash_keyed() {
        let key = [0u8; 32];
        let input = b"test input";
        let hash1 = hash_keyed(&key, input);
        let hash2 = hash_keyed(&key, input);
        // Same key and input should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_derive_key() {
        let context = "test context";
        let input = b"test input";
        let key1 = derive_key(context, input);
        let key2 = derive_key(context, input);
        // Same context and input should produce same key
        assert_eq!(key1, key2);
        // Derived key should be 32 bytes
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_derive_key_different_contexts() {
        let context1 = "context1";
        let context2 = "context2";
        let input = b"same input";
        let key1 = derive_key(context1, input);
        let key2 = derive_key(context2, input);
        // Different contexts should produce different keys
        assert_ne!(key1, key2);
    }
}
