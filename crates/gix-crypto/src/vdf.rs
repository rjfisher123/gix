//! Verifiable Delay Function (VDF) - Wesolowski implementation
//!
//! This module provides a real VDF using the Wesolowski construction.
//! Note: VDF computation is intentionally slow and cannot be parallelized.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use vdf::{InvalidProof, VDFParams, WesolowskiVDFParams, VDF};

/// VDF errors
#[derive(Error, Debug)]
pub enum VdfError {
    #[error("VDF evaluation failed")]
    EvaluationFailed,
    #[error("VDF verification failed: {0}")]
    VerificationFailed(String),
    #[error("Invalid proof")]
    InvalidProof,
}

/// VDF proof structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VdfProof {
    /// The output after VDF computation
    pub output: Vec<u8>,
    /// The proof that enables fast verification
    pub proof: Vec<u8>,
    /// Number of iterations (difficulty)
    pub iterations: u64,
}

impl VdfProof {
    /// Create a new VDF proof
    pub fn new(output: Vec<u8>, proof: Vec<u8>, iterations: u64) -> Self {
        VdfProof {
            output,
            proof,
            iterations,
        }
    }

    /// Get the output
    pub fn output(&self) -> &[u8] {
        &self.output
    }
}

/// Evaluate a VDF (solve the delay function)
///
/// # Arguments
/// * `input` - The challenge/seed
/// * `iterations` - The time parameter (difficulty)
///
/// # Returns
/// The output after the delay
///
/// # Warning
/// This function is intentionally slow! It performs sequential computation
/// that cannot be parallelized. Higher iteration counts will take longer.
pub fn evaluate(input: &[u8], iterations: u64) -> Result<Vec<u8>, VdfError> {
    // Convert input to challenge format
    let challenge = blake3::hash(input);
    
    // Create VDF parameters with appropriate security level
    // Using 2048-bit RSA modulus for security
    let params = WesolowskiVDFParams(2048).new();
    
    // Solve the VDF (this is the slow part!)
    let result = params.solve(challenge.as_bytes(), iterations)
        .map_err(|_| VdfError::EvaluationFailed)?;
    
    Ok(result.to_vec())
}

/// Generate a VDF proof
///
/// # Arguments
/// * `input` - The challenge/seed
/// * `iterations` - The time parameter (difficulty)
///
/// # Returns
/// A VDF proof that includes the output and a succinct proof for fast verification
///
/// # Warning
/// This function performs the full VDF computation and is intentionally slow!
pub fn prove(input: &[u8], iterations: u64) -> Result<VdfProof, VdfError> {
    // Convert input to challenge format
    let challenge = blake3::hash(input);
    
    // Create VDF parameters
    let params = WesolowskiVDFParams(2048).new();
    
    // Solve the VDF and generate proof
    let output = params.solve(challenge.as_bytes(), iterations)
        .map_err(|_| VdfError::EvaluationFailed)?;
    
    // Generate proof for fast verification
    let proof = params.prove(challenge.as_bytes(), iterations, &output)
        .map_err(|_| VdfError::EvaluationFailed)?;
    
    Ok(VdfProof::new(output.to_vec(), proof.to_vec(), iterations))
}

/// Verify a VDF proof
///
/// # Arguments
/// * `input` - The original challenge/seed
/// * `proof` - The VDF proof to verify
///
/// # Returns
/// `true` if the proof is valid, `false` otherwise
///
/// # Note
/// Verification is much faster than solving! This is the key property of VDFs.
pub fn verify(input: &[u8], vdf_proof: &VdfProof) -> bool {
    // Convert input to challenge format
    let challenge = blake3::hash(input);
    
    // Create VDF parameters (must match those used for proving)
    let params = WesolowskiVDFParams(2048).new();
    
    // Verify the proof
    match params.verify(
        challenge.as_bytes(),
        vdf_proof.iterations,
        &vdf_proof.output,
        &vdf_proof.proof,
    ) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vdf_evaluate() {
        let input = b"test input";
        let iterations = 1000; // Low iteration count for faster tests
        
        let output = evaluate(input, iterations).expect("VDF evaluation failed");
        assert!(!output.is_empty());
    }

    #[test]
    fn test_vdf_prove_and_verify() {
        let input = b"test input";
        let iterations = 1000; // Low iteration count for faster tests
        
        // Generate proof
        let proof = prove(input, iterations).expect("VDF proving failed");
        assert!(!proof.output.is_empty());
        assert!(!proof.proof.is_empty());
        assert_eq!(proof.iterations, iterations);
        
        // Verify proof
        assert!(verify(input, &proof), "VDF verification failed");
    }

    #[test]
    fn test_vdf_verify_wrong_input_fails() {
        let input = b"test input";
        let wrong_input = b"wrong input";
        let iterations = 1000;
        
        let proof = prove(input, iterations).unwrap();
        
        // Verification with wrong input should fail
        assert!(!verify(wrong_input, &proof));
    }

    #[test]
    fn test_vdf_deterministic() {
        let input = b"test input";
        let iterations = 1000;
        
        let output1 = evaluate(input, iterations).unwrap();
        let output2 = evaluate(input, iterations).unwrap();
        
        // Same input and iterations should produce same output
        assert_eq!(output1, output2);
    }

    #[test]
    fn test_vdf_different_iterations() {
        let input = b"test input";
        
        let proof1 = prove(input, 1000).unwrap();
        let proof2 = prove(input, 2000).unwrap();
        
        // Different iterations should produce different outputs
        assert_ne!(proof1.output, proof2.output);
    }

    #[test]
    fn test_vdf_serialization() {
        let input = b"test input";
        let iterations = 1000;
        
        let proof = prove(input, iterations).unwrap();
        
        // Serialize and deserialize
        let serialized = serde_json::to_string(&proof).unwrap();
        let deserialized: VdfProof = serde_json::from_str(&serialized).unwrap();
        
        // Deserialized proof should still verify
        assert!(verify(input, &deserialized));
    }
}
