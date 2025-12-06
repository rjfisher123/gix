# GIX gRPC Services - Implementation Guide

**Version:** v0.2.0  
**Status:** Complete  
**Date:** December 2025

## Overview

The GIX system has been successfully converted to a distributed gRPC architecture. The three core services now run as standalone gRPC servers, and the simulator acts as a gRPC client.

## Architecture

```
┌─────────────────┐
│   gix-sim       │  (gRPC Client)
│   (Simulator)   │
└────────┬────────┘
         │
         ├──────────────┬──────────────┬──────────────┐
         │              │              │              │
         v              v              v              v
    ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
    │ Router  │    │ Auction │    │Execution│    │  Proto  │
    │ Service │    │ Service │    │ Service │    │  Defs   │
    │  (AJR)  │    │ (GCAM)  │    │ (GSEE)  │    │(gix.v1) │
    │  :50051 │    │  :50052 │    │  :50053 │    └─────────┘
    └─────────┘    └─────────┘    └─────────┘
```

## Components

### 1. Protocol Definitions (`crates/gix-proto`)

**Purpose:** Define gRPC service interfaces and message types

**Key Files:**
- `proto/gix.proto` - Protobuf definitions
- `crates/gix-proto/build.rs` - Code generation at build time
- `crates/gix-proto/src/lib.rs` - Re-exports for convenience

**Services Defined:**
- `RouterService` - Job routing through anonymized lanes
- `AuctionService` - Job matching and pricing
- `ExecutionService` - Secure job execution

**Common Types:**
- `JobId`, `LaneId`, `SlpId` - Strongly-typed identifiers
- `PrecisionLevel` - Compute precision enums
- `ExecutionStatus` - Job execution status

### 2. Router Service (`services/ajr-router`)

**Purpose:** Anonymized Job Routing (AJR)

**Port:** 50051

**RPCs:**
- `RouteEnvelope` - Route a job through anonymized lanes
- `GetRouterStats` - Retrieve routing statistics

**Implementation Details:**
- **Library (`src/lib.rs`):**
  - `RouterState` - Maintains lane information and statistics
  - `process_envelope()` - Core routing logic
  - Lane selection based on priority (≥128 → Flash, <128 → Deep)
  - Capacity management and fallback logic

- **Binary (`src/main.rs`):**
  - Implements `RouterService` trait
  - gRPC server on port 50051
  - Converts between proto messages and internal types

**Key Features:**
- 2 routing lanes: Flash (high-priority), Deep (normal)
- Automatic fallback when lanes reach capacity
- Real-time statistics tracking

### 3. Auction Service (`services/gcam-node`)

**Purpose:** Global Compute Auction Mechanism (GCAM)

**Port:** 50052

**RPCs:**
- `RunAuction` - Match jobs with providers and determine pricing
- `GetAuctionStats` - Retrieve auction statistics

**Implementation Details:**
- **Library (`src/lib.rs`):**
  - `AuctionEngine` - Manages providers and routes
  - `ComputeProvider` - Represents compute providers with capabilities
  - `Route` - Network routes with latency/cost metrics
  - Dynamic pricing based on precision level
  - `process_envelope()` - Wrapper for envelope-based auction

- **Binary (`src/main.rs`):**
  - Implements `AuctionService` trait
  - gRPC server on port 50052
  - Job matching and route selection

**Key Features:**
- Provider matching based on precision support
- Dynamic pricing (BF16: 100, FP8: 80, E5M2: 60, INT8: 50)
- Route optimization (latency + cost)
- Statistics by precision and lane

### 4. Execution Service (`services/gsee-runtime`)

**Purpose:** Secure Execution Envelope (GSEE)

**Port:** 50053

**RPCs:**
- `ExecuteJob` - Execute a job in secure enclave
- `GetRuntimeStats` - Retrieve execution statistics

**Implementation Details:**
- **Library (`src/lib.rs`):**
  - `RuntimeState` - Manages execution environment
  - Compliance checks:
    - **Precision:** Validates supported precision levels
    - **Shape:** Validates sequence length and batch size
    - **Residency:** Validates data residency requirements
  - Simulated execution with realistic timing
  - `process_envelope()` - Full envelope validation and execution

- **Binary (`src/main.rs`):**
  - Implements `ExecutionService` trait
  - gRPC server on port 50053
  - Compliance enforcement

