# GIX Docker Infrastructure

This directory contains Docker configurations for running GIX services in containers.

## Directory Structure

```
infra/docker/
├── Dockerfile.rust-builder   # Multi-stage builder for all services
├── Dockerfile.router          # AJR Router service
├── Dockerfile.auction         # GCAM Auction service
└── Dockerfile.enclave         # GSEE Execution service
```

## Dockerfiles

### Dockerfile.rust-builder

Multi-stage build that creates all GIX service binaries and runtime images in one file.

**Build Stages:**
1. **Builder:** Installs dependencies (protoc, llvm, clang), builds all workspace members
2. **Runtime Base:** Debian slim with CA certificates, non-root user
3. **Service Stages:** Router, Auction, Execution (each copies its specific binary)

**Features:**
- Optimized release builds with LTO
- Minimal runtime images (Debian Bookworm Slim)
- Non-root user (`gix`) for security
- Health checks on all services
- Persistent storage support (Auction)

### Individual Dockerfiles

Each service has its own Dockerfile for independent building:

- **Dockerfile.router:** AJR Router (port 50051)
- **Dockerfile.auction:** GCAM Auction (port 50052, with volume)
- **Dockerfile.enclave:** GSEE Execution (port 50053)

## Building Images

### Build All Services (Multi-stage)

```bash
# Build all images from the rust-builder
docker build -f infra/docker/Dockerfile.rust-builder \
  --target router -t gix-router:latest .

docker build -f infra/docker/Dockerfile.rust-builder \
  --target auction -t gix-auction:latest .

docker build -f infra/docker/Dockerfile.rust-builder \
  --target execution -t gix-enclave:latest .
```

### Build Individual Services

```bash
# Router
docker build -f infra/docker/Dockerfile.router -t gix-router:latest .

# Auction
docker build -f infra/docker/Dockerfile.auction -t gix-auction:latest .

# Execution
docker build -f infra/docker/Dockerfile.enclave -t gix-enclave:latest .
```

### Using Docker Compose (Recommended)

```bash
# Build all services
docker-compose build

# Build specific service
docker-compose build gix-auction
```

## Running Containers

### Manual Run

```bash
# Router
docker run -d --name gix-router -p 50051:50051 gix-router:latest

# Auction (with persistent storage)
docker run -d --name gix-auction -p 50052:50052 \
  -v gcam-data:/data gix-auction:latest

# Execution
docker run -d --name gix-enclave -p 50053:50053 gix-enclave:latest
```

### Using Docker Compose (Recommended)

```bash
# Start all services
docker-compose up -d

# Start specific service
docker-compose up -d gix-auction

# View logs
docker-compose logs -f gix-auction

# Stop all services
docker-compose down
```

## Image Details

### Base Images

- **Builder:** `rust:1.75-bookworm` (~1.5GB)
- **Runtime:** `debian:bookworm-slim` (~80MB)

### Final Image Sizes (Approximate)

- **gix-router:** ~150MB
- **gix-auction:** ~150MB
- **gix-enclave:** ~150MB

### Security Features

1. **Non-root User:** All services run as user `gix` (UID 1000)
2. **Minimal Runtime:** Only essential libraries included
3. **CA Certificates:** Included for secure connections
4. **Read-only Root:** Can be configured with `--read-only` flag

## Health Checks

All services include health checks:

```dockerfile
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD timeout 2 bash -c '</dev/tcp/localhost/PORT' || exit 1
```

**Ports:**
- Router: 50051
- Auction: 50052
- Execution: 50053

## Environment Variables

### Common Variables

- `RUST_LOG`: Logging level (e.g., `info`, `debug`)
- `RUST_BACKTRACE`: Enable backtraces (`0`, `1`, `full`)

### Service-Specific

**Auction Service:**
- `GIX_DATA_DIR`: Data directory for persistent storage (default: `/data`)

## Volumes

### Auction Service

Persistent volume for auction database:

```yaml
volumes:
  - gcam-data:/data
```

**Contents:**
- Market ledger
- Provider states
- Auction statistics

**Backup:**
```bash
# Backup
docker run --rm -v gcam-data:/data -v $(pwd):/backup \
  busybox tar czf /backup/gcam-backup.tar.gz -C /data .

# Restore
docker run --rm -v gcam-data:/data -v $(pwd):/backup \
  busybox tar xzf /backup/gcam-backup.tar.gz -C /data
```

## Networking

### Docker Compose Network

Custom bridge network `gix-net` (172.28.0.0/16):

```yaml
networks:
  gix-net:
    name: gix-network
    driver: bridge
```

**Service Hostnames:**
- `router` (gix-router)
- `auction` (gix-auction)
- `enclave` (gix-enclave)

### Inter-service Communication

Services can communicate using hostnames:

```rust
// From one service to another
let client = AuctionServiceClient::connect("http://auction:50052").await?;
```

## Troubleshooting

### Build Issues

**Problem:** `protoc not found`
```bash
# Ensure protobuf-compiler is installed in builder stage
RUN apt-get install -y protobuf-compiler
```

**Problem:** Build timeout
```bash
# Increase Docker build timeout
docker build --build-arg BUILDKIT_TIMEOUT=600 ...
```

### Runtime Issues

**Problem:** Port already in use
```bash
# Check what's using the port
lsof -i :50051

# Use different port
docker run -p 50055:50051 gix-router:latest
```

**Problem:** Permission denied on volume
```bash
# Fix volume permissions
docker run --rm -v gcam-data:/data busybox chown -R 1000:1000 /data
```

### Health Check Failures

```bash
# Check service logs
docker logs gix-auction

# Test port manually
docker exec gix-auction bash -c 'echo > /dev/tcp/localhost/50052'

# Inspect health status
docker inspect --format='{{.State.Health}}' gix-auction
```

## Production Considerations

### Security Hardening

1. **Use specific base image versions:**
   ```dockerfile
   FROM rust:1.75.0-bookworm as builder
   FROM debian:bookworm-20231218-slim as runtime
   ```

2. **Scan images for vulnerabilities:**
   ```bash
   docker scan gix-auction:latest
   trivy image gix-auction:latest
   ```

3. **Run as non-root (already configured)**

4. **Use read-only root filesystem:**
   ```bash
   docker run --read-only -v /tmp:/tmp gix-router:latest
   ```

### Performance Optimization

1. **Multi-stage build caching:**
   ```dockerfile
   # Cache dependencies
   COPY Cargo.toml Cargo.lock ./
   RUN cargo fetch
   ```

2. **BuildKit for parallel builds:**
   ```bash
   DOCKER_BUILDKIT=1 docker-compose build
   ```

3. **Resource limits:**
   ```yaml
   deploy:
     resources:
       limits:
         cpus: '2.0'
         memory: 2G
   ```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Build Docker images
  run: |
    docker-compose build --parallel
    
- name: Test containers
  run: |
    docker-compose up -d
    ./scripts/health_check.sh
```

### Registry Push

```bash
# Tag for registry
docker tag gix-auction:latest ghcr.io/gix-network/auction:latest

# Push to registry
docker push ghcr.io/gix-network/auction:latest
```

## References

- [Docker Multi-stage Builds](https://docs.docker.com/build/building/multi-stage/)
- [Docker Compose](https://docs.docker.com/compose/)
- [Dockerfile Best Practices](https://docs.docker.com/develop/dev-best-practices/)

---

**Last Updated:** December 6, 2025  
**Version:** 0.1.0


