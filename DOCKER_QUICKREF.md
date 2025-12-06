# GIX Docker Quick Reference

**Version:** 0.1.0  
**Last Updated:** December 6, 2025

---

## Quick Start

```bash
# Deploy complete stack
./scripts/deploy_localnet.sh

# Or manually
docker-compose up -d
```

---

## Commands

### Build

```bash
# Build all services
docker-compose build

# Build specific service
docker-compose build gix-auction

# Build without cache
docker-compose build --no-cache

# Parallel build
docker-compose build --parallel
```

### Start/Stop

```bash
# Start all services
docker-compose up -d

# Start specific service
docker-compose up -d gix-router

# Stop all services
docker-compose down

# Stop and remove volumes
docker-compose down -v

# Restart service
docker-compose restart gix-auction
```

### Logs

```bash
# View all logs
docker-compose logs

# Follow logs
docker-compose logs -f

# Specific service
docker-compose logs -f gix-auction

# Last 100 lines
docker-compose logs --tail=100 gix-router
```

### Status

```bash
# Service status
docker-compose ps

# Detailed service info
docker ps

# Health status
docker inspect --format='{{.State.Health.Status}}' gix-auction
```

### Shell Access

```bash
# Execute command in container
docker exec gix-auction ls -la /data

# Interactive shell
docker exec -it gix-auction bash

# As root
docker exec -it --user root gix-auction bash
```

---

## Service Ports

| Service | Port | URL |
|---------|------|-----|
| **AJR Router** | 50051 | http://127.0.0.1:50051 |
| **GCAM Auction** | 50052 | http://127.0.0.1:50052 |
| **GSEE Execution** | 50053 | http://127.0.0.1:50053 |

---

## Testing

### Submit Job

```bash
# Start services
docker-compose up -d

# Wait for health
sleep 10

# Submit job
cargo run -p gix-cli -- submit examples/job_sample.yaml

# Check status
cargo run -p gix-cli -- status
```

### Port Connectivity

```bash
# Test router
timeout 2 bash -c 'echo > /dev/tcp/127.0.0.1/50051' && echo "OK"

# Test auction
timeout 2 bash -c 'echo > /dev/tcp/127.0.0.1/50052' && echo "OK"

# Test execution
timeout 2 bash -c 'echo > /dev/tcp/127.0.0.1/50053' && echo "OK"
```

---

## Volume Management

### Backup Auction Data

```bash
# Create backup
docker run --rm -v gix-gcam-data:/data -v $(pwd):/backup \
  busybox tar czf /backup/gcam-backup-$(date +%Y%m%d).tar.gz -C /data .

# Verify backup
tar -tzf gcam-backup-*.tar.gz
```

### Restore Auction Data

```bash
# Stop service
docker-compose stop gix-auction

# Restore
docker run --rm -v gix-gcam-data:/data -v $(pwd):/backup \
  busybox tar xzf /backup/gcam-backup-20251206.tar.gz -C /data

# Start service
docker-compose start gix-auction
```

### Inspect Volume

```bash
# List volume contents
docker run --rm -v gix-gcam-data:/data busybox ls -lah /data

# Volume size
docker run --rm -v gix-gcam-data:/data busybox du -sh /data
```

### Clean Volume

```bash
# Remove volume (WARNING: deletes all data!)
docker-compose down -v

# Or manually
docker volume rm gix-gcam-data
```

---

## Troubleshooting

### View Logs

```bash
# All services
docker-compose logs

# With timestamps
docker-compose logs -t

# Specific service with follow
docker-compose logs -f gix-auction
```

### Health Checks

```bash
# Check health status
docker inspect gix-auction --format='{{json .State.Health}}' | jq

# View health check command
docker inspect gix-auction --format='{{.Config.Healthcheck.Test}}'

# Manually test health
docker exec gix-auction bash -c 'echo > /dev/tcp/localhost/50052'
```

### Restart Unhealthy Service

```bash
# Restart
docker-compose restart gix-auction

# Or force recreate
docker-compose up -d --force-recreate gix-auction
```

### Clean Slate

```bash
# Stop everything
docker-compose down

# Remove all containers, networks, volumes
docker-compose down -v --remove-orphans

# Remove images
docker-compose down --rmi all

# Full clean
docker system prune -a
```

---

## Network

### Inspect Network

```bash
# List networks
docker network ls

# Inspect gix network
docker network inspect gix-network

# See connected containers
docker network inspect gix-network --format='{{range .Containers}}{{.Name}} {{end}}'
```

### Test Inter-service Communication

```bash
# From host to auction
curl -v http://127.0.0.1:50052

# From router to auction (internal DNS)
docker exec gix-router ping -c 3 auction

# Test gRPC connectivity
docker exec gix-router timeout 2 bash -c 'echo > /dev/tcp/auction/50052'
```

---

## Performance

### Resource Usage

```bash
# Real-time stats
docker stats

# Specific service
docker stats gix-auction

# One-time snapshot
docker stats --no-stream
```

### Image Sizes

```bash
# List images
docker images | grep gix

# Detailed image info
docker inspect gix-auction:latest --format='{{.Size}}'

# Disk usage
docker system df
```

---

## Development

### Rebuild After Code Changes

```bash
# Rebuild and restart
docker-compose up -d --build

# Specific service
docker-compose up -d --build gix-auction
```

### Debug Build

```bash
# Build with verbose output
docker-compose build --progress=plain gix-auction

# No cache (clean build)
docker-compose build --no-cache gix-auction
```

### Override Configuration

Create `docker-compose.override.yml`:

```yaml
version: '3.8'
services:
  gix-auction:
    environment:
      - RUST_LOG=debug
    ports:
      - "9090:9090"  # Additional port
```

---

## Production Tips

### Security Scanning

```bash
# Scan for vulnerabilities
docker scan gix-auction:latest

# Or use trivy
trivy image gix-auction:latest
```

### Tag for Registry

```bash
# Tag image
docker tag gix-auction:latest ghcr.io/gix-network/auction:0.1.0

# Push to registry
docker push ghcr.io/gix-network/auction:0.1.0
```

### Run with Resource Limits

```bash
# CPU and memory limits
docker run -d \
  --cpus="2.0" \
  --memory="2g" \
  --name gix-auction \
  gix-auction:latest
```

---

## Environment Variables

### Common Variables

```bash
# Set log level
RUST_LOG=debug docker-compose up -d

# Enable backtraces
RUST_BACKTRACE=1 docker-compose up -d
```

### Service-Specific

```bash
# Auction data directory
docker run -e GIX_DATA_DIR=/custom/path gix-auction:latest
```

---

## Useful Aliases

Add to `.bashrc` or `.zshrc`:

```bash
# GIX Docker aliases
alias gix-up='docker-compose up -d'
alias gix-down='docker-compose down'
alias gix-logs='docker-compose logs -f'
alias gix-ps='docker-compose ps'
alias gix-restart='docker-compose restart'
alias gix-rebuild='docker-compose up -d --build'

# Specific services
alias gix-logs-auction='docker-compose logs -f gix-auction'
alias gix-shell-auction='docker exec -it gix-auction bash'
```

---

## Quick Diagnostics

```bash
# Check everything
docker-compose ps                          # Service status
docker network inspect gix-network         # Network config
docker volume ls | grep gix                # Volumes
docker stats --no-stream                   # Resource usage
docker-compose logs --tail=50              # Recent logs

# Health summary
for service in gix-router gix-auction gix-enclave; do
  echo -n "$service: "
  docker inspect --format='{{.State.Health.Status}}' $service
done
```

---

**For detailed information, see:** `infra/docker/README.md`

