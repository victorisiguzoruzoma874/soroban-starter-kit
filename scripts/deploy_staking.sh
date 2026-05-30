#!/bin/bash

# Soroban Staking Contract Deployment Script
# Usage: ./deploy_staking.sh [network]
# Example: ./deploy_staking.sh testnet

set -e

NETWORK=${1:-testnet}
CONTRACT_NAME="soroban-staking-template"

echo "🚀 Deploying Staking Contract to $NETWORK..."

# Build the contract
echo "📦 Building contract..."
stellar contract build --manifest-path contracts/staking/Cargo.toml

# Deploy the contract
echo "🌐 Deploying to $NETWORK..."
CONTRACT_ID=$(stellar contract deploy \
    --wasm contracts/staking/target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm \
    --network "$NETWORK")

echo "✅ Staking contract deployed!"
echo "📋 Contract ID: $CONTRACT_ID"

# Save contract ID
echo "staking: $CONTRACT_ID" >> .contract-ids

# Example initialization (uncomment and fill in values to use)
# ADMIN_ADDRESS="G..."
# STAKE_TOKEN="C..."    # token users deposit to stake
# REWARD_TOKEN="C..."   # token distributed as rewards (can equal STAKE_TOKEN)

# stellar contract invoke \
#     --id "$CONTRACT_ID" \
#     --network "$NETWORK" \
#     --source "$ADMIN_ADDRESS" \
#     -- initialize \
#     --admin "$ADMIN_ADDRESS" \
#     --stake_token "$STAKE_TOKEN" \
#     --reward_token "$REWARD_TOKEN"

# Add rewards (admin only):
# REWARD_AMOUNT=1000000000  # in base units
# stellar contract invoke \
#     --id "$CONTRACT_ID" \
#     --network "$NETWORK" \
#     --source "$ADMIN_ADDRESS" \
#     -- add_rewards \
#     --amount "$REWARD_AMOUNT"

echo "🎉 Staking contract ready for use!"
echo "📝 Save this Contract ID: $CONTRACT_ID"
