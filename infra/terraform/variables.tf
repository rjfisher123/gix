variable "region" {
  description = "AWS region to deploy resources"
  type        = string
  default     = "us-east-1"
}

variable "ami_id" {
  description = "AMI ID for Ubuntu 22.04 LTS (update based on region)"
  type        = string
  # Ubuntu 22.04 LTS in us-east-1 (x86_64)
  # Find latest: aws ec2 describe-images --owners 099720109477 --filters "Name=name,Values=ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*" --query 'Images[*].[ImageId,CreationDate]' --output text | sort -k2 -r | head -n1
  default = "ami-0c7217cdde317cfec"
}

variable "instance_type" {
  description = "EC2 instance type"
  type        = string
  default     = "t3.medium"
}

variable "ssh_key_name" {
  description = "Name of the SSH key pair to use for EC2 access"
  type        = string
  # Must be created in AWS Console or via CLI before running terraform
  # Example: aws ec2 create-key-pair --key-name gix-node-key --query 'KeyMaterial' --output text > gix-node-key.pem
}

variable "git_repo_url" {
  description = "Git repository URL for GIX codebase"
  type        = string
  default     = "https://github.com/rjfisher123/gix.git"
}

variable "git_branch" {
  description = "Git branch to deploy"
  type        = string
  default     = "main"
}

variable "node_id" {
  description = "Unique identifier for this node (used in tags)"
  type        = string
  default     = "01"
}

