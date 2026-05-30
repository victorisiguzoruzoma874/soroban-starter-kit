# Deployment Guide

Step-by-step instructions for deploying Soroban contracts and the frontend across all environments.

---

## Prerequisites

### Install Stellar CLI

The Stellar CLI is required to generate identities, fund accounts, and deploy contracts.

```bash
# Install Stellar CLI
cargo install --locked stellar-cli --features opt

# Verify installation
stellar --version
```

**Expected output:**
```
stellar 21.x.x
```

### Generate a Stellar CLI Identity

Create a new identity for contract deployment:

```bash
# Generate a new identity (interactive)
stellar keys generate --global deployer

# Or generate non-interactively
stellar keys generate deployer --network testnet
```

**Expected output:**
```
Created identity "deployer" with public key: GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

The secret key is stored securely in `~/.stellar/keys.yaml` (never commit this file).

### Fund Your Account on Testnet

Use Friendbot to fund your testnet account with 10,000 XLM:

```bash
# Fund the deployer account
stellar keys fund deployer --network testnet
```

**Expected output:**
```
Funded account GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX with 10000 XLM
```

Verify the account is funded:

```bash
stellar account info --source-account deployer --network testnet
```

**Expected output:**
```
Account ID: GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
Sequence: 0
Balances:
  10000.0000000 XLM
```

### Verify Stellar CLI Configuration

Check that your identity is properly configured:

```bash
stellar keys list
```

**Expected output:**
```
deployer (testnet)
```

---

## Environments

| Environment | Network | RPC URL |
|-------------|---------|---------|
| `local` | Standalone | `http://localhost:8000` |
| `testnet` | Test SDF Network | `https://soroban-testnet.stellar.org` |
| `mainnet` | Public Global Stellar | `https://soroban.stellar.org` |

---

## 1. Environment Setup

Copy and configure environment variables:

```bash
cp .env.example .env
```

### Local

```env
VITE_STELLAR_NETWORK=local
VITE_SOROBAN_RPC_URL=http://localhost:8000
VITE_HORIZON_URL=http://localhost:8001
VITE_NETWORK_PASSPHRASE="Standalone Network ; February 2017"
```

### Testnet

```env
VITE_STELLAR_NETWORK=testnet
VITE_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
VITE_HORIZON_URL=https://horizon-testnet.stellar.org
VITE_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
```

### Mainnet

```env
VITE_STELLAR_NETWORK=mainnet
VITE_SOROBAN_RPC_URL=https://soroban.stellar.org
VITE_HORIZON_URL=https://horizon.stellar.org
VITE_NETWORK_PASSPHRASE="Public Global Stellar Network ; September 2015"
```

---

## 2. Identity Setup

`scripts/deploy.sh` uses a Stellar CLI identity (key pair) as the transaction source account. Before deploying, ensure the identity exists on your machine.

### Check if the default identity exists

```bash
stellar keys show default
```

If the command fails with "not found", create it:

```bash
# Generate a new key pair and store it globally
stellar keys generate --global default

# Or import an existing secret key
stellar keys add default --secret-key
# (you will be prompted to enter the secret key)
```

### Using a custom identity

Pass `--identity <name>` to use a key other than `default`:

```bash
./scripts/deploy.sh testnet --identity my-deployer
./scripts/deploy.sh local token --identity alice
```

`deploy.sh` validates the identity before building any contracts and prints setup guidance if it is missing.

### Funding the identity (testnet)

```bash
stellar keys fund default --network testnet
```

---

## 3. Local Deployment

```bash
# Start local Stellar node
./scripts/local-net.sh start

# Build and deploy all contracts
./scripts/deploy.sh local

# Deploy a single contract
./scripts/deploy.sh local token

# Start frontend dev server
npm run dev
```

---

## 4. Testnet Deployment

### Step 1: Ensure Your Identity is Funded

Verify your deployer account has sufficient XLM:

```bash
stellar account info --source-account deployer --network testnet
```

If the account shows 0 XLM, fund it via Friendbot:

```bash
stellar keys fund deployer --network testnet
```

### Step 2: Deploy Contracts

Deploy all contracts:

```bash
./scripts/deploy.sh testnet
```

Deploy a single contract:

```bash
./scripts/deploy.sh testnet token
# or
./scripts/deploy.sh testnet escrow
```

**Expected output:**
```
Building token contract...
Deploying token contract to testnet...
Contract ID: CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
Admin: GXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

### Step 3: Save Contract IDs

Save the contract IDs printed to stdout — you'll need them in `.env`:

```env
VITE_TOKEN_CONTRACT_ID=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
VITE_ESCROW_CONTRACT_ID=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

