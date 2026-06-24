#!/bin/bash

# Soroban NFT Contract Deployment Script
# Usage: ./deploy.sh [network]
# Example: ./deploy.sh testnet

set -e

NETWORK=${1:-testnet}
CONTRACT_NAME="soroban-nft-template"

echo "Deploying NFT Contract to $NETWORK..."

echo "Building contract..."
stellar contract build

echo "Deploying to $NETWORK..."
CONTRACT_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm \
    --network $NETWORK)

echo "NFT contract deployed!"
echo "Contract ID: $CONTRACT_ID"

CONTRACT_IDS_FILE="../../../.contract-ids"
if [ ! -f "$CONTRACT_IDS_FILE" ]; then
    echo "{\"network\": \"$NETWORK\", \"contracts\": {\"nft\": \"$CONTRACT_ID\"}}" > "$CONTRACT_IDS_FILE"
else
    TEMP_FILE="${CONTRACT_IDS_FILE}.tmp"
    jq --arg network "$NETWORK" --arg contract_id "$CONTRACT_ID" \
        '.network = $network | .contracts.nft = $contract_id' \
        "$CONTRACT_IDS_FILE" > "$TEMP_FILE"
    mv "$TEMP_FILE" "$CONTRACT_IDS_FILE"
fi

# Example initialization (uncomment to use):
# ADMIN_ADDRESS="G..."
# COLLECTION_NAME="My NFT Collection"
# COLLECTION_SYMBOL="MNC"
# MAX_SUPPLY=10000  # set to 0 for no cap

# stellar contract invoke \
#     --id $CONTRACT_ID \
#     --network $NETWORK \
#     -- initialize \
#     --admin $ADMIN_ADDRESS \
#     --name "$COLLECTION_NAME" \
#     --symbol "$COLLECTION_SYMBOL" \
#     --max_supply $MAX_SUPPLY

# Example minting (uncomment to use):
# TOKEN_ID=1
# RECIPIENT_ADDRESS="G..."
# TOKEN_URI="ipfs://QmYourHashHere/1.json"

# stellar contract invoke \
#     --id $CONTRACT_ID \
#     --network $NETWORK \
#     -- mint \
#     --to $RECIPIENT_ADDRESS \
#     --token_id $TOKEN_ID \
#     --token_uri "$TOKEN_URI"

echo "NFT contract ready for use!"
echo "Save this Contract ID: $CONTRACT_ID"
