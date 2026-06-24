#!/usr/bin/env bash
# monitor-escrow.sh — display a human-readable escrow status summary
#
# Usage:
#   ./scripts/monitor-escrow.sh [network] <CONTRACT_ID>
#   ./scripts/monitor-escrow.sh testnet CABC...
#   ./scripts/monitor-escrow.sh local CABC...
#
# Environment overrides:
#   STELLAR_RPC_URL            Override the default RPC endpoint
#   STELLAR_NETWORK_PASSPHRASE Override the default network passphrase

set -euo pipefail

# ── color codes ──────────────────────────────────────────────────────────────
RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# ── argument parsing ─────────────────────────────────────────────────────────
if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 [network] <CONTRACT_ID>" >&2
  echo "  network: testnet (default) | mainnet | local" >&2
  exit 1
fi

if [[ $# -eq 2 ]]; then
  NETWORK="$1"
  CONTRACT_ID="$2"
else
  NETWORK="testnet"
  CONTRACT_ID="$1"
fi

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

# ── helpers ──────────────────────────────────────────────────────────────────
STELLAR_ARGS=(
  --id "$CONTRACT_ID"
  --rpc-url "$RPC_URL"
  --network-passphrase "$PASSPHRASE"
  --network "$NETWORK"
)

invoke() {
  stellar contract invoke "${STELLAR_ARGS[@]}" -- "$@" 2>/dev/null || echo "N/A"
}

# Strip surrounding quotes from stellar CLI output
strip_quotes() {
  echo "$1" | tr -d '"'
}

# Map raw state string to a display label and color
state_display() {
  local raw
  raw=$(strip_quotes "$1")
  case "$raw" in
    Created)    echo "${CYAN}Created${RESET}" ;;
    Funded)     echo "${YELLOW}Funded${RESET}" ;;
    Delivered)  echo "${YELLOW}Delivered${RESET}" ;;
    Disputed)   echo "${RED}Disputed${RESET}" ;;
    Completed)  echo "${GREEN}Completed${RESET}" ;;
    Refunded)   echo "${CYAN}Refunded${RESET}" ;;
    Cancelled)  echo "${RED}Cancelled${RESET}" ;;
    N/A)        echo "${RED}Not initialized${RESET}" ;;
    *)          echo "$raw" ;;
  esac
}

# Convert remaining ledgers to approximate human-readable time (5 s per ledger)
ledgers_to_time() {
  local ledgers="$1"
  if [[ "$ledgers" == "N/A" ]]; then
    echo "N/A"
    return
  fi
  # Remove possible leading minus
  local abs_ledgers="${ledgers#-}"
  local total_seconds=$(( abs_ledgers * 5 ))
  local days=$(( total_seconds / 86400 ))
  local hours=$(( (total_seconds % 86400) / 3600 ))
  local minutes=$(( (total_seconds % 3600) / 60 ))

  if [[ "$ledgers" -lt 0 ]]; then
    echo "${days}d ${hours}h ${minutes}m ago (OVERDUE)"
  elif [[ "$days" -gt 0 ]]; then
    echo "${days}d ${hours}h ${minutes}m remaining"
  elif [[ "$hours" -gt 0 ]]; then
    echo "${hours}h ${minutes}m remaining"
  else
    echo "${minutes}m remaining"
  fi
}

# ── fetch escrow data ────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}Escrow Monitor — ${CYAN}${CONTRACT_ID}${RESET}"
echo -e "Network: ${BOLD}${NETWORK}${RESET}  RPC: ${RPC_URL}"
echo "────────────────────────────────────────────────────"

STATE_RAW=$(invoke get_state)
REMAINING_RAW=$(invoke get_remaining_ledgers)
DEADLINE_PASSED_RAW=$(invoke is_deadline_passed)

# Try to get full info; individual fields are shown if available
BUYER=$(invoke get_escrow_info 2>/dev/null | grep -o '"buyer":"[^"]*"' | cut -d'"' -f4 || echo "N/A")
SELLER=$(invoke get_escrow_info 2>/dev/null | grep -o '"seller":"[^"]*"' | cut -d'"' -f4 || echo "N/A")
ARBITER=$(invoke get_escrow_info 2>/dev/null | grep -o '"arbiter":"[^"]*"' | cut -d'"' -f4 || echo "N/A")
AMOUNT=$(invoke get_escrow_info 2>/dev/null | grep -o '"amount":[0-9-]*' | cut -d':' -f2 || echo "N/A")
DEADLINE=$(invoke get_escrow_info 2>/dev/null | grep -o '"deadline":[0-9]*' | cut -d':' -f2 || echo "N/A")

STATE_DISPLAY=$(state_display "$STATE_RAW")
TIME_DISPLAY=$(ledgers_to_time "${REMAINING_RAW:-N/A}")

echo -e "State:          ${STATE_DISPLAY}"
echo ""
echo -e "Buyer:          ${BOLD}${BUYER}${RESET}"
echo -e "Seller:         ${BOLD}${SELLER}${RESET}"
echo -e "Arbiter:        ${BOLD}${ARBITER}${RESET}"
echo ""
echo -e "Amount:         ${BOLD}${AMOUNT}${RESET} (base units)"
echo -e "Deadline:       ledger ${BOLD}${DEADLINE}${RESET}"
echo -e "Time remaining: ${BOLD}${TIME_DISPLAY}${RESET}"

# Highlight overdue escrows
if [[ "${DEADLINE_PASSED_RAW:-false}" == "true" ]]; then
  STATE_CLEAN=$(strip_quotes "$STATE_RAW")
  if [[ "$STATE_CLEAN" == "Funded" || "$STATE_CLEAN" == "Delivered" || "$STATE_CLEAN" == "Disputed" ]]; then
    echo ""
    echo -e "${RED}${BOLD}WARNING: Deadline has passed. Buyer may now claim a refund.${RESET}"
  fi
fi

echo "────────────────────────────────────────────────────"
echo ""
