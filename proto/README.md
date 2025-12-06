# GIX Protocol Buffer Definitions

This directory contains the Protocol Buffer (protobuf) definitions for the GIX gRPC services.

## Files

- `gix.proto` - Main protocol buffer definition file (v0.2.0) containing:
  - **Common Types** - JobId, LaneId, SlpId, PrecisionLevel, ExecutionStatus
  - **RouterService (AJR)** - Anonymized Job Routing
  - **AuctionService (GCAM)** - Global Compute Auction Mechanism
  - **ExecutionService (GSEE)** - Secure Execution Envelope

## Code Generation

The proto files are compiled to Rust code by the `gix-proto` crate using `tonic-build`.

To regenerate the code:

```bash
cargo build -p gix-proto
```

**Prerequisite:** You need the `protoc` (Protocol Buffer Compiler) installed and in your PATH.
- macOS: `brew install protobuf`
- Linux: `sudo apt-get install protobuf-compiler` (or equivalent)
- Windows: Download from [protobuf releases](https://github.com/protocolbuffers/protobuf/releases)

The generated code will be available in the `gix-proto` crate and can be imported by other crates.

## Usage

Import the generated types and services:

```rust
use gix_proto::v1::*; // Message types and enums
use gix_proto::RouterServiceClient; // Re-exported client
use gix_proto::{RouterService, RouterServiceServer}; // Re-exported server
```

## Protocol Version

Current version: **v0.2.0** (`gix.v1` package)

## Service Ports

- **RouterService:** 50051
- **AuctionService:** 50052
- **ExecutionService:** 50053

## Changes from Previous Version

### v0.2.0 (Current)
- Renamed `RuntimeService` → `ExecutionService` for clarity
- Renamed `GetStats` RPCs to be service-specific:
  - `GetRouterStats` (RouterService)
  - `GetAuctionStats` (AuctionService)
  - `GetRuntimeStats` (ExecutionService)
- All message types now use proper nested message types (JobId, LaneId, SlpId)
- Added structured common types for better type safety

When making breaking changes to the protocol, increment the version number in this README and update the spec file in `specs/integrated/network_protocol_v*.md`.