### Step 4: Verify Deployment

Verify the contract is deployed and responsive:

```bash
stellar contract info --id <CONTRACT_ID> --network testnet
```

**Expected output:**
```
Contract ID: CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
WASM Hash: XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

---

## 3b. Post-Deploy Initialization

After deploying contracts, run the initialization script to call each contract's `initialize` function:

```bash
# Populate .contract-ids with deployed IDs (one per line: name=CONTRACT_ID)
echo "token=CABC..." >> .contract-ids
echo "escrow=CDEF..." >> .contract-ids

# Initialize all contracts on the current network
./scripts/initialize.sh testnet   # or local / mainnet
```

**Environment variable overrides:**

| Variable | Default | Description |
|----------|---------|-------------|
| `INIT_FN` | `initialize` | Function name to invoke |
| `SOURCE_ACCOUNT` | `default` | Stellar key to sign transactions |
| `CONTRACT_IDS_FILE` | `.contract-ids` | Path to contract ID list |
| `INIT_ARGS_<NAME>` | _(none)_ | Extra CLI args for a specific contract, e.g. `INIT_ARGS_TOKEN="--admin GABC..."` |

The script is idempotent: contracts that return an "already initialized" error are reported as skipped rather than failures.

---

## 5. Mainnet Deployment

> ⚠️ Mainnet deployments are irreversible. Complete testnet validation first.

```bash
# Ensure deployer account has sufficient XLM for fees
stellar keys generate --global deployer-mainnet

# Dry-run simulation (no broadcast)
stellar contract deploy --wasm <path>.wasm \
  --rpc-url https://soroban.stellar.org \
  --network-passphrase "Public Global Stellar Network ; September 2015" \
  --source-account deployer-mainnet \
  --simulate-only

# Deploy
./scripts/deploy.sh mainnet
```

---

## 6. Frontend Deployment

### Build

```bash
npm run build          # outputs to dist/
```

### Docker

```bash
docker build -f docker/Dockerfile --target prod -t soroban-kit:prod .
docker run -p 80:80 soroban-kit:prod
```

### Docker Compose (full stack)

```bash
docker compose -f docker/docker-compose.yml up --build
```

### Static Hosting (Netlify / Vercel / S3)

```bash
npm run build
# Upload dist/ to your static host
# Set environment variables in the host's dashboard (not in .env)
```

---

## 7. CI/CD Pipeline

The GitHub Actions workflow at `.github/workflows/ci.yml` runs on every push:

1. `npm ci` — install dependencies
2. `npm run lint` — ESLint checks
3. `npm test` — Vitest unit tests
4. `cargo test` — Rust contract tests
5. `stellar contract build` — WASM compilation check

To add automated deployment, extend `.github/workflows/ci.yml`:

```yaml
- name: Deploy to testnet
  if: github.ref == 'refs/heads/main'
  run: ./scripts/deploy.sh testnet
  env:
    STELLAR_SECRET_KEY: ${{ secrets.STELLAR_SECRET_KEY }}
```

---

## 8. Automated Guide Generation

Run the docs check script to validate documentation coverage and regenerate the docs report:

```bash
node scripts/check-docs.mjs
# Outputs: docs-report.json
```

To regenerate environment-specific config snippets automatically:

```bash
node scripts/generate-guides.mjs
# Outputs: docs/integration-guide.md, docs/deployment-guide.md
```

---

## 9. Validation Checklist

### Pre-deployment

- [ ] All contract tests pass: `cargo test`
- [ ] Frontend tests pass: `npm test`
- [ ] Lint passes: `npm run lint`
- [ ] `.env` is configured for the target network
- [ ] Deployer account is funded
- [ ] Contract IDs from previous deployments are recorded

### Post-deployment

- [ ] **Contract health verified** — run `./scripts/check-contract-ids.sh` (see §13)
- [ ] Contract responds to `simulate` calls
- [ ] Frontend connects to the correct RPC endpoint
- [ ] Admin functions are restricted to the correct address
- [ ] Event emission verified via Stellar Laboratory
- [ ] Contract ID saved to `.env` and committed (not the secret key)

---

## 10. Security Considerations

- Store `STELLAR_SECRET_KEY` only in CI secrets, never in `.env` committed to git
- Use separate deployer accounts per environment (local / testnet / mainnet)
- Verify WASM hash after deployment: `stellar contract info --id <CONTRACT_ID>`
- Enable `set_admin` only during initialization; lock it down immediately after
- Audit contract code with `cargo audit` before mainnet deployment:
  ```bash
  cargo install cargo-audit
  cargo audit
  ```
- Use `--simulate-only` to estimate fees before broadcasting

---

## 11. Performance Optimization

- Build contracts with `--release` flag (default in `deploy.sh`)
- Minimize contract storage reads — cache values in `Env::storage().instance()`
- Use `TTL` extensions for long-lived contract data to avoid expiry
- Profile WASM size: `wasm-opt -Oz input.wasm -o output.wasm`
- Frontend: set `Cache-Control: max-age=31536000` for hashed static assets

---

## 12. Contract Upgrades (Timelock)

Both the Token and Escrow contracts enforce a **two-step upgrade process** when
built with the `upgradeable` / `pausable` feature flags. A minimum delay of
`UPGRADE_DELAY_LEDGERS` (17 280 ledgers ≈ 24 hours at 5 s/ledger) is enforced
between proposing and executing a WASM upgrade.

### Step 1 — Propose the upgrade

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <ADMIN_KEY> \
  --network testnet \
  -- propose_upgrade \
  --wasm_hash <NEW_WASM_HASH>
```

