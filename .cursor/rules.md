# GIX Development Rules and Guidelines

## Architecture Philosophy

### Reverse-Pyramid Methodology

GIX follows a **reverse-pyramid** development approach, building from the foundation upward:

1. **Crypto Layer** (`gix-crypto`) - Post-quantum cryptographic primitives
2. **GXF Layer** (`gix-gxf`) - Exchange format schema and validation
3. **Common Layer** (`gix-common`) - Shared types and utilities
4. **Service Layer** (`services/*`) - Runnable daemons and services

**Rule**: Dependencies must flow downward only. Services depend on common/GXF/crypto, never the reverse.

## Coding Standards

### Rust Style

- **Edition**: Use Rust 2021 edition
- **Formatting**: Always run `cargo fmt` before committing
- **Linting**: All code must pass `cargo clippy` with no warnings
- **Documentation**: All public APIs must have doc comments

### Security Invariants

1. **No `unsafe` Rust**: All code must be safe Rust. If unsafe is required, it must be:
   - Isolated in a well-reviewed, minimal module
   - Documented with safety invariants
   - Approved by security review

2. **No Secret Leakage**: 
   - Never log secrets, keys, or sensitive data
   - Use constant-time operations for cryptographic code
   - Clear sensitive data from memory when done

3. **Input Validation**: 
   - Validate all inputs at boundaries
   - Use strong types (e.g., `JobId`, `LaneId`) instead of raw bytes
   - Fail fast with clear error messages

4. **Cryptographic Correctness**:
   - Use only approved post-quantum algorithms (Kyber, Dilithium, Blake3)
   - All crypto operations must be constant-time
   - Never reuse nonces or keys

### Error Handling

- Use `Result<T, GixError>` for all fallible operations
- Provide context in error messages
- Never panic in library code (services may panic on unrecoverable errors)

### Testing

- Unit tests for all public APIs
- Integration tests for service workflows
- Property-based tests for cryptographic operations (via `gix-testing`)
- Mock implementations for external dependencies

## Project Structure

### Crates (`crates/`)

Shared libraries that are **not** runnable binaries:
- `gix-crypto`: Cryptographic primitives
- `gix-common`: Shared types and errors
- `gix-gxf`: GXF schema and serialization
- `gix-testing`: Test utilities and mocks

### Services (`services/`)

Runnable daemon binaries:
- `ajr-router`: Mixnet routing service
- `gsee-runtime`: Secure enclave execution runtime
- `gcam-node`: Auction clearing engine

### Tools (`tools/`)

Development and operational tools:
- `gix-cli`: Unified command-line interface
- `gix-sim`: Localnet simulator
- `circuits`: Zero-knowledge proof systems

### SDK (`sdk/`)

Client libraries for external developers:
- `rust`: Native Rust SDK
- `python`: PyO3 bindings
- `js`: WASM bindings

## Specification-Driven Development

All implementation must be driven by specifications in `specs/`:
- `crypto_spec.md`: Cryptographic requirements
- `gxf_spec.md`: GXF format specification
- Component-specific specs in `specs/ajr/`, `specs/gsee/`, `specs/gcam/`

**Rule**: If it's not in the spec, don't implement it yet.

## Git Workflow

- `main`: Production-ready code
- `develop`: Integration branch
- Feature branches: `feature/description`
- All PRs require:
  - Passing CI
  - Code review
  - Updated documentation if needed

## Documentation

- Every public API must have doc comments
- Complex algorithms must have inline comments
- Architecture decisions documented in `specs/`
- README.md in each major directory

## Performance

- Profile before optimizing
- Use `#[inline]` judiciously
- Prefer zero-copy operations where possible
- Cache expensive computations

## Dependencies

- Minimize external dependencies
- Prefer well-maintained, audited crates
- Pin versions for security-critical dependencies
- Document why each dependency is needed




