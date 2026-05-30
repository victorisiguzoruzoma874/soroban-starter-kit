# infra/terraform

Terraform configuration for automated Soroban contract deployment to Stellar networks.

## Overview

This directory provisions:

- A Stellar admin account (via an existing secret key)
- Testnet Friendbot funding (testnet only)
- Token and escrow contract deployments
- Token contract initialisation

Because no official Terraform provider exists for Stellar, resources use `null_resource` + `local-exec` to invoke the [Stellar CLI](https://github.com/stellar/stellar-cli).

## Prerequisites

| Tool | Minimum version |
|---|---|
| Terraform | 1.5 |
| Stellar CLI (`stellar`) | 21.x |
| Bash | 4.x |

## Variables

| Name | Description | Default |
|---|---|---|
| `environment` | Deployment environment (`testnet` \| `mainnet`) | `testnet` |
| `network` | Stellar network (`testnet` \| `mainnet` \| `local`) | `testnet` |
| `network_passphrase` | Stellar network passphrase | testnet passphrase |
| `admin_key_path` | Path to the admin secret-key file | `~/.config/stellar/admin.key` |
| `token_name` | Token display name | `Soroban Token` |
| `token_symbol` | Token ticker symbol | `STK` |
| `cloud` | Backend cloud provider (`aws` \| `gcp` \| `azure`) | `aws` |

## Outputs

| Name | Description |
|---|---|
| `environment` | Active deployment environment |
| `network` | Stellar network used |
| `admin_address` | Public address of the admin account |
| `token_contract_id` | Deployed token contract ID |
| `escrow_contract_id` | Deployed escrow contract ID |

## Usage

### 1. Prepare the admin key

```bash
# Generate a new keypair (save the secret key securely)
stellar keys generate admin
stellar keys show admin  # note the secret key

# Write only the secret key to a file
echo "<secret-key>" > ~/.config/stellar/admin.key
chmod 600 ~/.config/stellar/admin.key
```

### 2. Build contracts

Contracts must be compiled to WASM before running `terraform apply`:

```bash
cd /path/to/soroban-starter-kit
for dir in contracts/*/; do
  stellar contract build --manifest-path "$dir/Cargo.toml"
done
```

### 3. Initialise Terraform

```bash
cd infra/terraform

# Local plan without remote backend
terraform init -backend=false

# With S3 backend (CI)
terraform init \
  -backend-config="bucket=<bucket>" \
  -backend-config="key=soroban-starter-kit/testnet/terraform.tfstate" \
  -backend-config="region=us-east-1" \
  -backend-config="dynamodb_table=<lock-table>"
```

### 4. Plan and apply

```bash
terraform plan \
  -var="network=testnet" \
  -var="environment=testnet" \
  -var="network_passphrase=Test SDF Network ; September 2015"

terraform apply
```

### 5. View outputs

```bash
terraform output -json
```

## CI/CD

The `infra.yml` workflow handles plan/apply/destroy via GitHub Actions. Required secrets:

| Secret | Purpose |
|---|---|
| `TF_STATE_BUCKET` | S3 bucket for remote state |
| `TF_LOCK_TABLE` | DynamoDB table for state locking |
| `AWS_ROLE_ARN` | IAM role for OIDC authentication |

## State files

The following files are written to `infra/terraform/` at apply time and are excluded from version control via `.gitignore`:

- `.admin_address` — admin public key
- `.token_contract_id` — deployed token contract ID
- `.escrow_contract_id` — deployed escrow contract ID

Add these patterns to `infra/terraform/.gitignore` if not already present:

```
.admin_address
.token_contract_id
.escrow_contract_id
.terraform/
*.tfplan
```