This stores the hash and a `ready_after` ledger number on-chain and emits an
`upgrade_proposed` event. Announce the upgrade publicly so users have time to
review and exit if needed.

### Step 2 — Execute the upgrade (after the timelock)

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source-account <ADMIN_KEY> \
  --network testnet \
  -- execute_upgrade
```

The call will fail with `NotAuthorized` / `Unauthorized` if the current ledger
is still before `ready_after`. Once executed, an `upgrade_executed` event is
emitted and the WASM is replaced atomically.

### Verify the new WASM hash

```bash
stellar contract info --id <CONTRACT_ID> --network testnet
```

Compare the reported WASM hash against the expected value before announcing the
upgrade as complete.

### Security notes

- Never skip the timelock on mainnet. The delay gives users time to react.
- Gate `propose_upgrade` behind a multi-sig or governance vote for production.
- Rehearse the full upgrade flow on testnet before executing on mainnet.

---

## 13. Troubleshooting

### Identity and Funding Issues

| Problem | Solution |
|---------|----------|
| `stellar: command not found` | Install Stellar CLI: `cargo install --locked stellar-cli --features opt` |
| `Identity "deployer" not found` | Generate identity: `stellar keys generate --global deployer` |
| `Account not found` | Fund account via Friendbot: `stellar keys fund deployer --network testnet` |
| `Insufficient balance for fees` | Verify funding: `stellar account info --source-account deployer --network testnet` |
| `Friendbot rate limit exceeded` | Wait 5 minutes and retry, or use a different account |
| `Invalid network passphrase` | Ensure `.env` has correct `VITE_NETWORK_PASSPHRASE` for the target network |

### Deployment Issues

| Problem | Solution |
|---------|----------|
| `wasm32` target missing | `rustup target add wasm32-unknown-unknown` |
| Deploy fails: insufficient fee | Increase fee in `deploy.sh` or fund account |
| Local node unhealthy | `./scripts/local-net.sh reset` then retry |
| Contract already initialized | Deploy a fresh contract; initialization is one-time |
| Frontend shows wrong network | Check `VITE_STELLAR_NETWORK` in `.env` |
| CORS errors from RPC | Use a proxy or the official RPC endpoints |

---

## 13. Post-Deployment Contract Verification

After deploying, confirm that every contract in `.contract-ids` is alive:

```bash
./scripts/check-contract-ids.sh
```

The script reads `.contract-ids` (format: `name=<CONTRACT_ID>`), invokes `get_state`
on each contract, and categorises results:

| Status | Meaning |
|--------|---------|
| **ALIVE** | Contract responded normally |
| **EXPIRED TTL** | Entry has expired; extend TTL with `stellar contract extend` |
| **UNREACHABLE** | Contract not found or RPC error; re-deploy if necessary |

Override the default file or network:

```bash
./scripts/check-contract-ids.sh .contract-ids.testnet
STELLAR_NETWORK=mainnet ./scripts/check-contract-ids.sh
```

The script exits non-zero if any contract is expired or unreachable, making it
suitable for use in CI/CD pipelines.

---

## Resources

- [Stellar CLI Reference](https://developers.stellar.org/docs/tools/stellar-cli)
- [Soroban Deployment Docs](https://soroban.stellar.org/docs/getting-started/deploy-to-testnet)
- [Stellar Friendbot (testnet funding)](https://friendbot.stellar.org)
- [Stellar Laboratory](https://laboratory.stellar.org/)
- [cargo-audit](https://crates.io/crates/cargo-audit)
