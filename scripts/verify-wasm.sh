#!/usr/bin/env bash
# scripts/verify-wasm.sh — Verify WASM reproducibility between builds.
#
# Usage:
#   ./scripts/verify-wasm.sh <contract> <expected_sha256>
#
# Examples:
#   ./scripts/verify-wasm.sh token abc123...
#   ./scripts/verify-wasm.sh marketplace def456...
#
# The script builds the contract with `stellar contract build` and compares the
# resulting WASM hash against the expected SHA-256 supplied as the second argument.
# Exit code 0 = match (verified), non-zero = mismatch or error.

set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <contract> <expected_sha256>" >&2
  exit 1
fi

CONTRACT="$1"
EXPECTED="$2"
CONTRACT_DIR="contracts/${CONTRACT}"

if [ ! -d "${CONTRACT_DIR}" ]; then
  echo "ERROR: contract directory '${CONTRACT_DIR}' not found." >&2
  exit 1
fi

echo "==> Building ${CONTRACT}..."
stellar contract build --manifest-path "${CONTRACT_DIR}/Cargo.toml"

WASM_PATH="target/wasm32-unknown-unknown/release/${CONTRACT//-/_}.wasm"

# Also try hyphenated name in case the crate name differs.
if [ ! -f "${WASM_PATH}" ]; then
  WASM_PATH="target/wasm32-unknown-unknown/release/soroban_${CONTRACT//-/_}_template.wasm"
fi

if [ ! -f "${WASM_PATH}" ]; then
  # Fall back: find the newest wasm in the release directory.
  WASM_PATH="$(ls -t target/wasm32-unknown-unknown/release/*.wasm 2>/dev/null | head -n1)"
fi

if [ -z "${WASM_PATH}" ] || [ ! -f "${WASM_PATH}" ]; then
  echo "ERROR: could not locate WASM output for contract '${CONTRACT}'." >&2
  exit 1
fi

echo "==> WASM path: ${WASM_PATH}"

ACTUAL="$(sha256sum "${WASM_PATH}" | awk '{print $1}')"
echo "==> Actual   SHA-256: ${ACTUAL}"
echo "==> Expected SHA-256: ${EXPECTED}"

if [ "${ACTUAL}" = "${EXPECTED}" ]; then
  echo "✅ VERIFIED: hashes match."
  exit 0
else
  echo "❌ MISMATCH: WASM hash does not match expected value." >&2
  exit 1
fi