**Key Features:**
- Strict compliance checking
- Support for BF16, FP8, E5M2, INT8
- Max sequence length: 8192
- Max batch size: 32
- Region restrictions: US, EU

### 5. Simulator (`tools/gix-sim`)

**Purpose:** End-to-end testing and demonstration

**Implementation Details:**
- **Library (`src/lib.rs`):**
  - `Simulation` struct with gRPC clients
  - Random job generation
  - Full workflow orchestration
  - Statistics aggregation

- **Binary (`src/main.rs`):**
  - CLI entry point
  - Runs 5 simulation ticks
  - Displays real-time statistics

**Workflow per Tick:**
1. Generate random test job
2. Create GXF envelope
3. Route through AJR (gRPC call)
4. Run GCAM auction (gRPC call)
5. Execute in GSEE (gRPC call)
6. Track statistics

## Running the System

### Prerequisites

```bash
# Install protoc (required for building gix-proto)
# macOS:
brew install protobuf

# Linux:
sudo apt-get install protobuf-compiler

# Windows:
# Download from https://github.com/protocolbuffers/protobuf/releases
```

### Build Everything

```bash
cd /Users/ryanfisher/gix

# Build all crates and services
cargo build --workspace

# Or build specific components
cargo build -p gix-proto
cargo build -p ajr-router
cargo build -p gcam-node
cargo build -p gsee-runtime
cargo build -p gix-sim
```

### Start Services (3 terminals)

**Terminal 1 - Router Service:**
```bash
cargo run --bin ajr-router
# Output: AJR Router Service starting on 127.0.0.1:50051
```

**Terminal 2 - Auction Service:**
```bash
cargo run --bin gcam-node
# Output: GCAM Node Service starting on 127.0.0.1:50052
```

**Terminal 3 - Execution Service:**
```bash
cargo run --bin gsee-runtime
# Output: GSEE Runtime Service starting on 127.0.0.1:50053
```

### Run Simulator (4th terminal)

```bash
cargo run --bin gix-sim

# Expected output:
# GIX Simulator Starting
# Connecting to services...
#   - AJR Router:      http://127.0.0.1:50051
#   - GCAM Node:       http://127.0.0.1:50052
#   - GSEE Runtime:    http://127.0.0.1:50053
# 
# Connected! Running 5 simulation ticks...
# 
# [Tick 1] Tick 1: Processed 1 jobs | Router: 1 routed | Auction: 1 matches (volume: 80) | Runtime: 1 executed (1 completed, 0 rejected)
# [Tick 2] Tick 2: Processed 2 jobs | Router: 2 routed | Auction: 2 matches (volume: 180) | Runtime: 2 executed (2 completed, 0 rejected)
# ...
```

## Message Flow

### Example: Job Execution Flow

1. **Simulator → Router (RouteEnvelope)**
   ```
   Request: { envelope: <serialized GXF envelope> }
   Response: { lane_id: { id: 0 }, success: true, error: "" }
   ```

2. **Simulator → Auction (RunAuction)**
   ```
   Request: { job: <serialized GXF job>, priority: 150 }
   Response: { 
     job_id: { id: <bytes> },
     slp_id: { id: "slp-001" },
     lane_id: { id: 0 },
     price: 100,
     route: ["US-WEST", "US-EAST"],
     success: true,
     error: ""
   }
   ```

3. **Simulator → Execution (ExecuteJob)**
   ```
   Request: { envelope: <serialized GXF envelope> }
   Response: {
     job_id: { id: <bytes> },
     status: EXECUTION_STATUS_COMPLETED,
     duration_ms: 523,
     output_hash: <32 bytes>,
     success: true,
     error: ""
   }
   ```

## Code Structure

### Each Service Has Two Parts:

1. **Library (`src/lib.rs`)**
   - Business logic
   - State management
   - Reusable across binaries
   - No gRPC dependencies in core logic

2. **Binary (`src/main.rs`)**
   - gRPC server implementation
   - Protocol conversion (proto types ↔ internal types)
   - Server initialization and lifecycle

### Benefits of This Structure:

- ✅ Clean separation of concerns
- ✅ Testable business logic (no network dependencies)
- ✅ Easy to add new transports (HTTP REST, WebSocket, etc.)
- ✅ Library can be used in other contexts (CLI, embedded)

## Testing

### Unit Tests

