#!/usr/bin/env bash
#
# GIX LocalNet Deployment Script
#
# Deploys the complete GIX network stack using Docker Compose.
# Includes health checks and status verification.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Change to project root
cd "${PROJECT_ROOT}"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           GIX LocalNet Deployment Script                  ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo -e "${RED}✗ Error: Docker is not running${NC}"
    echo "Please start Docker and try again."
    exit 1
fi

echo -e "${GREEN}✓ Docker is running${NC}"

# Check if Docker Compose is available
if ! command -v docker-compose &> /dev/null; then
    if ! docker compose version &> /dev/null; then
        echo -e "${RED}✗ Error: Docker Compose is not installed${NC}"
        exit 1
    fi
    DOCKER_COMPOSE="docker compose"
else
    DOCKER_COMPOSE="docker-compose"
fi

echo -e "${GREEN}✓ Docker Compose is available${NC}"
echo ""

# Stop existing containers if running
echo -e "${YELLOW}→ Stopping existing containers (if any)...${NC}"
$DOCKER_COMPOSE down --remove-orphans || true
echo ""

# Build and start services
echo -e "${YELLOW}→ Building Docker images...${NC}"
echo "This may take several minutes on first run."
echo ""

$DOCKER_COMPOSE build --no-cache

echo ""
echo -e "${GREEN}✓ Images built successfully${NC}"
echo ""

# Start services
echo -e "${YELLOW}→ Starting GIX services...${NC}"
$DOCKER_COMPOSE up -d

echo ""
echo -e "${GREEN}✓ Services started${NC}"
echo ""

# Wait for health checks
echo -e "${YELLOW}→ Waiting for services to become healthy...${NC}"
echo ""

MAX_WAIT=60  # Maximum wait time in seconds
WAIT_INTERVAL=2  # Check interval in seconds
elapsed=0

services=("gix-router" "gix-auction" "gix-enclave")

for service in "${services[@]}"; do
    echo -n "  Checking ${service}... "
    
    while [ $elapsed -lt $MAX_WAIT ]; do
        health_status=$(docker inspect --format='{{.State.Health.Status}}' ${service} 2>/dev/null || echo "starting")
        
        if [ "$health_status" = "healthy" ]; then
            echo -e "${GREEN}✓ healthy${NC}"
            break
        fi
        
        sleep $WAIT_INTERVAL
        elapsed=$((elapsed + WAIT_INTERVAL))
        echo -n "."
    done
    
    if [ $elapsed -ge $MAX_WAIT ]; then
        echo -e "${RED}✗ timeout${NC}"
        echo ""
        echo -e "${RED}Service ${service} did not become healthy within ${MAX_WAIT} seconds${NC}"
        echo "Check logs with: docker-compose logs ${service}"
        exit 1
    fi
    
    elapsed=0
done

echo ""
echo -e "${GREEN}✓ All services are healthy${NC}"
echo ""

# Display service status
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                   Service Status                           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""

$DOCKER_COMPOSE ps

echo ""

# Display connection information
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                Connection Information                      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${GREEN}AJR Router:${NC}      http://127.0.0.1:50051"
echo -e "  ${GREEN}GCAM Auction:${NC}    http://127.0.0.1:50052"
echo -e "  ${GREEN}GSEE Execution:${NC}  http://127.0.0.1:50053"
echo ""

# Display useful commands
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                   Useful Commands                          ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "  View logs:           docker-compose logs -f [service]"
echo "  Stop services:       docker-compose down"
echo "  Restart service:     docker-compose restart [service]"
echo "  Service shell:       docker exec -it [container] bash"
echo ""
echo "  Example - Submit job:"
echo "    cargo run -p gix-cli -- submit examples/job_sample.yaml"
echo ""
echo "  Example - Check status:"
echo "    cargo run -p gix-cli -- status"
echo ""

# Test connectivity
echo -e "${YELLOW}→ Testing service connectivity...${NC}"
echo ""

test_port() {
    local port=$1
    local service=$2
    
    if timeout 2 bash -c "echo > /dev/tcp/127.0.0.1/${port}" 2>/dev/null; then
        echo -e "  ${GREEN}✓ ${service} (port ${port}) is accessible${NC}"
        return 0
    else
        echo -e "  ${RED}✗ ${service} (port ${port}) is not accessible${NC}"
        return 1
    fi
}

all_accessible=true

test_port 50051 "AJR Router" || all_accessible=false
test_port 50052 "GCAM Auction" || all_accessible=false
test_port 50053 "GSEE Execution" || all_accessible=false

echo ""

if [ "$all_accessible" = true ]; then
    echo -e "${GREEN}✓ All services are accessible${NC}"
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║          GIX LocalNet is running successfully!             ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${YELLOW}⚠ Some services may not be fully accessible yet${NC}"
    echo "Wait a few moments and check logs if issues persist."
    exit 1
fi

