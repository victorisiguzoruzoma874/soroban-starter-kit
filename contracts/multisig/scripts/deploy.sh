#!/bin/bash

# Soroban Multisig Contract Deployment Script
# Usage: ./deploy.sh [network] [source]
# Example: ./deploy.sh testnet alice

set -e

NETWORK=${1:-testnet}
SOURCE=${2:-}
CONTRACT_NAME="soroban_multisig_template"

echo "Deploying Multisig Contract to $NETWORK..."

echo "Building contract..."
stellar contract build

DEPLOY_ARGS=(contract deploy --wasm "target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm" --network "$NETWORK")
if [ -n "$SOURCE" ]; then
    DEPLOY_ARGS+=(--source "$SOURCE")
fi

echo "Deploying to $NETWORK..."
CONTRACT_ID=$(stellar "${DEPLOY_ARGS[@]}")

echo "Multisig contract deployed."
echo "Contract ID: $CONTRACT_ID"
echo "multisig: $CONTRACT_ID" >> ../../../.contract-ids

echo "Initialize with:"
echo "stellar contract invoke --id $CONTRACT_ID --network $NETWORK -- initialize --signers '[...]' --threshold 2"
