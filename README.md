# GIX: Global Intelligence Exchange Monorepo

**Version:** v0.2.0 "Distributed Network Release"  
**Status:** Beta - Production Infrastructure Ready  
**Date:** December 6, 2025

Unified development repository for all components of the Global Intelligence Exchange (GIX).

## Components

- **GSEE** (Secure Execution Envelope) - Secure enclave execution runtime
- **AJR** (Anonymized Job Routing) - Mixnet routing service
- **GCAM** (Global Compute Auction Mechanism) - Auction clearing engine with persistent storage

## Architecture

GIX follows a **reverse-pyramid** development methodology:

1. **Crypto Layer** - Post-quantum cryptographic primitives (Kyber1024, Dilithium3, VDF)
2. **GXF Layer** - Exchange format schema and validation
3. **Common Layer** - Shared types and utilities
4. **Network Layer** - gRPC protocol definitions and microservices
5. **Service Layer** - Runnable daemons and services

## Repository Structure

```
gix/
├── crates/          # Shared libraries (non-running modules)
├── services/        # Binaries/daemons (runnable)
├── tools/           # Development and operational tools
├── sdk/             # Client libraries (Rust, Python, JS)
├── specs/           # Source of truth (specification documents)
└── infra/           # Infrastructure-as-code (Terraform, Docker, K8s)
```

## Release Information

**Current Release:** v0.2.0 "Distributed Network Release"

This release delivers a production-ready distributed architecture with:
- ✅ gRPC microservices with Protocol Buffers
- ✅ Production-grade post-quantum cryptography
- ✅ Persistent storage with crash recovery
- ✅ Docker containerization
- ✅ Prometheus metrics & Grafana dashboards
- ✅ User-facing CLI tool
- ✅ Complete observability stack

See [RELEASE_NOTES.md](RELEASE_NOTES.md) for detailed release information and [CHANGELOG.md](CHANGELOG.md) for version history.

## Getting Started

### Prerequisites

- **Docker & Docker Compose** - For containerized deployment (recommended)
- **Rust toolchain** (stable, 2021 edition) - For building from source
- **Protocol Buffer Compiler (`protoc`)** - Required for building `gix-proto`
  - macOS: `brew install protobuf`
  - Linux: `sudo apt-get install protobuf-compiler`
  - Windows: Download from [protobuf releases](https://github.com/protocolbuffers/protobuf/releases)

### Quickstart (Docker - Recommended)

Deploy the complete GIX stack with monitoring in one command:

```bash
# Deploy all services
./scripts/deploy_localnet.sh

# Wait for services to start (~30 seconds)
# Services will be available at:
#   - AJR Router:     http://localhost:50051
#   - GCAM Auction:   http://localhost:50052
#   - GSEE Execution: http://localhost:50053
#   - Prometheus:     http://localhost:9090
#   - Grafana:        http://localhost:3000 (admin/admin)
```

**Using the CLI:**

```bash
# Generate a wallet (first time only)
cargo run -p gix-cli -- keygen

# Submit a job
cargo run -p gix-cli -- submit examples/job_sample.yaml

# Check auction status
cargo run -p gix-cli -- status
```

**Stop Services:**

```bash
docker-compose down
```

### Alternative: Build from Source

```bash
# Build all crates and services
cargo build --workspace --release

# Run services individually (3 terminals)
cargo run --release --bin ajr-router
cargo run --release --bin gcam-node
cargo run --release --bin gsee-runtime

# Submit a job (4th terminal)
cargo run -p gix-cli -- keygen
cargo run -p gix-cli -- submit examples/job_sample.yaml
```

## Development Guidelines

See [.cursor/rules.md](.cursor/rules.md) for detailed development rules and guidelines.

Key principles:
- **No unsafe Rust** (unless absolutely necessary and well-reviewed)
- **No secret leakage** in logs or error messages
- **Specification-driven development** (implement only what's in `specs/`)
- **Reverse-pyramid dependencies** (services → common/GXF → crypto)

## Monitoring

GIX includes a complete observability stack with Prometheus and Grafana:

### Access Dashboards

- **Grafana**: http://localhost:3000
  - Username: `admin`
  - Password: `admin`
  - Pre-configured with Prometheus data source
  - Create custom dashboards for your metrics

- **Prometheus**: http://localhost:9090
  - Query metrics with PromQL
  - View service targets and health
  - Create alerting rules

### Raw Metrics Endpoints

- **Router Metrics**: http://localhost:9001/metrics
- **Auction Metrics**: http://localhost:9002/metrics

### Key Metrics

**Router (AJR):**
- `gix_packets_routed_total{lane}` - Packets routed per lane
- `gix_router_active_jobs{lane}` - Active jobs per lane

**Auction (GCAM):**
- `gix_clearing_price{slp}` - Current clearing price
- `gix_auctions_total` - Total auctions processed
- `gix_provider_utilization{slp}` - Provider utilization

### Example Queries

```promql
# Auction rate (per second)
rate(gix_auctions_total[5m])

# Average clearing price
avg(gix_clearing_price)

# Router throughput by lane
sum(rate(gix_packets_routed_total[5m])) by (lane)
```

See [OBSERVABILITY_QUICKREF.md](OBSERVABILITY_QUICKREF.md) for comprehensive monitoring guide.

## Cloud Deployment

Deploy GIX to AWS (or other cloud providers) using Terraform:

### Prerequisites

1. **Install Terraform:**
   ```bash
   brew install terraform  # macOS
   ```

2. **Configure AWS credentials:**
   ```bash
   aws configure
   ```

3. **Create SSH key pair:**
   ```bash
   aws ec2 create-key-pair \
     --key-name gix-node-key \
     --query 'KeyMaterial' \
     --output text > ~/.ssh/gix-node-key.pem
   chmod 400 ~/.ssh/gix-node-key.pem
   ```

### Deploy to AWS

```bash
cd infra/terraform

# Initialize Terraform
terraform init

# Configure variables (copy and edit)
cp terraform.tfvars.example terraform.tfvars
nano terraform.tfvars

# Preview deployment
terraform plan

# Deploy (takes 5-10 minutes)
terraform apply
```

After deployment, Terraform outputs the public IP and service endpoints:

```
Outputs:

public_ip = "54.123.456.789"
router_endpoint = "54.123.456.789:50051"
auction_endpoint = "54.123.456.789:50052"
execution_endpoint = "54.123.456.789:50053"
grafana_url = "http://54.123.456.789:3000"
```

### Use Cloud Node with CLI

```bash
# Submit jobs to your cloud node
gix-cli submit \
  --router-url http://54.123.456.789:50051 \
  examples/job_sample.yaml

# Check status
gix-cli status --router-url http://54.123.456.789:50051
```

### Cleanup

```bash
terraform destroy
```

**See [infra/terraform/README.md](infra/terraform/README.md) for complete cloud deployment guide.**

## Specifications

All implementation is driven by specifications in `specs/`:
- `crypto_spec.md` - Cryptographic requirements
- `gxf_spec.md` - GXF format specification
- Component-specific specs in `specs/ajr/`, `specs/gsee/`, `specs/gcam/`

## Security

GIX uses post-quantum cryptographic primitives:
- **Kyber** for key encapsulation
- **Dilithium** for digital signatures
- **Blake3** for hashing

All cryptographic code must be:
- Constant-time
- Side-channel resistant
- Properly audited

## License

Apache-2.0 OR MIT

## Contributing

1. Read the specifications in `specs/`
2. Follow the development guidelines in `.cursor/rules.md`
3. Ensure all tests pass and code is formatted
4. Submit PRs with clear descriptions



