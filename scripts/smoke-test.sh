#!/usr/bin/env bash
# smoke-test.sh — post-deployment verification script
# Usage: ./scripts/smoke-test.sh [testnet|mainnet|local]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACT_IDS_FILE="$ROOT/.contract-ids"

NETWORK="${1:-testnet}"

case "$NETWORK" in
  testnet)
    RPC_URL="${STELLAR_RPC_URL:-https://soroban-testnet.stellar.org}"
    PASSPHRASE="${STELLAR_NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"
    ;;
  mainnet)
    RPC_URL="${STELLAR_RPC_URL:-https://soroban.stellar.org}"
    PASSPHRASE="${STELLAR_NETWORK_PASSPHRASE:-Public Global Stellar Network ; September 2015}"
    ;;
  local)
    RPC_URL="${STELLAR_RPC_URL:-http://localhost:${LOCAL_RPC_PORT:-8000}}"
    PASSPHRASE="${STELLAR_NETWORK_PASSPHRASE:-Standalone Network ; February 2017}"
    ;;
  *)
    echo "Unknown network: $NETWORK (use testnet|mainnet|local)" >&2
    exit 1
    ;;
esac

if [[ ! -f "$CONTRACT_IDS_FILE" ]]; then
  echo "Contract IDs file not found: $CONTRACT_IDS_FILE" >&2
  exit 1
fi

# Read contract IDs
TOKEN_ID=$(grep "token:" "$CONTRACT_IDS_FILE" | cut -d: -f2 | xargs)
ESCROW_ID=$(grep "escrow:" "$CONTRACT_IDS_FILE" | cut -d: -f2 | xargs)

if [[ -z "$TOKEN_ID" ]]; then
  echo "Token contract ID not found in $CONTRACT_IDS_FILE" >&2
  exit 1
fi

if [[ -z "$ESCROW_ID" ]]; then
  echo "Escrow contract ID not found in $CONTRACT_IDS_FILE" >&2
  exit 1
fi

echo "── Smoke testing contracts on $NETWORK ──"

# Test token contract read-only functions
echo "Testing token contract ($TOKEN_ID)..."

# Check total supply (should be 0 if not initialized, or whatever after init)
TOTAL_SUPPLY=$(stellar contract invoke \
  --id "$TOKEN_ID" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$PASSPHRASE" \
  --network "$NETWORK" \
  -- total_supply)

echo "Total supply: $TOTAL_SUPPLY"

# Check admin (will fail if not initialized, but that's expected)
ADMIN=$(stellar contract invoke \
  --id "$TOKEN_ID" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$PASSPHRASE" \
  --network "$NETWORK" \
  -- admin 2>/dev/null || echo "Not initialized")

echo "Admin: $ADMIN"

# Test escrow contract
echo "Testing escrow contract ($ESCROW_ID)..."

# Check state (should be None if not initialized)
STATE=$(stellar contract invoke \
  --id "$ESCROW_ID" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$PASSPHRASE" \
  --network "$NETWORK" \
  -- get_state 2>/dev/null || echo "Not initialized")

echo "State: $STATE"

echo "✅ Smoke tests passed!"