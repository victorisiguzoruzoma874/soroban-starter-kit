#!/usr/bin/env bash
# Minimal end-to-end example using the Stellar CLI:
# deploys token, mints to buyer, runs full escrow lifecycle against a local node.
#
# Prerequisites:
#   stellar-cli installed (cargo install --locked stellar-cli --features opt)
#   ./scripts/local-net.sh start
#
# Usage:
#   ./examples/shell/run.sh
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-local}"
RPC_URL="${SOROBAN_RPC_URL:-http://localhost:8000/soroban/rpc}"
NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-Standalone Network ; February 2017}"

MINT_AMOUNT=1000000
ESCROW_AMOUNT=500000
DEADLINE=$(( $(date +%s) + 3600 ))

echo "=== Generating keypairs ==="
stellar keys generate admin   --network "$NETWORK" --overwrite
stellar keys generate buyer   --network "$NETWORK" --overwrite
stellar keys generate seller  --network "$NETWORK" --overwrite

ADMIN_KEY=$(stellar keys address admin)
BUYER_KEY=$(stellar keys address buyer)
SELLER_KEY=$(stellar keys address seller)

echo "Admin:  $ADMIN_KEY"
echo "Buyer:  $BUYER_KEY"
echo "Seller: $SELLER_KEY"

echo ""
echo "=== Funding accounts via friendbot ==="
curl -sf "http://localhost:8000/friendbot?addr=$ADMIN_KEY"  > /dev/null
curl -sf "http://localhost:8000/friendbot?addr=$BUYER_KEY"  > /dev/null
curl -sf "http://localhost:8000/friendbot?addr=$SELLER_KEY" > /dev/null
echo "Funded all accounts"

echo ""
echo "=== Building and deploying contracts ==="
stellar contract build --manifest-path contracts/token/Cargo.toml  2>/dev/null
stellar contract build --manifest-path contracts/escrow/Cargo.toml 2>/dev/null

TOKEN_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/token.wasm \
  --source admin \
  --network "$NETWORK")

ESCROW_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/escrow.wasm \
  --source admin \
  --network "$NETWORK")

echo "Token  contract: $TOKEN_ID"
echo "Escrow contract: $ESCROW_ID"

echo ""
echo "=== Initializing token contract ==="
stellar contract invoke --id "$TOKEN_ID" --source admin --network "$NETWORK" \
  -- initialize \
  --admin "$ADMIN_KEY" \
  --name "DemoToken" \
  --symbol "DEMO" \
  --decimal 7

echo ""
echo "=== Minting tokens to buyer ==="
stellar contract invoke --id "$TOKEN_ID" --source admin --network "$NETWORK" \
  -- mint \
  --to "$BUYER_KEY" \
  --amount "$MINT_AMOUNT"
echo "Minted $MINT_AMOUNT DEMO to buyer"

echo ""
echo "=== Creating escrow ==="
stellar contract invoke --id "$ESCROW_ID" --source buyer --network "$NETWORK" \
  -- create \
  --buyer  "$BUYER_KEY" \
  --seller "$SELLER_KEY" \
  --token  "$TOKEN_ID" \
  --amount "$ESCROW_AMOUNT" \
  --deadline "$DEADLINE"
echo "Escrow created"

echo ""
echo "=== Funding escrow ==="
stellar contract invoke --id "$ESCROW_ID" --source buyer --network "$NETWORK" \
  -- fund
echo "Escrow funded"

echo ""
echo "=== Marking delivery ==="
stellar contract invoke --id "$ESCROW_ID" --source seller --network "$NETWORK" \
  -- mark_delivery
echo "Delivery marked"

echo ""
echo "=== Releasing funds to seller ==="
stellar contract invoke --id "$ESCROW_ID" --source buyer --network "$NETWORK" \
  -- release
echo "Funds released"

echo ""
echo "Full escrow lifecycle complete."
