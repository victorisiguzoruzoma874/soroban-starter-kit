#!/usr/bin/env bash
# Reads .contract-ids and checks whether each deployed contract is alive,
# expired (TTL), or unreachable.
set -euo pipefail

CONTRACT_IDS_FILE="${1:-.contract-ids}"
NETWORK="${STELLAR_NETWORK:-testnet}"

if [[ ! -f "$CONTRACT_IDS_FILE" ]]; then
  echo "ERROR: $CONTRACT_IDS_FILE not found." >&2
  exit 1
fi

alive=()
expired=()
unreachable=()

while IFS='=' read -r name contract_id || [[ -n "$name" ]]; do
  [[ "$name" =~ ^#.*$ || -z "$name" ]] && continue
  name="${name// /}"
  contract_id="${contract_id// /}"

  echo "Checking $name ($contract_id)..."

  output=$(stellar contract invoke \
    --id "$contract_id" \
    --network "$NETWORK" \
    -- get_state 2>&1 || true)

  if echo "$output" | grep -qi "ttl\|expired\|entry has expired"; then
    expired+=("$name ($contract_id)")
  elif echo "$output" | grep -qi "error\|not found\|unreachable\|failed"; then
    unreachable+=("$name ($contract_id)")
  else
    alive+=("$name ($contract_id)")
  fi
done < "$CONTRACT_IDS_FILE"

echo ""
echo "=== Contract Health Report ==="
echo ""
echo "ALIVE (${#alive[@]}):"
for c in "${alive[@]+"${alive[@]}"}"; do echo "  ✓ $c"; done

echo ""
echo "EXPIRED TTL (${#expired[@]}):"
for c in "${expired[@]+"${expired[@]}"}"; do echo "  ⚠ $c"; done

echo ""
echo "UNREACHABLE (${#unreachable[@]}):"
for c in "${unreachable[@]+"${unreachable[@]}"}"; do echo "  ✗ $c"; done

if [[ ${#expired[@]} -gt 0 || ${#unreachable[@]} -gt 0 ]]; then
  exit 1
fi
