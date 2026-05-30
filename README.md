# Soroban Contract Templates

[![CI](https://github.com/Fidelis900/soroban-starter-kit/actions/workflows/ci.yml/badge.svg)](https://github.com/Fidelis900/soroban-starter-kit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Fidelis900/soroban-starter-kit/branch/main/graph/badge.svg)](https://codecov.io/gh/Fidelis900/soroban-starter-kit)

A curated collection of production-ready Soroban smart contract templates. These templates help developers quickly bootstrap common use cases on Soroban (Stellar's smart contract platform) for DeFi, payments, governance, and more.

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/your-username/soroban-contract-templates.git
cd soroban-contract-templates

# Build a contract from the repo root (example: token)
stellar contract build --manifest-path contracts/token/Cargo.toml
# Alternatively: cd contracts/token && stellar contract build

# Deploy to testnet
./scripts/deploy.sh testnet

# Run tests
cargo test
```

## 📦 Contract Templates

| Template | Description | Use Cases | Status |
|----------|-------------|-----------|---------|
| **Token** | Custom fungible token with mint/burn/admin controls | DeFi tokens, governance tokens, utility tokens | ✅ Complete |
| **Escrow** | Two-party escrow with timeout and refund mechanism | P2P trading, service payments, milestone payments | ✅ Complete |
| **Vesting** | Token vesting with cliff + linear release schedule | Team allocations, investor lockups, employee grants | ✅ Complete |
| **Staking** | Token staking with proportional reward distribution | DeFi yield, protocol incentives, liquidity mining | ✅ Complete |
| **Multisig** | N-of-M wallet for threshold-approved contract calls | DAO treasuries, team wallets, shared administration | ✅ Complete |

### Token Contract Features
- **Standard Interface**: Full Soroban token compatibility
- **Administrative Controls**: Mint, burn, and admin management
- **Metadata Support**: Name, symbol, and decimals
- **Allowance System**: Approve and transfer_from functionality
- **Event Emission**: All operations emit events for tracking
- **Error Handling**: Custom error types for better debugging

### Escrow Contract Features
- **Two-Party Security**: Secure buyer-seller transactions
- **Deadline Protection**: Automatic refunds after deadline
- **Arbiter Support**: Third-party dispute resolution
- **State Management**: Clear transaction lifecycle
- **Token Agnostic**: Works with any Soroban token
- **Event Emission**: All operations emit events for tracking

### Vesting Contract Features
- **Cliff + Linear Schedule**: Tokens unlock linearly between `cliff_ledger` and `end_ledger`
- **Admin Revocation**: Admin can cancel unvested tokens at any time; vested tokens remain claimable
- **Incremental Claims**: Beneficiary claims accrued tokens on demand
- **Token Agnostic**: Works with any Soroban-compatible token
- **Event Emission**: `initialized`, `claimed`, and `revoked` events for off-chain tracking
- **TTL Management**: Instance storage TTL is extended on every interaction

### Staking Contract Features
- **Proportional Rewards**: Rewards distributed pro-rata to each staker's share of the pool
- **Reward-Per-Token Accumulator**: Gas-efficient global accumulator pattern; no per-staker loops
- **Separate Stake / Reward Tokens**: Stake token and reward token can be the same or different
- **Admin Reward Deposits**: Admin calls `add_rewards` to top up the reward pool at any time
- **Incremental Claims**: Stakers call `claim_rewards` independently; rewards accrue continuously
- **Token Agnostic**: Works with any Soroban-compatible token
- **Event Emission**: `staked`, `unstaked`, `rewards_claimed`, and `rewards_added` events
- **TTL Management**: Instance storage TTL is extended on every interaction
### Multisig Contract Features
- **N-of-M Authorization**: Configure any valid threshold across unique signers
- **Signer Management**: Add or remove signers with threshold-approved changes
- **Transaction Proposals**: Store target contract, function, and arguments
- **Signature Tracking**: Prevent duplicate signatures and non-signer approvals
- **Threshold Execution**: Execute proposed calls only after enough signatures
- **Event Emission**: Initialization, signer changes, signatures, and execution emit events

Each template includes:
- ✅ Complete contract implementation
- ✅ Comprehensive unit tests (8+ test cases each)
- ✅ Deployment scripts with examples
- ✅ Usage examples and documentation

## 🛠 Prerequisites

- [Rust](https://rustup.rs/) **1.82.0** (pinned via `rust-toolchain.toml` — `rustup` picks this up automatically)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup#install-the-soroban-cli)
- [Docker](https://www.docker.com/) (for local Stellar node)

> **Zero-install option:** Open this repo in a pre-configured environment with all tools ready — see the [Dev Container & Codespaces Guide](docs/devcontainer.md).

## 📖 Usage

### Building Contracts

```bash
cd contracts/[template-name]
stellar contract build
```

### Running Tests

```bash
cd contracts/[template-name]
cargo test
```

### Deploying to Testnet

```bash
cd contracts/[template-name]
./scripts/deploy.sh testnet
```

### Local Development

Start a local Stellar node with Soroban RPC:

```bash
docker compose up stellar-node
```

## ⚠️ Error Reference

> For full details — causes, triggers, and resolution steps — see [docs/error-reference.md](docs/error-reference.md).

### Token Contract Errors (`TokenError`)

| Code | Name | Description |
|------|------|-------------|
| 1 | `InsufficientBalance` | Caller's balance is too low to complete the transfer or burn |
| 2 | `InsufficientAllowance` | Approved allowance is too low for the requested `transfer_from` amount |
| 3 | `Unauthorized` | Caller is not the admin or does not have permission for this operation |
| 4 | `AlreadyInitialized` | `initialize` was called on a contract that has already been set up |
| 5 | `NotInitialized` | An operation was attempted before the contract was initialized |
| 6 | `InvalidAmount` | Amount is zero, negative, or exceeds the configured max supply |
| 7 | `Overflow` | Arithmetic overflow occurred during a balance or supply calculation |

### Escrow Contract Errors (`EscrowError`)

| Code | Name | Description |
|------|------|-------------|
| 1 | `NotAuthorized` | Caller is not permitted to invoke this function (wrong party or arbiter) |
| 2 | `InvalidState` | The escrow is not in the required state for this operation |
| 3 | `DeadlinePassed` | The escrow deadline has already elapsed; the operation is no longer valid |
| 4 | `DeadlineNotReached` | The deadline has not yet passed; premature refund or timeout claim attempted |
| 5 | `AlreadyInitialized` | `initialize` was called on an escrow that is already set up |
| 6 | `NotInitialized` | An operation was attempted before the escrow was initialized |
| 7 | `InsufficientFunds` | The buyer's token balance is too low to cover the escrowed amount |
| 8 | `InvalidAmount` | The specified amount is zero or otherwise invalid |
| 9 | `InvalidParties` | Buyer, seller, or arbiter addresses are invalid or conflict with each other |

### Vesting Contract Errors (`VestingError`)

| Code | Name | Description |
|------|------|-------------|
| 1 | `AlreadyInitialized` | `initialize` was called on a contract that is already set up |
| 2 | `NotInitialized` | An operation was attempted before the contract was initialized |
| 3 | `Unauthorized` | Caller is not the admin |
| 4 | `InvalidAmount` | The vesting amount is zero or negative |
| 5 | `InvalidSchedule` | `cliff_ledger` >= `end_ledger`, or `end_ledger` is in the past |
| 6 | `NothingToClaim` | No tokens have vested since the last claim (or vested amount is zero) |
| 7 | `AlreadyRevoked` | `revoke` was called on a schedule that has already been revoked |

### Staking Contract Errors (`StakingError`)

| Code | Name | Description |
|------|------|-------------|
| 1 | `AlreadyInitialized` | `initialize` was called on a contract that is already set up |
| 2 | `NotInitialized` | An operation was attempted before the contract was initialized |
| 3 | `Unauthorized` | Caller is not the admin |
| 4 | `InvalidAmount` | Amount is zero or negative |
| 5 | `NoStake` | Staker has no stake to unstake or claim from |
| 6 | `InsufficientStake` | Requested unstake amount exceeds the staker's current stake |
| 7 | `NoRewards` | No rewards are available to claim |
### Multisig Contract Errors (`MultisigError`)

| Code | Name | Description |
|------|------|-------------|
| 1 | `AlreadyInitialized` | `initialize` was called after the signer set was already configured |
| 2 | `NotInitialized` | An operation was attempted before the multisig was initialized |
| 3 | `InvalidThreshold` | Threshold is zero or greater than the number of signers |
| 4 | `InvalidSigners` | Signer or approval lists are empty or contain duplicates |
| 5 | `NotSigner` | Caller, approver, or signer is not part of the wallet signer set |
| 6 | `TransactionNotFound` | Requested transaction ID does not exist |
| 7 | `AlreadyExecuted` | Transaction has already been executed |
| 8 | `AlreadySigned` | Signer already approved the transaction |
| 9 | `ThresholdNotMet` | Transaction does not have enough signatures to execute |
| 10 | `InsufficientApprovals` | Signer-management change lacks enough threshold approvals |
## 📂 Examples

End-to-end working examples are provided in the `examples/` directory:

| Example | Description |
|---------|-------------|
| [`examples/typescript/index.js`](examples/typescript/index.js) | Node.js script — deploys token, mints to buyer, runs full escrow lifecycle |
| [`examples/shell/run.sh`](examples/shell/run.sh) | Equivalent shell script using the Stellar CLI |

Both examples target a local Stellar node. Start one with `./scripts/local-net.sh start` before running.

### TypeScript

```bash
npm install @stellar/stellar-sdk
TOKEN_CONTRACT_ID=<id> ESCROW_CONTRACT_ID=<id> node examples/typescript/index.js
```

### Shell

```bash
./examples/shell/run.sh
```

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for dev setup, test commands, code style, and the PR process.

## 📚 Resources

- [System Architecture](docs/architecture.md) — High-level design, contract relationships, storage tiers, event model, and admin framework
- [Security Best Practices](docs/security.md)
- [Integration Guide](docs/integration-guide.md)
- [Deployment Guide](docs/deployment-guide.md)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Developer Discord](https://discord.gg/stellardev)
- [Soroban Examples](https://github.com/stellar/soroban-examples)
- [Freighter Wallet](https://freighter.app/)
- [Stellar Laboratory](https://laboratory.stellar.org/)
- [Security Best Practices](docs/security.md)
 - [Architecture Decision Records](docs/adr/README.md)

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

---

**Ready to build on Soroban?** Start with any template and customize it for your use case! 🚀
