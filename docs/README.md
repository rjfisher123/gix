# GIX Documentation

This directory contains comprehensive documentation for the GIX system.

## Available Documentation

### Core Documentation

- **[gRPC Services Guide](grpc_services_guide.md)** - Complete guide to the GIX gRPC architecture
  - Service architecture and implementation
  - Running the system
  - Development workflow
  - Testing strategies
  - Troubleshooting

### Specifications

See `../specs/` directory:
- `gxf_spec.md` - GXF v3 schema specification
- `crypto_spec.md` - Cryptographic primitives
- `integrated/network_protocol_v0.2.0.md` - Network protocol specification
- `ajr/README.md` - Anonymized Job Routing
- `gcam/README.md` - Global Compute Auction Mechanism
- `gsee/README.md` - Secure Execution Envelope

### Quick Start

1. **Prerequisites:**
   ```bash
   # Install Rust toolchain
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install protoc
   brew install protobuf  # macOS
   # or
   sudo apt-get install protobuf-compiler  # Linux
   ```

2. **Build:**
   ```bash
   cd /Users/ryanfisher/gix
   cargo build --workspace
   ```

3. **Run Services (3 terminals):**
   ```bash
   # Terminal 1
   cargo run --bin ajr-router
   
   # Terminal 2
   cargo run --bin gcam-node
   
   # Terminal 3
   cargo run --bin gsee-runtime
   ```

4. **Run Simulator (4th terminal):**
   ```bash
   cargo run --bin gix-sim
   ```

## Documentation Structure

```
docs/
├── README.md                    # This file
├── grpc_services_guide.md       # gRPC architecture guide
└── (future docs)

specs/
├── README.md                    # Specifications overview
├── gxf_spec.md                  # GXF format
├── crypto_spec.md               # Cryptography
├── integrated/
│   ├── network_protocol_v0.2.0.md
│   └── README.md
├── ajr/README.md                # AJR specification
├── gcam/README.md               # GCAM specification
└── gsee/README.md               # GSEE specification

proto/
└── README.md                    # Protobuf definitions

crates/
└── (each crate has inline doc comments)

services/
└── (each service has inline doc comments)
```

## Contributing

When adding new features:

1. Update relevant specifications in `specs/`
2. Update `docs/grpc_services_guide.md` if changing architecture
3. Add inline documentation to code (doc comments)
4. Update `proto/README.md` if changing protocol
5. Update this README if adding new documentation

## Documentation Standards

- Use Markdown for all documentation
- Include code examples where relevant
- Keep specifications separate from implementation guides
- Version all protocol/API documentation
- Include diagrams for complex architectures (ASCII art is fine)

## Getting Help

- Check the troubleshooting section in `grpc_services_guide.md`
- Review inline code documentation
- Check specification files in `specs/`
- Examine example code in `tools/gix-sim`

