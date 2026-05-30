#!/bin/bash

# Helper script to retrieve contract IDs from .contract-ids JSON file
# Usage: ./get-contract-id.sh [contract_name] [network]
# Example: ./get-contract-id.sh escrow testnet
# Example: ./get-contract-id.sh token

set -e

CONTRACT_NAME=${1:-escrow}
NETWORK=${2:-}

CONTRACT_IDS_FILE=".contract-ids"

if [ ! -f "$CONTRACT_IDS_FILE" ]; then
    echo "Error: $CONTRACT_IDS_FILE not found. Please run deploy.sh first." >&2
    exit 1
fi

# Get the contract ID using jq
if [ -z "$NETWORK" ]; then
    # Just get the contract ID without network filtering
    CONTRACT_ID=$(jq -r ".contracts.\"$CONTRACT_NAME\"" "$CONTRACT_IDS_FILE" 2>/dev/null)
else
    # Verify network matches and get contract ID
    STORED_NETWORK=$(jq -r ".network" "$CONTRACT_IDS_FILE" 2>/dev/null)
    if [ "$STORED_NETWORK" != "$NETWORK" ]; then
        echo "Error: Network mismatch. Expected $NETWORK but found $STORED_NETWORK in $CONTRACT_IDS_FILE" >&2
        exit 1
    fi
    CONTRACT_ID=$(jq -r ".contracts.\"$CONTRACT_NAME\"" "$CONTRACT_IDS_FILE" 2>/dev/null)
fi

if [ -z "$CONTRACT_ID" ] || [ "$CONTRACT_ID" = "null" ]; then
    echo "Error: Contract '$CONTRACT_NAME' not found in $CONTRACT_IDS_FILE" >&2
    exit 1
fi

echo "$CONTRACT_ID"
