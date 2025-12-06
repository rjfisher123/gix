# GIX Network Protocol v0.2.0

**Version:** 0.2.0  
**Status:** Draft  
**Date:** December 2025

## Overview

This document defines the network protocol for the Global Intelligence Exchange (GIX) system. It specifies the gRPC service definitions, message types, and communication patterns for the three core services:

- **RouterService (AJR)** - Anonymized Job Routing
- **AuctionService (GCAM)** - Global Compute Auction Mechanism
- **ExecutionService (GSEE)** - Secure Execution Envelope

## Service Definitions

### Common Types

```protobuf
// Job identifier (16 bytes)
message JobId {
    bytes id = 1;
}

// Lane identifier (0-255)
message LaneId {
    uint32 id = 1;
}

// SLP identifier
message SlpId {
    string id = 1;
}

// Precision levels
enum PrecisionLevel {
    PRECISION_LEVEL_UNSPECIFIED = 0;
    PRECISION_LEVEL_BF16 = 1;
    PRECISION_LEVEL_FP8 = 2;
    PRECISION_LEVEL_E5M2 = 3;
    PRECISION_LEVEL_INT8 = 4;
}

// Execution status
enum ExecutionStatus {
    EXECUTION_STATUS_UNSPECIFIED = 0;
    EXECUTION_STATUS_COMPLETED = 1;
    EXECUTION_STATUS_FAILED = 2;
    EXECUTION_STATUS_REJECTED = 3;
}
```

### Router Service (AJR)

Handles anonymized job routing through mixnet lanes.

```protobuf
service RouterService {
    // Route an envelope through the anonymized job routing system
    rpc RouteEnvelope(RouteEnvelopeRequest) returns (RouteEnvelopeResponse);
    
    // Get router statistics
    rpc GetRouterStats(GetRouterStatsRequest) returns (GetRouterStatsResponse);
}

message RouteEnvelopeRequest {
    bytes envelope = 1; // Serialized GXF envelope (JSON)
}

message RouteEnvelopeResponse {
    LaneId lane_id = 1;
    bool success = 2;
    string error = 3;
}

message GetRouterStatsRequest {}

message GetRouterStatsResponse {
    uint64 total_routed = 1;
    map<uint32, uint64> lane_stats = 2; // lane_id -> count
}
```

### Auction Service (GCAM)

Handles job matching, pricing, and route selection.

```protobuf
service AuctionService {
    // Run an auction for a job
    rpc RunAuction(RunAuctionRequest) returns (RunAuctionResponse);
    
    // Get auction statistics
    rpc GetAuctionStats(GetAuctionStatsRequest) returns (GetAuctionStatsResponse);
}

message RunAuctionRequest {
    bytes job = 1; // Serialized GXF job (JSON)
    uint32 priority = 2;
}

message RunAuctionResponse {
    JobId job_id = 1;
    SlpId slp_id = 2;
    LaneId lane_id = 3;
    uint64 price = 4;
    repeated string route = 5;
    bool success = 6;
    string error = 7;
}

message GetAuctionStatsRequest {}

message GetAuctionStatsResponse {
    uint64 total_auctions = 1;
    uint64 total_matches = 2;
    uint64 total_volume = 3;
    map<string, uint64> matches_by_precision = 4;
    map<uint32, uint64> matches_by_lane = 5;
}
```

### Execution Service (GSEE)

Handles secure job execution within enclaves.

```protobuf
service ExecutionService {
    // Execute a job in the secure execution envelope
    rpc ExecuteJob(ExecuteJobRequest) returns (ExecuteJobResponse);
    
    // Get runtime statistics
    rpc GetRuntimeStats(GetRuntimeStatsRequest) returns (GetRuntimeStatsResponse);
}

message ExecuteJobRequest {
    bytes envelope = 1; // Serialized GXF envelope (JSON)
}

message ExecuteJobResponse {
    JobId job_id = 1;
    ExecutionStatus status = 2;
    uint64 duration_ms = 3;
    bytes output_hash = 4;
    bool success = 5;
    string error = 6;
}

message GetRuntimeStatsRequest {}

message GetRuntimeStatsResponse {
    uint64 total_executed = 1;
    uint64 total_completed = 2;
    uint64 total_failed = 3;
    uint64 total_rejected = 4;
    map<string, uint64> jobs_by_precision = 5;
}
```

## Communication Flow

### Job Execution Workflow

1. **Client → RouterService:**
   - Client submits GXF envelope via `RouteEnvelope`
   - Router selects lane based on priority
   - Returns lane assignment

2. **Client → AuctionService:**
   - Client submits job for auction via `RunAuction`
   - Auction engine matches with providers
   - Returns match details (SLP, route, price)

3. **Client → ExecutionService:**
   - Client submits envelope for execution via `ExecuteJob`
   - Runtime performs compliance checks
   - Executes job and returns results

## Transport Layer

- **Protocol:** gRPC over HTTP/2
- **Serialization:** Protocol Buffers (v3)
- **Encoding:** Binary (protobuf wire format)
- **Payload Encoding:** GXF envelopes and jobs are serialized as JSON and transmitted as bytes

## Service Ports

- **RouterService:** 50051
- **AuctionService:** 50052
- **ExecutionService:** 50053

## Error Handling

All services follow a consistent error pattern:
- `success` field indicates operation success
- `error` field contains human-readable error message when `success = false`
- gRPC status codes used for transport-level errors

## Security Considerations

- **TLS:** All services should use TLS in production (currently plaintext for localnet)
- **Authentication:** Future versions will include mutual TLS and API keys
- **Envelope Encryption:** GXF envelopes may contain encrypted payloads (handled at application layer)

## Versioning

- **Current Version:** v0.2.0
- **Package:** gix.v1
- **Breaking Changes:** Any changes to message structure or RPC signatures require version increment


