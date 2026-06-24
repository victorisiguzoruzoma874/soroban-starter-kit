#!/bin/bash

# Soroban Timelock Contract Deployment Script
# Usage: ./deploy.sh [network]
# Example: ./deploy.sh testnet

set -e

NETWORK=${1:-testnet}
CONTRACT_NAME="soroban-timelock-template"

echo "Deploying Timelock Contract to $NETWORK..."

echo "Building contract..."
stellar contract build

echo "Deploying to $NETWORK..."
CONTRACT_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm \
    --network $NETWORK)

echo "Timelock contract deployed!"
echo "Contract ID: $CONTRACT_ID"

CONTRACT_IDS_FILE="../../../.contract-ids"
if [ ! -f "$CONTRACT_IDS_FILE" ]; then
    echo "{\"network\": \"$NETWORK\", \"contracts\": {\"timelock\": \"$CONTRACT_ID\"}}" > "$CONTRACT_IDS_FILE"
else
    TEMP_FILE="${CONTRACT_IDS_FILE}.tmp"
    jq --arg network "$NETWORK" --arg contract_id "$CONTRACT_ID" \
        '.network = $network | .contracts.timelock = $contract_id' \
        "$CONTRACT_IDS_FILE" > "$TEMP_FILE"
    mv "$TEMP_FILE" "$CONTRACT_IDS_FILE"
fi

# Example initialization (uncomment to use):
# ADMIN_ADDRESS="G..."
# TOKEN_ADDRESS="C..."
# BENEFICIARY_ADDRESS="G..."
# RELEASE_LEDGER=1234567        # target ledger sequence number
# AMOUNT=1000000000             # in token stroops

# stellar contract invoke \
#     --id $CONTRACT_ID \
#     --network $NETWORK \
#     -- initialize \
#     --admin $ADMIN_ADDRESS \
#     --token $TOKEN_ADDRESS \
#     --beneficiary $BENEFICIARY_ADDRESS \
#     --release_ledger $RELEASE_LEDGER \
#     --amount $AMOUNT

echo "Timelock contract ready for use!"
echo "Save this Contract ID: $CONTRACT_ID"