```bash
# Test individual crates
cargo test -p gix-common
cargo test -p gix-crypto
cargo test -p gix-gxf
cargo test -p ajr-router
cargo test -p gcam-node
cargo test -p gsee-runtime
```

### Integration Tests

```bash
# Full end-to-end test
# 1. Start all three services (see "Start Services" above)
# 2. Run simulator
cargo run --bin gix-sim
```

### Manual Testing with grpcurl

```bash
# Install grpcurl
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest

# List services
grpcurl -plaintext localhost:50051 list

# Call router stats
grpcurl -plaintext localhost:50051 gix.v1.RouterService/GetRouterStats

# Call auction stats
grpcurl -plaintext localhost:50052 gix.v1.AuctionService/GetAuctionStats

# Call runtime stats
grpcurl -plaintext localhost:50053 gix.v1.ExecutionService/GetRuntimeStats
```

## Troubleshooting

### Issue: "protoc not found"

**Solution:**
```bash
# macOS
brew install protobuf

# Linux (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install protobuf-compiler

# Verify installation
protoc --version
```

### Issue: Services won't start (port already in use)

**Solution:**
```bash
# Check what's using the ports
lsof -i :50051
lsof -i :50052
lsof -i :50053

# Kill processes if needed
kill -9 <PID>
```

### Issue: Simulator can't connect

**Cause:** Services not running

**Solution:**
1. Verify all three services are running
2. Check logs for any errors
3. Ensure ports 50051, 50052, 50053 are accessible

### Issue: Compilation errors in gix-proto

**Cause:** Usually related to protoc or version mismatches

**Solution:**
```bash
# Clean and rebuild
cargo clean
cargo build -p gix-proto

# Check proto file syntax
protoc --proto_path=proto --decode_raw < proto/gix.proto
```

## Development Workflow

### Adding a New RPC

1. **Update `proto/gix.proto`:**
   ```protobuf
   service RouterService {
     rpc NewMethod(NewRequest) returns (NewResponse);
   }
   
   message NewRequest { ... }
   message NewResponse { ... }
   ```

2. **Rebuild gix-proto:**
   ```bash
   cargo build -p gix-proto
   ```

3. **Implement in service (`src/main.rs`):**
   ```rust
   async fn new_method(
       &self,
       request: Request<NewRequest>,
   ) -> Result<Response<NewResponse>, Status> {
       // Implementation
   }
   ```

4. **Add business logic in library (`src/lib.rs`)**

5. **Test end-to-end**

### Adding a New Service

1. Create new proto service definition
2. Implement new binary in `services/`
3. Add library with business logic
4. Update simulator to use new service
5. Document in this guide

## Performance Considerations

### Current Performance (Simulation Mode)

- **Router:** ~1ms per request (in-memory)
- **Auction:** ~2ms per auction (mock matching)
- **Execution:** 10-500ms (simulated based on sequence length)
- **Network:** Local loopback (sub-millisecond)

### Production Considerations

- Add connection pooling for clients
- Implement request timeouts
- Add circuit breakers for fault tolerance
- Enable TLS for encrypted transport
- Add authentication/authorization
- Implement rate limiting
- Add distributed tracing (OpenTelemetry)
- Use load balancers for horizontal scaling

## Next Steps

### Planned Improvements

1. **Security:**
   - [ ] Implement TLS/mTLS
   - [ ] Add API key authentication
   - [ ] Implement request signing

2. **Reliability:**
   - [ ] Add health checks
   - [ ] Implement graceful shutdown
   - [ ] Add retry logic with exponential backoff
   - [ ] Circuit breaker pattern

3. **Observability:**
   - [ ] Structured logging with trace IDs
   - [ ] Metrics (Prometheus)
   - [ ] Distributed tracing (Jaeger/Zipkin)
   - [ ] Request/response logging

4. **Performance:**
   - [ ] Connection pooling
   - [ ] Request pipelining
   - [ ] Compression (gzip)
   - [ ] Streaming for large payloads

5. **Deployment:**
   - [ ] Docker containers
   - [ ] Kubernetes manifests
   - [ ] Helm charts
   - [ ] CI/CD pipelines

## References

- [gRPC Documentation](https://grpc.io/docs/)
- [Tonic (Rust gRPC)](https://github.com/hyperium/tonic)
- [Protocol Buffers](https://protobuf.dev/)
- [GIX Network Protocol Spec](../specs/integrated/network_protocol_v0.2.0.md)


