# Changelog

All notable changes to the GIX project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-12-06

### Codename: Distributed Network Release

**Status:** Beta - Production Infrastructure Ready

This release transforms GIX from a monolithic simulation into a distributed microservices architecture with production-grade cryptography, persistent storage, and comprehensive observability.

### Added

#### Network Layer (`gix-proto`)
- **gRPC Microservices**: Full Protocol Buffer definitions for all services
- **RouterService (AJR)**: Port 50051 - Envelope routing and lane statistics
- **AuctionService (GCAM)**: Port 50052 - Job matching and auction stats
- **ExecutionService (GSEE)**: Port 50053 - Job execution and runtime stats
- **Type-safe Protocol**: Compile-time verification of service contracts
- **HTTP/2 Transport**: Multiplexed connections with TLS-ready infrastructure

#### Cryptography Layer (`gix-crypto`)
- **Real PQC**: Production Kyber1024 (NIST Level 5) and Dilithium3 (NIST Level 3)
- **Wesolowski VDF**: Real verifiable delay function with 2048-bit RSA modulus
- **Blake3 Integration**: High-performance cryptographic hashing
- **Comprehensive Testing**: 16+ test scenarios covering all cryptographic operations
- **Security**: Post-quantum resistant to both classical and quantum adversaries

#### Persistence Layer (`gcam-node`)
- **Sled Database**: Embedded key-value store with ACID transactions
- **Crash Recovery**: Automatic state restoration from write-ahead log
- **Persistent Volumes**: Market ledger survives process restarts
- **Graceful Shutdown**: CTRL+C handler flushes database before exit
- **Multi-tree Storage**: Separate trees for providers, routes, and statistics

#### CLI Tool (`gix-cli`)
- **Wallet Management**: Secure Dilithium3 keypair generation and storage
- **Job Submission**: YAML-based job specification and submission
- **Status Queries**: Real-time auction statistics via gRPC
- **Security**: File permissions (600), non-root user, clear warnings
- **User Experience**: Colored output, helpful errors, sensible defaults

#### Infrastructure & Operations
- **Docker Containerization**: Multi-stage builds with minimal runtime images (~150MB)
- **Docker Compose**: Complete stack deployment with health checks
- **Prometheus Metrics**: Real-time metrics collection from all services
- **Grafana Dashboards**: Web-based visualization and monitoring
- **Deployment Scripts**: Automated localnet deployment with connectivity testing
- **Security**: Non-root containers, isolated networks, read-only potential
- **Cloud Deployment (Phase 6)**: Production-ready Terraform IaC for AWS
  - Automated EC2 provisioning with Ubuntu 22.04 LTS
  - Security groups with proper firewall rules
  - Elastic IP for stable addressing
  - User data script for zero-touch bootstrap
  - Multi-node scaling support via Terraform workspaces
  - Cost-effective deployment (~$35-50/month per node)

#### Observability
- **Router Metrics**: Packets routed, active jobs per lane
- **Auction Metrics**: Clearing prices, match rates, provider utilization
- **System Metrics**: CPU, memory, network I/O via Prometheus
- **Alerting Ready**: Prometheus alert rules for SLA monitoring
- **15s Scrape Interval**: Near real-time metrics collection

### Changed

#### Architecture
- **Monolith → Microservices**: Services communicate via gRPC instead of function calls
- **In-memory → Persistent**: GCAM state now survives restarts
- **Mock → Production**: Real cryptographic implementations throughout
- **Local → Distributed**: Services can run on separate nodes

#### Performance
- **Release Builds**: LTO enabled for optimized binaries
- **Async Runtime**: Tokio for efficient concurrent operations
- **Binary Serialization**: Bincode for efficient database storage
- **HTTP/2 Multiplexing**: Multiple RPCs over single connection

#### Developer Experience
- **CLI Tool**: End users can submit jobs without writing code
- **Deployment Script**: One-command stack deployment
- **Metrics Dashboards**: Visual system health monitoring
- **Comprehensive Docs**: 10+ documentation files covering all aspects

### Improved

#### Security
- **PQC Encryption**: Kyber1024 key encapsulation (vs. mock)
- **PQC Signatures**: Dilithium3 signatures (vs. mock)
- **VDF Timing**: Real sequential computation (vs. hash chain)
- **Container Security**: Non-root users, minimal images, isolated networks

#### Reliability
- **Health Checks**: All services monitored for availability
- **Graceful Shutdown**: Proper cleanup on CTRL+C
- **Database Flushing**: Ensures data persistence
- **Service Dependencies**: Ordered startup prevents partial deployments

#### Observability
- **Structured Logging**: Tracing throughout codebase
- **Metrics Export**: Prometheus-compatible metrics
- **Visualization**: Grafana dashboards for real-time monitoring
- **Debugging**: Service logs accessible via docker-compose

