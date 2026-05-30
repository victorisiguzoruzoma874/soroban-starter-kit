#!/bin/bash

# Soroban Vesting Contract Deployment Script
# Usage: ./deploy_vesting.sh [network]
# Example: ./deploy_vesting.sh testnet

set -e

NETWORK=${1:-testnet}
CONTRACT_NAME="soroban-vesting-template"

echo "🚀 Deploying Vesting Contract to $NETWORK..."

# Build the contract
echo "📦 Building contract..."
stellar contract build --manifest-path contracts/vesting/Cargo.toml

# Deploy the contract
echo "🌐 Deploying to $NETWORK..."
CONTRACT_ID=$(stellar contract deploy \
    --wasm contracts/vesting/target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm \
    --network "$NETWORK")

echo "✅ Vesting contract deployed!"
echo "📋 Contract ID: $CONTRACT_ID"

# Save contract ID
echo "vesting: $CONTRACT_ID" >> .contract-ids

# Example initialization (uncomment and fill in values to use)
# ADMIN_ADDRESS="G..."
# BENEFICIARY_ADDRESS="G..."
# TOKEN_CONTRACT="C..."
# AMOUNT=1000000000   # total tokens to vest (in stroops / base units)
# CLIFF_LEDGER=500000  # ledger sequence at which vesting begins
# END_LEDGER=600000    # ledger sequence at which all tokens are fully vested

# stellar contract invoke \
#     --id "$CONTRACT_ID" \
#     --network "$NETWORK" \
#     --source "$ADMIN_ADDRESS" \
#     -- initialize \
#     --admin "$ADMIN_ADDRESS" \
#     --beneficiary "$BENEFICIARY_ADDRESS" \
#     --token "$TOKEN_CONTRACT" \
#     --cliff_ledger "$CLIFF_LEDGER" \
#     --end_ledger "$END_LEDGER" \
#     --amount "$AMOUNT"

echo "🎉 Vesting contract ready for use!"
echo "📝 Save this Contract ID: $CONTRACT_ID"
