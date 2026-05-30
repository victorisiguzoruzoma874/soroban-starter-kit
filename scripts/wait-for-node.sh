#!/usr/bin/env bash
# wait-for-node.sh — poll until the local Stellar/Soroban RPC node is ready
# Usage: ./scripts/wait-for-node.sh [rpc-url] [max-wait-seconds]
set -euo pipefail

RPC_URL="${1:-http://localhost:${LOCAL_RPC_PORT:-8000}/soroban/rpc}"
MAX_WAIT="${2:-120}"
INTERVAL=3

echo "Waiting for Soroban RPC at $RPC_URL (timeout: ${MAX_WAIT}s) ..."
elapsed=0
while true; do
  if curl -sf -X POST \
       -H "Content-Type: application/json" \
       -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
       "$RPC_URL" 2>/dev/null | grep -q "healthy"; then
    echo "Node is ready (${elapsed}s)."
    exit 0
  fi
  if (( elapsed >= MAX_WAIT )); then
    echo "ERROR: Soroban RPC at $RPC_URL did not become ready within ${MAX_WAIT}s" >&2
    echo "  - Check that the container is running: docker compose ps" >&2
    echo "  - Inspect logs: docker compose logs stellar-node" >&2
    exit 1
  fi
  echo "  Not ready yet (${elapsed}s elapsed), retrying in ${INTERVAL}s ..."
  sleep "$INTERVAL"
  elapsed=$(( elapsed + INTERVAL ))
done
