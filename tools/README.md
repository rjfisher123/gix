# GIX Tools

Development and operational tools for the GIX system.

## Tools

- **gix-cli**: Unified command-line interface for interacting with all GIX components
- **gix-sim**: Localnet simulator that runs the complete GIX workflow (jobs → routes → auction → enclave)
- **circuits**: Zero-knowledge proof systems (Halo2, Plonky2, Nova) for privacy-preserving verification

## Usage

### CLI
```bash
cargo run --bin gix -- --help
```

### Simulator
```bash
cargo run --bin gix-sim
```

### Circuits
The circuits crate provides ZK proof systems for verifying job execution and routing correctness.



