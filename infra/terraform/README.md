# GIX Cloud Deployment with Terraform

This directory contains Terraform configuration for deploying GIX network nodes to AWS.

## Prerequisites

1. **Install Terraform:**
   ```bash
   # macOS
   brew install terraform
   
   # Linux
   wget https://releases.hashicorp.com/terraform/1.6.0/terraform_1.6.0_linux_amd64.zip
   unzip terraform_1.6.0_linux_amd64.zip
   sudo mv terraform /usr/local/bin/
   ```

2. **Configure AWS Credentials:**
   ```bash
   aws configure
   # Enter your AWS Access Key ID, Secret Access Key, and default region
   ```

3. **Create SSH Key Pair:**
   ```bash
   aws ec2 create-key-pair \
     --key-name gix-node-key \
     --query 'KeyMaterial' \
     --output text > ~/.ssh/gix-node-key.pem
   
   chmod 400 ~/.ssh/gix-node-key.pem
   ```

## Deployment Steps

### 1. Initialize Terraform

```bash
cd infra/terraform
terraform init
```

### 2. Configure Variables

Copy the example variables file and customize:

```bash
cp terraform.tfvars.example terraform.tfvars
nano terraform.tfvars  # Edit with your values
```

Required variables:
- `ssh_key_name`: Name of your AWS SSH key pair
- `region`: AWS region (default: us-east-1)
- `git_repo_url`: Your GIX repository URL

### 3. Plan Deployment

Preview the infrastructure changes:

```bash
terraform plan
```

### 4. Deploy

Apply the configuration:

```bash
terraform apply
```

Type `yes` when prompted to confirm.

### 5. Access Your Node

After deployment completes (5-10 minutes), Terraform will output:

```
Outputs:

public_ip = "54.123.456.789"
router_endpoint = "54.123.456.789:50051"
auction_endpoint = "54.123.456.789:50052"
execution_endpoint = "54.123.456.789:50053"
grafana_url = "http://54.123.456.789:3000"
prometheus_url = "http://54.123.456.789:9090"
ssh_command = "ssh -i ~/.ssh/gix-node-key.pem ubuntu@54.123.456.789"
```

### 6. Verify Deployment

**SSH into the node:**
```bash
ssh -i ~/.ssh/gix-node-key.pem ubuntu@<PUBLIC_IP>
```

**Check Docker containers:**
```bash
docker ps
```

**View logs:**
```bash
cd /opt/gix
docker-compose logs -f
```

### 7. Use the Node with GIX CLI

Update your local `gix-cli` to connect to the cloud node:

```bash
# Generate keys (if you haven't already)
gix-cli keygen

# Submit a job to the cloud node
gix-cli submit \
  --router-url http://<PUBLIC_IP>:50051 \
  examples/job_sample.yaml

# Check status
gix-cli status --router-url http://<PUBLIC_IP>:50051
```

### 8. Access Monitoring

- **Grafana:** Open `http://<PUBLIC_IP>:3000` in your browser
  - Default credentials: `admin` / `admin`
- **Prometheus:** Open `http://<PUBLIC_IP>:9090`

## Scaling: Deploy Multiple Nodes

To deploy multiple nodes:

```bash
# Deploy node 1
terraform apply -var="node_id=01"

# Deploy node 2 (in separate directory or workspace)
terraform apply -var="node_id=02"
```

Or use Terraform workspaces:

```bash
terraform workspace new node-02
terraform apply
```

## Cleanup

To destroy all resources:

```bash
terraform destroy
```

Type `yes` to confirm.

## Cost Estimation

- **t3.medium instance:** ~$0.0416/hour (~$30/month)
- **30 GB EBS volume:** ~$3/month
- **Elastic IP:** Free while attached
- **Data transfer:** Variable (typically $0.09/GB out)

**Estimated monthly cost:** ~$35-50/month per node

## Security Best Practices

1. **Restrict SSH access:**
   Edit `main.tf` to limit SSH to your IP:
   ```hcl
   cidr_blocks = ["YOUR_IP/32"]
   ```

2. **Enable AWS CloudWatch:**
   Monitor instance health and set up alarms.

3. **Use AWS Secrets Manager:**
   For production, store sensitive values in Secrets Manager.

4. **Enable MFA:**
   Require multi-factor authentication for AWS console access.

5. **Regular updates:**
   SSH into nodes periodically and run:
   ```bash
   cd /opt/gix
   git pull
   docker-compose up --build -d
   ```

## Troubleshooting

**Issue: Terraform can't find AMI**
- Update `ami_id` in `terraform.tfvars` for your region
- Find AMI: `aws ec2 describe-images --owners 099720109477 --filters "Name=name,Values=ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*" --region YOUR_REGION`

**Issue: Connection timeout**
- Check security group rules in AWS Console
- Verify instance is running: `terraform show`
- Check user_data logs: `ssh` into instance and run `cat /var/log/cloud-init-output.log`

**Issue: Docker containers not starting**
- SSH into node: `ssh -i ~/.ssh/gix-node-key.pem ubuntu@<IP>`
- Check logs: `cd /opt/gix && docker-compose logs`
- Rebuild: `docker-compose up --build -d`

## Support

For issues or questions:
- GitHub Issues: https://github.com/rjfisher123/gix/issues
- Documentation: https://github.com/rjfisher123/gix/blob/main/README.md

