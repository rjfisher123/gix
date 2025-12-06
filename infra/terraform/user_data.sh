#!/bin/bash
set -e

# User Data Script for GIX Node Deployment
# This script runs on first boot of the EC2 instance

echo "=== GIX Node Bootstrap Script Starting ==="
date

# Update system packages
echo "[1/6] Updating system packages..."
export DEBIAN_FRONTEND=noninteractive
apt-get update -y
apt-get upgrade -y

# Install Docker
echo "[2/6] Installing Docker..."
apt-get install -y \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg \
    lsb-release

# Add Docker's official GPG key
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

# Set up Docker repository
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null

apt-get update -y
apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin

# Start and enable Docker
systemctl start docker
systemctl enable docker

# Install Docker Compose standalone (v2.x)
echo "[3/6] Installing Docker Compose..."
DOCKER_COMPOSE_VERSION="v2.24.0"
curl -SL "https://github.com/docker/compose/releases/download/$${DOCKER_COMPOSE_VERSION}/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
chmod +x /usr/local/bin/docker-compose

# Install Git
echo "[4/6] Installing Git..."
apt-get install -y git

# Clone GIX repository
echo "[5/6] Cloning GIX repository..."
cd /opt
git clone ${git_repo_url} gix
cd gix
git checkout ${git_branch}

# Make scripts executable
chmod +x scripts/*.sh

# Deploy GIX network
echo "[6/6] Deploying GIX localnet..."
cd /opt/gix
./scripts/deploy_localnet.sh

# Set up log rotation for Docker
cat > /etc/docker/daemon.json <<'EOF'
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  }
}
EOF
systemctl restart docker

# Create systemd service for GIX (auto-restart on reboot)
cat > /etc/systemd/system/gix-network.service <<'EOF'
[Unit]
Description=GIX Distributed Intelligence Network
After=docker.service
Requires=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/gix
ExecStart=/usr/local/bin/docker-compose up -d
ExecStop=/usr/local/bin/docker-compose down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable gix-network.service

echo "=== GIX Node Bootstrap Complete ==="
echo "Node is now running and ready to accept connections."
date

# Log deployment info
echo "Deployment completed at $(date)" >> /opt/gix/deployment.log
docker ps >> /opt/gix/deployment.log

