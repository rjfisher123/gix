# Cryptographic Specification

## Status

🚧 Specification in development

## Overview

GIX uses post-quantum cryptographic primitives to ensure security against both classical and quantum adversaries.

## Algorithms

### Key Encapsulation Mechanism (KEM)
- **Algorithm**: Kyber
- **Purpose**: Secure key exchange for envelope encryption

### Digital Signatures
- **Algorithm**: Dilithium
- **Purpose**: Authentication and non-repudiation

### Hashing
- **Algorithm**: Blake3
- **Purpose**: Cryptographic hashing and key derivation

## Security Requirements

- All implementations must be constant-time
- No secret leakage in logs or error messages
- Proper key management and rotation
- Side-channel resistance

## TODO

- Detailed algorithm parameters
- Key sizes and security levels
- Envelope encryption scheme
- KEM hierarchy design



