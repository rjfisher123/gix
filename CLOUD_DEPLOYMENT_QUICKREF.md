# GIX Cloud Deployment Quick Reference

## One-Command Deployment

```bash
cd infra/terraform
terraform init
terraform apply -auto-approve
```

## Prerequisites Setup

```bash
# Install Terraform (macOS)
brew install terraform

# Configure AWS
aws configure
# Enter: Access Key ID, Secret Access Key, Region (us-east-1)

# Create SSH key
aws ec2 create-key-pair \
  --key-name gix-node-key \
  --query 'KeyMaterial' \
  --output text > ~/.ssh/gix-node-key.pem
chmod 400 ~/.ssh/gix-node-key.pem
```

## Configuration

```bash
# Create config file
cat > infra/terraform/terraform.tfvars <<EOF
region        = "us-east-1"
instance_type = "t3.medium"
ssh_key_name  = "gix-node-key"
git_repo_url  = "https://github.com/rjfisher123/gix.git"
git_branch    = "main"
node_id       = "01"
EOF
```

## Deployment

```bash
cd infra/terraform

# Initialize (first time only)
terraform init

# Deploy
terraform apply
# Type "yes" to confirm
# Wait 5-10 minutes for bootstrap
```

## Access Your Node

```bash
# Get public IP (from terraform output)
export NODE_IP=$(terraform output -raw public_ip)

# SSH into node
ssh -i ~/.ssh/gix-node-key.pem ubuntu@$NODE_IP

# Check deployment status
tail -f /opt/gix/deployment.log

# Verify services
docker ps
```

## Use with GIX CLI

```bash
# Submit job to cloud node
gix-cli submit \
  --router-url http://$NODE_IP:50051 \
  examples/job_sample.yaml

# Check status
gix-cli status --router-url http://$NODE_IP:50051
```

## Access Monitoring

- **Grafana**: http://$NODE_IP:3000 (admin/admin)
- **Prometheus**: http://$NODE_IP:9090
- **Router Metrics**: http://$NODE_IP:9001/metrics
- **Auction Metrics**: http://$NODE_IP:9002/metrics

## Cleanup

```bash
terraform destroy
# Type "yes" to confirm
```

## Multi-Node Deployment

```bash
# Deploy node 1
terraform workspace new node-01
terraform apply -var="node_id=01"

# Deploy node 2
terraform workspace new node-02
terraform apply -var="node_id=02"

# List workspaces
terraform workspace list

# Switch workspace
terraform workspace select node-01
```

## Cost Estimate

- **t3.medium**: ~$30/month
- **30GB EBS**: ~$2.40/month
- **Data transfer**: ~$5-15/month
- **Total**: ~$35-50/month per node

## Troubleshooting

### Connection Timeout
```bash
# Check security group
aws ec2 describe-security-groups --filters "Name=group-name,Values=gix-node-security-group"

# Check instance status
terraform show | grep instance_state
```

### Bootstrap Failed
```bash
# SSH into instance
ssh -i ~/.ssh/gix-node-key.pem ubuntu@$NODE_IP

# Check logs
cat /var/log/cloud-init-output.log

# Manually restart deployment
cd /opt/gix
docker-compose up --build -d
```

### Docker Issues
```bash
# SSH into node
ssh -i ~/.ssh/gix-node-key.pem ubuntu@$NODE_IP

# Check Docker status
systemctl status docker

# Check container logs
cd /opt/gix
docker-compose logs -f
```

## Security Hardening

```bash
# Restrict SSH to your IP (edit main.tf)
ingress {
  description = "SSH"
  from_port   = 22
  to_port     = 22
  protocol    = "tcp"
  cidr_blocks = ["YOUR_IP/32"]  # Change this
}

# Apply changes
terraform apply
```

## Terraform Commands

```bash
# Show current state
terraform show

# List resources
terraform state list

# Get specific output
terraform output public_ip

# Refresh state
terraform refresh

# Validate config
terraform validate

# Format code
terraform fmt

# Plan without applying
terraform plan -out=plan.tfplan
```

## AWS Regions

Common AMI IDs for Ubuntu 22.04 LTS:

| Region | AMI ID |
|--------|--------|
| us-east-1 | ami-0c7217cdde317cfec |
| us-west-2 | ami-0fcf52bcf5db7b003 |
| eu-west-1 | ami-0d2a4a5d69e46ea0b |
| ap-southeast-1 | ami-0dc2d3e4c0f9ebd18 |

Find latest:
```bash
aws ec2 describe-images \
  --owners 099720109477 \
  --filters "Name=name,Values=ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*" \
  --region YOUR_REGION \
  --query 'Images[*].[ImageId,CreationDate]' \
  --output text | sort -k2 -r | head -n1
```

## Support

- Full Guide: [infra/terraform/README.md](infra/terraform/README.md)
- Phase 6 Docs: [PHASE_6_CLOUD_DEPLOYMENT_COMPLETE.md](PHASE_6_CLOUD_DEPLOYMENT_COMPLETE.md)
- GitHub Issues: https://github.com/rjfisher123/gix/issues

