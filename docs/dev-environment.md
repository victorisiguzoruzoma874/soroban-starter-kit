# Development Environment Setup

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Node.js | 20+ | https://nodejs.org |
| Docker | 24+ | https://docs.docker.com/get-docker/ |
| Rust | 1.78+ | https://rustup.rs |
| Stellar CLI | latest | `cargo install --locked stellar-cli --features opt` |
| just (optional) | latest | `cargo install just` |

### Installing `just`

`just` is a command runner popular in the Rust ecosystem with better cross-platform support than `make`:

```bash
# Via cargo (all platforms)
cargo install just

# Via homebrew (macOS/Linux)
brew install just

# Via winget (Windows)
winget install Casey.Just
```

Run `just --list` from the project root to see all available recipes.

---

## Quick Start (recommended)

```bash
git clone https://github.com/your-org/soroban-starter-kit.git
cd soroban-starter-kit
bash scripts/setup.sh        # installs deps, creates .env, adds git hook
npm run dev                  # http://localhost:3000
```

---

## Docker Compose

All services (frontend, contract builder, local Stellar node) are defined in `docker/docker-compose.yml`.

```bash
# Start everything
docker compose -f docker/docker-compose.yml up

# Frontend only
docker compose -f docker/docker-compose.yml up frontend

# Production build
docker build -f docker/Dockerfile --target prod -t fidelis:prod .
```

---

## Local Stellar Network

### Starting the node

```bash
# Start only the Stellar/Soroban node
docker compose -f docker/docker-compose.yml up stellar-node

# Or use the helper script (starts node and waits for healthy status)
./scripts/local-net.sh start    # start node, wait for healthy
./scripts/local-net.sh status   # check status
./scripts/local-net.sh reset    # wipe chain data and restart
./scripts/local-net.sh stop     # stop node
./scripts/local-net.sh logs     # tail logs
```

### Waiting for readiness

The node takes ~30 seconds to initialize. Use `scripts/wait-for-node.sh` to block until the Soroban RPC endpoint is accepting requests:

```bash
# Default: polls http://localhost:8000/soroban/rpc, waits up to 120s
./scripts/wait-for-node.sh

# Custom URL and timeout
./scripts/wait-for-node.sh http://localhost:8000/soroban/rpc 180
```

The script exits 0 when the node is healthy and 1 if it times out, making it safe to use in CI and deployment pipelines.

### Health check

The `stellar-node` service in `docker/docker-compose.yml` includes a built-in health check that queries `POST /soroban/rpc` with `getHealth`. Docker marks the container healthy only after the RPC responds with `"healthy"`, so dependent services (`frontend`, `contracts`) will not start until the node is ready.

Endpoints when running locally:

| Service | URL |
|---------|-----|
| Soroban RPC | http://localhost:8000/soroban/rpc |
| Horizon API | http://localhost:8001 |

Set `VITE_STELLAR_NETWORK=local` in `.env` to point the frontend at the local node.

---

## Contract Deployment

```bash
./scripts/deploy.sh testnet          # deploy all contracts to testnet
./scripts/deploy.sh testnet token    # deploy only the token contract
./scripts/deploy.sh local            # deploy to local node
```

---

## Dev Container (VS Code / GitHub Codespaces)

Open the repo in VS Code and click **"Reopen in Container"** when prompted, or run:

```
Dev Containers: Reopen in Container
```

The container automatically runs `scripts/setup.sh` on creation and forwards ports 3000, 8000, and 8001.

---

## Environment Variables

Copy `.env.example` to `.env` and fill in values:

```bash
cp .env.example .env
```

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_STELLAR_NETWORK` | `testnet` | `testnet`, `mainnet`, or `local` |
| `VITE_SOROBAN_RPC_URL` | testnet RPC | Soroban RPC endpoint |
| `VITE_HORIZON_URL` | testnet Horizon | Horizon REST endpoint |
| `VITE_NETWORK_PASSPHRASE` | testnet passphrase | Network passphrase |
| `VITE_VAPID_PUBLIC_KEY` | _(empty)_ | VAPID key for push notifications |
| `LOCAL_RPC_PORT` | `8000` | Local node RPC port |
| `LOCAL_HORIZON_PORT` | `8001` | Local node Horizon port |

---

## Troubleshooting

**`npm ci` fails**
- Ensure Node.js 20+: `node --version`
- Delete `node_modules/` and retry

**Docker Compose port conflict**
- Change `LOCAL_RPC_PORT` / `LOCAL_HORIZON_PORT` in `.env`

**`stellar` command not found**
- Run: `cargo install --locked stellar-cli --features opt`
- Ensure `~/.cargo/bin` is in `$PATH`

**Local node never becomes healthy**
- Check Docker has enough memory (≥4 GB recommended)
- Run `./scripts/local-net.sh logs` to inspect errors

**Contract build fails (`wasm32` target missing)**
```bash
rustup target add wasm32-unknown-unknown
```

**Freighter wallet not connecting**
- Install the [Freighter extension](https://freighter.app)
- Switch Freighter to the matching network (Testnet / Mainnet)

---

## Fuzz Testing

Fuzz testing helps discover edge cases and potential vulnerabilities in contract code.

### Running Fuzz Tests

```bash
# Install cargo-fuzz if not already installed
cargo install cargo-fuzz

# Run fuzz target for token contract
cd fuzz
cargo fuzz run token_fuzz

# Run with a specific number of iterations
cargo fuzz run token_fuzz -- -max_len=1024 -runs=10000

# Run with a corpus directory
cargo fuzz run token_fuzz -- corpus/
```

### Fuzz Targets

- **token_fuzz**: Exercises token contract operations (mint, burn, transfer, approve, transfer_from, balance) with arbitrary inputs

### Interpreting Results

- Fuzz tests generate random inputs and monitor for crashes, panics, or undefined behavior
- Crashes are saved to `fuzz/artifacts/` for reproduction
- Use `RUST_BACKTRACE=1` for detailed crash information

```bash
RUST_BACKTRACE=1 cargo fuzz run token_fuzz -- fuzz/artifacts/token_fuzz/crash-*
```
## Secrets Management

### Golden rules

- **Never commit `.env`** — it is listed in `.gitignore`. If you accidentally stage it, run `git reset HEAD .env`.
- **Never commit real keys, mnemonics, or tokens** — not even in comments or test fixtures.
- **`.env.example` is the source of truth** for which variables exist. It must contain only placeholder values (e.g. `SXXX…`, `your-api-key-here`).
- **Rotate immediately** if a secret is ever pushed to a remote branch. Treat the key as permanently compromised regardless of whether the commit was later removed.

### Local setup

```bash
cp .env.example .env   # create your local config from the template
# edit .env and fill in real values — this file is gitignored
```

### Sharing configuration between developers

Share the *shape* of config (variable names, descriptions) via `.env.example`. Share actual values through a secrets manager (e.g. AWS Secrets Manager, 1Password, Doppler) or an encrypted channel — never via chat, email, or a repository.

### Pre-commit hook

A sample hook lives at `.githooks/pre-commit`. Enable it once per clone:

```bash
git config core.hooksPath .githooks
chmod +x .githooks/pre-commit
```

The hook blocks commits that contain common secret patterns (Stellar secret keys, mnemonics, bearer tokens, API keys) or that stage a `.env` file directly.
