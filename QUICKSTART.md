# GIX Monorepo - Quick Reference

**Version:** v0.2.0  
**Status:** Development  
**Architecture:** Distributed gRPC Services

## Repository Structure

```
gix/
├── crates/              # Shared libraries
│   ├── gix-common/      # Common types (JobId, LaneId, etc.)
│   ├── gix-crypto/      # Crypto primitives (Blake3, VDF, Kyber)
│   ├── gix-gxf/         # GXF v3 schema and validation
│   ├── gix-proto/       # gRPC protocol definitions
│   └── gix-testing/     # Test utilities
│
├── services/            # Standalone gRPC servers
│   ├── ajr-router/      # Anonymized Job Routing (:50051)
│   ├── gcam-node/       # Global Compute Auction (:50052)
│   └── gsee-runtime/    # Secure Execution Envelope (:50053)
│
├── tools/               # Development tools
│   ├── gix-sim/         # End-to-end simulator (gRPC client)
│   ├── gix-cli/         # Command-line interface
│   └── circuits/        # Circuit implementations
│
├── proto/               # Protocol Buffer definitions
│   └── gix.proto        # gRPC service definitions (gix.v1)
│
├── docs/                # Documentation
│   ├── README.md        # Docs index
│   └── grpc_services_guide.md  # gRPC implementation guide
│
├── specs/               # Specifications
│   ├── gxf_spec.md
│   ├── crypto_spec.md
│   └── integrated/
│       └── network_protocol_v0.2.0.md
│
├── sdk/                 # Client SDKs
│   ├── rust/
│   ├── python/
│   └── js/
│
├── infra/               # Infrastructure as code
│   ├── docker/
│   ├── k8s/
│   └── terraform/
│
└── Cargo.toml           # Workspace configuration
```

## Quick Start

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Protocol Buffer Compiler (required!)
brew install protobuf              # macOS
sudo apt install protobuf-compiler # Linux
```

### Build

```bash
cd /Users/ryanfisher/gix
cargo build --workspace
```

### Run Services (3 terminals)

```bash
# Terminal 1 - Router
cargo run --bin ajr-router
# → Listening on 127.0.0.1:50051

# Terminal 2 - Auction
cargo run --bin gcam-node
# → Listening on 127.0.0.1:50052

# Terminal 3 - Runtime
cargo run --bin gsee-runtime
# → Listening on 127.0.0.1:50053
```

### Run Simulator (4th terminal)

```bash
cargo run --bin gix-sim
# → Connects to all services
# → Runs 5 simulation ticks
# → Shows real-time statistics
```

## Core Components

### 1. Services (Standalone gRPC Servers)

| Service | Port | Purpose | RPCs |
|---------|------|---------|------|
| **AJR Router** | 50051 | Anonymized job routing | `RouteEnvelope`, `GetRouterStats` |
| **GCAM Node** | 50052 | Job matching & pricing | `RunAuction`, `GetAuctionStats` |
| **GSEE Runtime** | 50053 | Secure job execution | `ExecuteJob`, `GetRuntimeStats` |

### 2. Shared Libraries

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| **gix-common** | Common types | `JobId`, `LaneId`, `SlpId`, `GixError` |
| **gix-crypto** | Cryptography | `hash_blake3()`, `vdf_*()`, `kyber_*()` |
| **gix-gxf** | GXF schema | `GxfEnvelope`, `GxfJob`, `PrecisionLevel` |
| **gix-proto** | gRPC definitions | Generated proto types, client/server traits |

### 3. Protocol (gix.proto)

```protobuf
package gix.v1;

// Common Types
message JobId { bytes id = 1; }
message LaneId { uint32 id = 1; }
message SlpId { string id = 1; }
enum PrecisionLevel { BF16, FP8, E5M2, INT8 }
enum ExecutionStatus { COMPLETED, FAILED, REJECTED }

// Services
service RouterService { ... }
service AuctionService { ... }
service ExecutionService { ... }
```

## Development Workflow

### Adding a New Feature

1. **Update Protocol:**
   ```bash
   # Edit proto/gix.proto
   nano proto/gix.proto
   
   # Rebuild
   cargo build -p gix-proto
   ```

2. **Implement in Service:**
   ```rust
   // services/xxx/src/lib.rs - business logic
   pub async fn new_function() { ... }
   
   // services/xxx/src/main.rs - gRPC handler
   async fn new_rpc(&self, request: Request<...>) -> Result<...> {
       // Convert proto → internal types
       // Call business logic
       // Convert result → proto
   }
   ```

3. **Test End-to-End:**
   ```bash
   # Start services
   cargo run --bin ajr-router &
   cargo run --bin gcam-node &
   cargo run --bin gsee-runtime &
   
   # Update simulator to use new feature
   # Run simulator
   cargo run --bin gix-sim
   ```

### Running Tests

```bash
# Unit tests (no network)
cargo test -p gix-common
cargo test -p gix-crypto
cargo test -p gix-gxf
cargo test -p ajr-router --lib
cargo test -p gcam-node --lib
cargo test -p gsee-runtime --lib

