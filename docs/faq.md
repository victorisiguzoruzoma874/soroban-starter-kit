# FAQ — Soroban Starter Kit

## Getting Started

### How do I add a new contract to this repository?

Follow the [Adding a Contract](adding-a-contract.md) guide. In brief:

1. Create a new directory under `contracts/your-contract/`
2. Generate a `Cargo.toml` with `stellar contract init`
3. Implement your contract logic in `src/lib.rs`
4. Add integration tests in `tests/` if using shared test infrastructure
5. Link it from the root `Cargo.toml` workspace

### How do I run tests for all contracts?

```bash
# Run all unit tests
cargo test

# Run tests for a specific contract
cargo test -p token

# Run tests with release optimizations
cargo test --release
```

Or use `just`:
```bash
just test
```

### How do I run tests for just one contract?

```bash
cd contracts/token
cargo test
```

## Deployment

### How do I deploy a contract to Stellar Testnet?

1. Export your keypair:
   ```bash
   export SOROBAN_SECRET_KEY="your-secret-key"
   ```

2. Set the target network:
   ```bash
   stellar network add --rpc-url https://soroban-testnet.stellar.org testnet
   stellar network use testnet
   ```

3. Deploy:
   ```bash
   cd contracts/token
   stellar contract deploy
   ```

Detailed steps are in [Deployment Guide](deployment-guide.md).

### How do I deploy to my local Soroban network?

Start a local network:
```bash
docker compose up stellar-node
```

Then deploy:
```bash
cd contracts/token
stellar contract deploy --network local
```

### Can I deploy multiple contracts at once?

Yes, use the `deploy.sh` script:
```bash
./scripts/deploy.sh testnet
```

Or deploy each individually:
```bash
cd contracts/token && stellar contract deploy --network testnet
cd contracts/escrow && stellar contract deploy --network testnet
```

## Feature Flags

### How do I enable feature flags for a contract?

Feature flags are configured in each contract's `Cargo.toml`. For example, the token contract has optional features:

```bash
# Build with a specific feature
cargo build -p token --features fee-token

# Build with multiple features
cargo build -p token --features fee-token,cap-supply
```

### What feature flags are available?

Check the `[features]` section of each contract's `Cargo.toml`. See [ADR-0005: Feature Flags](adr/0005-feature-flags.md) for the design rationale.

### Where are feature flags documented?

Each contract's source code and `Cargo.toml` defines available features. Integration tests demonstrate common feature combinations.

## Token Contract

### How do I customize the token contract?

1. **Change name/symbol**: Edit `initialize()` call with desired values
2. **Set supply cap**: Enable and use the `cap-supply` feature in `Cargo.toml`
3. **Add fee logic**: Enable and configure the `fee-token` feature
4. **Mint/burn controls**: The contract is admin-controlled; only the admin can mint or burn

See [Token Features](adr/0007-token-interface-compliance.md) for compliance details.

### How do I initialize a token?

```typescript
// Using Stellar.js
const txBuilder = new TransactionBuilder(...);
const call = new ContractInvoke({
  contractId: TOKEN_CONTRACT_ID,
  method: 'initialize',
  args: [
    new Address(admin),
    new U32(18),           // decimals
    nativeToScVal('MyToken', 'string'),
    nativeToScVal('MTK', 'string'),
  ],
});
```

Or use the shell script example in `examples/shell/run.sh`.

### How do I mint tokens?

Only the admin can mint. Call the `mint` function:

```rust
token::Client::new(&env, &token_contract_id)
    .mint(&recipient, &amount);
```

### How do I transfer tokens?

Any holder can transfer their balance:

```rust
token::Client::new(&env, &token_contract_id)
    .transfer(&from, &to, &amount);
```

## Common Issues

### Why is my deployment failing with "unauthorized"?

You are likely not the admin of the contract. Only the contract admin can invoke certain functions. Check who initialized the contract and use that keypair.

### Why is my contract out of money for fees?

