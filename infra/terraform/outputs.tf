output "environment" {
  description = "Active deployment environment"
  value       = var.environment
}

output "network" {
  description = "Stellar network used for deployment"
  value       = var.network
}

output "admin_address" {
  description = "Stellar public address of the admin account used for deployment"
  value       = length(data.local_file.admin_address) > 0 ? trimspace(data.local_file.admin_address[0].content) : ""
}

output "token_contract_id" {
  description = "Deployed token contract ID on the target network"
  value       = length(data.local_file.token_contract_id) > 0 ? trimspace(data.local_file.token_contract_id[0].content) : ""
}

output "escrow_contract_id" {
  description = "Deployed escrow contract ID on the target network"
  value       = length(data.local_file.escrow_contract_id) > 0 ? trimspace(data.local_file.escrow_contract_id[0].content) : ""
}