# Integration tests (requires running services)
cargo run --bin gix-sim
```

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin ajr-router

# Use grpcurl to test services
grpcurl -plaintext localhost:50051 list
grpcurl -plaintext localhost:50051 gix.v1.RouterService/GetRouterStats
```

## Architecture Overview

### Request Flow

```
Client/Simulator
    │
    ├─1─→ RouteEnvelope ────→ AJR Router (:50051)
    │                          └─→ Select lane (Flash/Deep)
    │                          └─→ Return lane assignment
    │
    ├─2─→ RunAuction ───────→ GCAM Node (:50052)
    │                          └─→ Match with providers
    │                          └─→ Calculate pricing
    │                          └─→ Select route
    │                          └─→ Return match details
    │
    └─3─→ ExecuteJob ───────→ GSEE Runtime (:50053)
                               └─→ Check compliance
                               └─→ Execute job
                               └─→ Return results
```

### Data Flow

```
GxfJob (JSON)
    ↓
GxfEnvelope (JSON + metadata)
    ↓
Protobuf (bytes + status/error)
    ↓
gRPC (HTTP/2)
    ↓
Service Handler
    ↓
Business Logic (Rust)
    ↓
Result
    ↓
Protobuf Response
    ↓
gRPC Response
```

## Key Design Principles

1. **Library + Binary Separation**
   - `lib.rs` = pure business logic
   - `main.rs` = gRPC server + protocol conversion
   - Benefits: testable, reusable, transport-agnostic

2. **Type Safety**
   - Proper proto message types (not raw primitives)
   - Rust's type system for compile-time safety
   - No `stringly-typed` APIs

3. **Service Isolation**
   - Each service runs independently
   - No shared state
   - Communicate only via gRPC

4. **Clear Boundaries**
   - Protocol layer (proto) ↔ Business logic (lib) ↔ Network (main)
   - Easy to swap transports (gRPC → HTTP REST)
   - Easy to test (mock at boundaries)

## Common Tasks

### Check Service Health

```bash
# Check if services are running
lsof -i :50051  # Router
lsof -i :50052  # Auction
lsof -i :50053  # Runtime

# Test connectivity
grpcurl -plaintext localhost:50051 gix.v1.RouterService/GetRouterStats
grpcurl -plaintext localhost:50052 gix.v1.AuctionService/GetAuctionStats
grpcurl -plaintext localhost:50053 gix.v1.ExecutionService/GetRuntimeStats
```

### Clean Build

```bash
# Full clean
cargo clean

# Rebuild proto (if proto file changed)
cargo build -p gix-proto

# Rebuild everything
cargo build --workspace
```

### Update Dependencies

```bash
# Check for updates
cargo outdated

# Update Cargo.lock
cargo update

# Update specific crate
cargo update -p tonic
```

## Troubleshooting

### "protoc not found"

```bash
# Install protobuf compiler
brew install protobuf              # macOS
sudo apt install protobuf-compiler # Linux

# Verify
protoc --version
```

### "Port already in use"

```bash
# Find process using port
lsof -i :50051

# Kill process
kill -9 <PID>

# Or kill all services
pkill -f ajr-router
pkill -f gcam-node
pkill -f gsee-runtime
```

### "Connection refused"

- Ensure all three services are running
- Check logs for startup errors
- Verify ports are correct (50051, 50052, 50053)

### Compilation Errors

```bash
# Clean and rebuild
cargo clean
cargo build -p gix-proto
cargo build --workspace

# Check for version mismatches
grep -r "tonic.*=" Cargo.toml crates/*/Cargo.toml services/*/Cargo.toml
grep -r "prost.*=" Cargo.toml crates/*/Cargo.toml services/*/Cargo.toml
```

## Documentation

- **Implementation Guide:** `docs/grpc_services_guide.md`
- **Protocol Spec:** `specs/integrated/network_protocol_v0.2.0.md`
- **GXF Spec:** `specs/gxf_spec.md`
- **Crypto Spec:** `specs/crypto_spec.md`
- **Implementation Status:** `IMPLEMENTATION_STATUS.md`

## Performance Notes

### Current (Localnet Simulation)

- Router: ~1ms per request
- Auction: ~2ms per auction
- Execution: 10-500ms (simulated)
- Network: <1ms (loopback)

### Production Targets

- TLS overhead: +5-10ms
- Network latency: 10-100ms (regional)
- Provider matching: <50ms
- Compliance checks: <10ms
- Execution: 100ms-10s (depending on job)

## Next Steps

### Phase 2 Remaining

- [ ] Add TLS/mTLS support
- [ ] Implement authentication
- [ ] Add health checks
- [ ] Implement graceful shutdown
- [ ] Add metrics (Prometheus)
- [ ] Add distributed tracing

### Phase 3 (Future)

- [ ] Deploy to cloud
- [ ] Add more providers
- [ ] Implement real execution
- [ ] Add billing system
- [ ] Build web UI
- [ ] Create client SDKs

---

**Questions?** See `docs/grpc_services_guide.md` for detailed information.

**Found a bug?** Check `IMPLEMENTATION_STATUS.md` for known issues.

**Last Updated:** December 6, 2025

