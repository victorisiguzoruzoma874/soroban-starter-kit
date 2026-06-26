#!/usr/bin/env bash
# deploy.sh — build and deploy Soroban contracts
# Usage: ./scripts/deploy.sh [testnet|mainnet|local] [contract] [--identity <name>]
set -euo pipefail

check_prerequisites() {
  local missing=()
  for cmd in stellar cargo; do
    command -v "$cmd" &>/dev/null || missing+=("$cmd")
  done
  if [[ ${#missing[@]} -gt 0 ]]; then
    echo "Missing required tools: ${missing[*]}" >&2
    exit 1
  fi
}
check_prerequisites

WASM_OPT_AVAILABLE=false
if command -v wasm-opt &>/dev/null; then
  WASM_OPT_AVAILABLE=true
else
  echo "INFO: wasm-opt not found — skipping WASM size optimisation." >&2
  echo "      Install Binaryen (https://github.com/WebAssembly/binaryen) to enable." >&2
fi

optimize_wasm() {
  local wasm="$1"
  if [[ "$WASM_OPT_AVAILABLE" == "false" ]]; then
    return
  fi
  local before
  before=$(stat -c%s "$wasm" 2>/dev/null || stat -f%z "$wasm")
  wasm-opt -Oz --output "${wasm}.opt" "$wasm" && mv "${wasm}.opt" "$wasm"
  local after
  after=$(stat -c%s "$wasm" 2>/dev/null || stat -f%z "$wasm")
  local pct=$(( (before - after) * 100 / before ))
  echo "  wasm-opt: ${before}B → ${after}B (-${pct}%)"
}

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACTS_DIR="$ROOT/contracts"

[[ -d "$CONTRACTS_DIR" ]] || { echo "Contracts directory not found: $CONTRACTS_DIR"; exit 1; }

NETWORK="${1:-testnet}"
CONTRACT="${2:-all}"

# Parse --identity flag from remaining args
IDENTITY="default"
for i in "$@"; do
  if [[ "$i" == "--identity" ]]; then
    shift_next=true
  elif [[ "${shift_next:-false}" == "true" ]]; then
    IDENTITY="$i"
    shift_next=false
  fi
done

# Validate that the chosen identity exists
if ! stellar keys show "$IDENTITY" &>/dev/null 2>&1; then
  echo "ERROR: Stellar identity '$IDENTITY' not found." >&2
  echo "" >&2
  echo "To set up an identity, run one of the following:" >&2
  echo "  stellar keys generate --global $IDENTITY" >&2
  echo "  stellar keys add $IDENTITY --secret-key" >&2
  if [[ "$IDENTITY" == "default" ]]; then
    echo "" >&2
    echo "Or specify a different identity with: --identity <name>" >&2
    echo "  ./scripts/deploy.sh $NETWORK $CONTRACT --identity <your-key-name>" >&2
  fi
  exit 1
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

deploy_contract() {
  local name="$1"
  local dir="$CONTRACTS_DIR/$name"
  [[ -d "$dir" ]] || { echo "Contract not found: $name"; return 1; }

  echo "── Building $name ──"
  (cd "$dir" && stellar contract build)

  WASM=$(find "$dir/target/wasm32-unknown-unknown/release" -name "*.wasm" | head -1)
  [[ -n "$WASM" ]] || { echo "No WASM found for $name"; return 1; }

  optimize_wasm "$WASM"

  echo "── Deploying $name to $NETWORK ──"
  CONTRACT_ID=$(stellar contract deploy \
    --wasm "$WASM" \
    --rpc-url "$RPC_URL" \
    --network-passphrase "$PASSPHRASE" \
    --source-account "$IDENTITY" \
    --network "$NETWORK")
  echo "$name: $CONTRACT_ID" >> "$ROOT/.contract-ids"
  echo "Contract ID: $CONTRACT_ID"
}

if [[ "$CONTRACT" == "all" ]]; then
  for dir in "$CONTRACTS_DIR"/*/; do
    deploy_contract "$(basename "$dir")"
  done
else
  deploy_contract "$CONTRACT"
fi
