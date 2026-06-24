#!/bin/bash

# Soroban Atomic Swap Contract Deployment Script
# Usage: ./deploy.sh [network]
# Example: ./deploy.sh testnet

set -e

NETWORK=${1:-testnet}
CONTRACT_NAME="soroban-swap-template"

echo "Deploying Swap Contract to $NETWORK..."

echo "Building contract..."
stellar contract build

echo "Deploying to $NETWORK..."
CONTRACT_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm \
    --network $NETWORK)

echo "Swap contract deployed!"
echo "Contract ID: $CONTRACT_ID"

CONTRACT_IDS_FILE="../../../.contract-ids"
if [ ! -f "$CONTRACT_IDS_FILE" ]; then
    echo "{\"network\": \"$NETWORK\", \"contracts\": {\"swap\": \"$CONTRACT_ID\"}}" > "$CONTRACT_IDS_FILE"
else
    TEMP_FILE="${CONTRACT_IDS_FILE}.tmp"
    jq --arg network "$NETWORK" --arg contract_id "$CONTRACT_ID" \
        '.network = $network | .contracts.swap = $contract_id' \
        "$CONTRACT_IDS_FILE" > "$TEMP_FILE"
    mv "$TEMP_FILE" "$CONTRACT_IDS_FILE"
fi

# Example propose_swap (uncomment to use):
# PARTY_A_ADDRESS="G..."
# TOKEN_A_ADDRESS="C..."
# AMOUNT_A=1000000000
# TOKEN_B_ADDRESS="C..."
# AMOUNT_B=500000000
# DEADLINE=1234567  # ledger sequence number

# stellar contract invoke \
#     --id $CONTRACT_ID \
#     --network $NETWORK \
#     -- propose_swap \
#     --party_a $PARTY_A_ADDRESS \
#     --token_a $TOKEN_A_ADDRESS \
#     --amount_a $AMOUNT_A \
#     --token_b $TOKEN_B_ADDRESS \
#     --amount_b $AMOUNT_B \
#     --deadline $DEADLINE

echo "Swap contract ready for use!"
echo "Save this Contract ID: $CONTRACT_ID"
