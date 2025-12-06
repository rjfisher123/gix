# GIX Shared Libraries

This directory contains shared Rust libraries (crates) used across GIX components.

## Crates

- **gix-crypto**: Post-quantum cryptographic primitives (Kyber, Dilithium, Blake3)
- **gix-common**: Shared types (`JobId`, `LaneId`), error types, utilities
- **gix-gxf**: GXF schema, validators, serialization
- **gix-testing**: Mocks, test vectors, property testing utilities

## Dependency Rules

Crates follow a reverse-pyramid dependency structure:
- `gix-crypto` has no GIX dependencies (only external crypto libraries)
- `gix-gxf` depends on `gix-common` and `gix-crypto`
- `gix-common` depends on `gix-crypto`
- `gix-testing` depends on all other crates

Services and tools depend on these crates, never the reverse.




