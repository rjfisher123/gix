output "instance_id" {
  description = "EC2 Instance ID"
  value       = aws_instance.gix_node.id
}

output "public_ip" {
  description = "Public IP address of the GIX node"
  value       = aws_eip.gix_node_eip.public_ip
}

output "public_dns" {
  description = "Public DNS name of the GIX node"
  value       = aws_instance.gix_node.public_dns
}

output "ssh_command" {
  description = "SSH command to connect to the node"
  value       = "ssh -i ~/.ssh/${var.ssh_key_name}.pem ubuntu@${aws_eip.gix_node_eip.public_ip}"
}

output "router_endpoint" {
  description = "Router Service (AJR) endpoint"
  value       = "${aws_eip.gix_node_eip.public_ip}:50051"
}

output "auction_endpoint" {
  description = "Auction Service (GCAM) endpoint"
  value       = "${aws_eip.gix_node_eip.public_ip}:50052"
}

output "execution_endpoint" {
  description = "Execution Service (GSEE) endpoint"
  value       = "${aws_eip.gix_node_eip.public_ip}:50053"
}

output "grafana_url" {
  description = "Grafana monitoring dashboard URL"
  value       = "http://${aws_eip.gix_node_eip.public_ip}:3000"
}

output "prometheus_url" {
  description = "Prometheus metrics URL"
  value       = "http://${aws_eip.gix_node_eip.public_ip}:9090"
}

output "deployment_info" {
  description = "Complete deployment information"
  value = {
    node_id            = var.node_id
    region             = var.region
    instance_type      = var.instance_type
    public_ip          = aws_eip.gix_node_eip.public_ip
    router_endpoint    = "${aws_eip.gix_node_eip.public_ip}:50051"
    auction_endpoint   = "${aws_eip.gix_node_eip.public_ip}:50052"
    execution_endpoint = "${aws_eip.gix_node_eip.public_ip}:50053"
    grafana_dashboard  = "http://${aws_eip.gix_node_eip.public_ip}:3000"
    prometheus         = "http://${aws_eip.gix_node_eip.public_ip}:9090"
  }
}

