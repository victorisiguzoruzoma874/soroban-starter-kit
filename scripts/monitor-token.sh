#!/usr/bin/env bash
# monitor-token.sh — Display token contract status
# Usage: ./scripts/monitor-token.sh [testnet|mainnet|local]
set -euo pipefail

NETWORK="${1:-testnet}"
CONTRACT_ID="${TOKEN_CONTRACT_ID:?Set TOKEN_CONTRACT_ID env var}"

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

invoke() {
  stellar contract invoke \
    --id "$CONTRACT_ID" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$PASSPHRASE" \
    --network "$NETWORK" \
    -- "$@" 2>/dev/null || echo "n/a"
}

ADMIN=$(invoke admin)
TOTAL_SUPPLY=$(invoke total_supply)
MAX_SUPPLY=$(invoke max_supply 2>/dev/null || echo "uncapped")
VERSION=$(invoke version)

# Paused state: attempt to call pause (read-only simulate); if the contract
# responds with "already paused" or similar the flag is set; otherwise assume
# not paused. We simply report "n/a" for contracts without the pausable feature.
PAUSED=$(stellar contract invoke \
  --id "$CONTRACT_ID" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$PASSPHRASE" \
  --network "$NETWORK" \
  --simulate-only \
  -- pause 2>&1 | grep -qi '"true"' && echo "true" || \
  stellar contract invoke \
    --id "$CONTRACT_ID" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$PASSPHRASE" \
    --network "$NETWORK" \
    --simulate-only \
    -- unpause 2>&1 | grep -qi 'NotAuthorized\|error' && echo "false" || echo "n/a")

echo "=============================="
echo " Token Contract Monitor"
echo " Network : $NETWORK"
echo " Contract: $CONTRACT_ID"
echo "=============================="
printf "%-14s %s\n" "Admin:"        "$ADMIN"
printf "%-14s %s\n" "Total Supply:" "$TOTAL_SUPPLY"
printf "%-14s %s\n" "Max Supply:"   "$MAX_SUPPLY"
printf "%-14s %s\n" "Paused:"       "$PAUSED"
printf "%-14s %s\n" "Version:"      "$VERSION"
echo "=============================="