### Fixed
- **Dependency Flow**: Correct reverse-pyramid architecture
- **Port Conflicts**: Services on dedicated ports (50051-50053, 9001-9002)
- **Data Loss**: Persistent storage prevents state loss on restart
- **Type Safety**: gRPC ensures correct message formats

### Deployment

#### Quick Start
```bash
# Deploy complete stack
./scripts/deploy_localnet.sh

# Access services
# - Prometheus: http://localhost:9090
# - Grafana: http://localhost:3000 (admin/admin)
# - Metrics: http://localhost:9001, :9002

# Submit job
cargo run -p gix-cli -- keygen
cargo run -p gix-cli -- submit examples/job_sample.yaml
```

#### System Requirements
- Docker & Docker Compose
- 4GB RAM (2GB for services, 2GB for monitoring)
- 10GB disk space (for Docker images and data)

### Breaking Changes
- **Service Communication**: Direct function calls replaced with gRPC
- **Configuration**: Environment variables for service addresses
- **Storage**: GCAM now requires writable data directory

### Migration Notes
- Version 0.1.0 state is not compatible with 0.2.0 (fresh start required)
- Update all client code to use gRPC clients instead of library calls
- Docker Compose is now the recommended deployment method

### Known Limitations
- **Production Crypto**: VDF proving is CPU-intensive (intentional)
- **Database**: Single-node only (no distributed database)
- **Networking**: No TLS yet (plaintext gRPC in localnet)
- **Monitoring**: Dashboard templates not included (manual setup required)

### Roadmap to v0.3.0
- [ ] **TLS/mTLS**: Encrypted service communication
- [ ] **Distributed Database**: Multi-node GCAM coordination
- [ ] **Advanced Monitoring**: Pre-configured Grafana dashboards
- [ ] **Performance**: Benchmarking and optimization
- [ ] **Documentation**: API documentation via Rustdoc

---

## [0.1.0-alpha] - 2025-12-XX

### Codename: Genesis

**Status:** Internal Developer Preview

This release marks the first end-to-end execution of the Global Intelligence Exchange (GIX) protocol stack. It establishes the "Localnet" simulation environment where the Market, Transport, and Execution layers interoperate using the canonical GXF v3 job format.

### Added

#### Cryptography Layer (`gix-crypto`)
- **Hashing**: Blake3 standardized for Content Addressing
- **PQC**: Mock interfaces established for Kyber-1024 (KEM) and Dilithium-3 (Signatures)
- **VDF**: Mock "Hash-Chain" Verifiable Delay Function implemented for TLP timing

#### Data Layer (`gix-gxf`)
- **Schema**: Full GXF v3 ABI implementation (DAGs, Edges, Metadata)
- **Compliance**: Strict validation logic for:
  - Precision Levels (BF16, FP8, E5M2, INT8)
  - Sequence Length Constraints (KV-Cache limits)
  - Dynamic Shape Levels (L0-L3)

#### Transport Layer (`services/ajr-router`)
- **Protocol**: AJR v1.5.4 packet processing
- **Routing**: Priority-based Lane selection ("Flash" vs. "Deep")
- **Immutability**: LaneID enforcement logic active

#### Market Layer (`services/gcam-node`)
- **Auction Engine**: Sealed-bid matching logic
- **Pricing**: Dynamic $C_{eff}$ calculation based on REC/SLP multipliers
- **Sovereignty**: Hard-filter enforcement for Regional constraints

#### Execution Layer (`services/gsee-runtime`)
- **Enclave**: "Strict Mock" runtime enabled
- **Policy**: Pre-execution compliance checks (Residency, Precision support)
- **Output**: Deterministic Proof-of-Performance (PoP) generation

#### Simulation (`tools/gix-sim`)
- **Full Lifecycle**: Complete "Tick" simulation from Job Generation → GCAM Auction → AJR Routing → GSEE Execution → Settlement
- **Integration**: End-to-end workflow using real service components

### Architecture

- **Reverse-Pyramid Dependencies**: Established dependency flow from crypto → GXF → common → services
- **Monorepo Structure**: Unified workspace with crates, services, tools, SDK, specs, and infra
- **Library/Service Split**: Services expose both binary and library interfaces for integration

### Known Limitations

- **Network I/O**: Internal function calls (no gRPC/Quic streams yet)
- **Cryptography**: Mock implementations (not production-ready PQC)
- **State**: In-memory only (no persistent ledger storage)

### Roadmap to v0.2.0

- [ ] **Network I/O**: Replace internal function calls with gRPC/Quic streams
- [ ] **Real Crypto**: Swap mocks for `pqcrypto-kyber` and `vdf` libraries
- [ ] **Persistent State**: Add `sled` or `rocksdb` for ledger storage

---

## [Unreleased]

### Planned
- Network protocol implementation
- Production-grade cryptography
- Persistent state management
- Performance optimizations
- Extended test coverage