Contracts on Testnet consume fees from the account that deployed them. Top up your account on [Stellar Lab](https://laboratory.stellar.org/) or use the Testnet Friendbot.

### How do I extend contract storage TTL?

Call the TTL bump function (varies by contract):

```rust
// For token contract
token::Client::new(&env, &token_contract_id).bump();
```

All contracts define `LEDGER_LIFETIME_THRESHOLD` and `LEDGER_BUMP_AMOUNT` constants for TTL strategy.

### Why does my contract state expire?

Soroban requires periodic TTL (time-to-live) extension. If no one calls a state-mutating function for ~7 days, the contract's persistent storage expires. Call a bump function to extend it.

## Errors

### Where can I find error codes and their meanings?

See [Error Reference](error-reference.md) for a complete list of error codes and what they mean for each contract.

### How do I handle errors in my integration?

Check the error code in the transaction result:

```typescript
try {
  await submitTx();
} catch (err) {
  const code = err.message; // e.g., "token.1" = InsufficientBalance
  console.error(`Error ${code}: check docs/error-reference.md`);
}
```

## Architecture & Design

### What storage strategy does the starter kit use?

Contracts use three storage tiers:
- **Instance storage**: Contract config, admin, global state
- **Persistent storage**: Per-user balances, allowances, history
- **Temporary storage**: Short-lived scratch data (not used extensively here)

See [ADR-0001: Storage Tier Choices](adr/0001-storage-tier-choices.md).

### Why are there so many contracts?

Each contract solves a specific DeFi primitive (token, escrow, staking, DAO, etc.). Pick the templates you need and customize them for your use case. They are designed to be independent and composable.

### Can I use these contracts in production?

These contracts are **audited templates**. Before mainnet deployment:
1. Conduct a security review with your team or auditors
2. Test thoroughly on Testnet
3. Verify the gas costs match your expectations (see [Gas Costs](gas-costs.md))
4. Consider coverage for your use case via insurance

## Development Workflow

### How do I set up my dev environment?

See [Dev Environment](dev-environment.md) for step-by-step setup. Quick start:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Stellar CLI
cargo install stellar-cli

# Clone and enter repo
git clone https://github.com/Fidelis900/soroban-starter-kit.git
cd soroban-starter-kit

# Run tests
cargo test
```

### Can I use VS Code with these contracts?

Yes. Install the [Rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension. The repo is pre-configured with `.vscode/settings.json`.

### How do I run benchmarks?

```bash
cargo bench
```

Detailed benchmark results are in `benches/`. This repository does not currently stress-test gas usage; that is tracked manually in [Gas Costs](gas-costs.md).

## Integration & Examples

### Where are the example integrations?

See `examples/`:
- `examples/typescript/index.js` — Node.js integration using Stellar.js
- `examples/shell/run.sh` — Equivalent shell-based example using Stellar CLI

Both demonstrate a token mint + escrow lifecycle.

### How do I integrate a contract into my own application?

1. Get the contract's WASM file or deploy an instance
2. Use [Stellar.js](https://github.com/stellar/js-stellar-sdk) or another SDK to invoke functions
3. See [Integration Guide](integration-guide.md) for detailed examples

### Can I compose multiple contracts?

Yes. Contracts can invoke other contracts. Pass contract IDs as arguments and use the SDK client to call them:

```rust
let token = token::Client::new(&env, &token_id);
token.transfer(&from, &to, &amount);
```

## Security

### Where is the security guidance?

See [Security Best Practices](security.md) for hardening, common pitfalls, and recommendations.

### Are these contracts audited?

The token and escrow contracts have undergone security review. Others are reference implementations. Before mainnet use, conduct your own audit or hire a professional auditor.

### What is the incident response process?

See [Incident Response](incident-response.md) for reporting security issues and the mitigation workflow.

## Contributing

### How do I contribute a new contract or improvement?

See [CONTRIBUTING.md](../CONTRIBUTING.md) for the full contribution workflow, code style, and PR process.

### What is the code style?

- **Formatting**: `cargo fmt`
- **Linting**: `cargo clippy`
- **Testing**: All public functions must have tests
- **Docs**: Public items must have doc comments with examples

Pre-commit hooks enforce these checks automatically.

## Support & Community

### Where can I get help?

- **Stellar Docs**: [soroban.stellar.org](https://soroban.stellar.org/docs)
- **Discord**: [Stellar Developer Discord](https://discord.gg/stellardev)
- **GitHub Issues**: [This repository](https://github.com/Fidelis900/soroban-starter-kit/issues)
- **Examples**: [stellar/soroban-examples](https://github.com/stellar/soroban-examples)

### How do I report a security issue?

Do **not** open a public issue. Email security concerns to the Stellar team or use the private security advisory feature on GitHub.

### Where is the roadmap?

The starter kit is a stable reference for common DeFi patterns. New contracts may be added based on community demand. Check [CHANGELOG.md](../CHANGELOG.md) for recent additions.
