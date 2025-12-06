# GIX Network - Cloud Deployment Infrastructure
# Default provider: AWS (can be adapted for GCP, Azure, etc.)

terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.region
}

# Security Group for GIX Node
resource "aws_security_group" "gix_node_sg" {
  name        = "gix-node-security-group"
  description = "Security group for GIX network node"

  # SSH Access
  ingress {
    description = "SSH"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Router Service (AJR)
  ingress {
    description = "Router gRPC"
    from_port   = 50051
    to_port     = 50051
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Auction Service (GCAM)
  ingress {
    description = "Auction gRPC"
    from_port   = 50052
    to_port     = 50052
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Execution Service (GSEE)
  ingress {
    description = "Enclave gRPC"
    from_port   = 50053
    to_port     = 50053
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Prometheus
  ingress {
    description = "Prometheus"
    from_port   = 9090
    to_port     = 9090
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Grafana Dashboard
  ingress {
    description = "Grafana"
    from_port   = 3000
    to_port     = 3000
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Router Metrics
  ingress {
    description = "Router Metrics"
    from_port   = 9001
    to_port     = 9001
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Auction Metrics
  ingress {
    description = "Auction Metrics"
    from_port   = 9002
    to_port     = 9002
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Outbound (allow all)
  egress {
    description = "All outbound traffic"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name    = "gix-node-sg"
    Project = "GIX"
  }
}

# EC2 Instance for GIX Node
resource "aws_instance" "gix_node" {
  ami                    = var.ami_id
  instance_type          = var.instance_type
  key_name               = var.ssh_key_name
  vpc_security_group_ids = [aws_security_group.gix_node_sg.id]

  root_block_device {
    volume_size = 30
    volume_type = "gp3"
  }

  user_data = templatefile("${path.module}/user_data.sh", {
    git_repo_url = var.git_repo_url
    git_branch   = var.git_branch
  })

  tags = {
    Name    = "gix-node-${var.node_id}"
    Project = "GIX"
    Version = "v0.2.0"
  }
}

# Elastic IP (optional but recommended for stable addressing)
resource "aws_eip" "gix_node_eip" {
  instance = aws_instance.gix_node.id
  domain   = "vpc"

  tags = {
    Name    = "gix-node-eip"
    Project = "GIX"
  }
}

