# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `cargo audit` security scanning job in CI workflow (#238)
- Error Reference section in README documenting all `TokenError` and `EscrowError` codes (#234)
- This CHANGELOG file (#231)
- Terraform provider version pinning and `.terraform` directory caching between plan and apply jobs (#242)

## [0.1.0] - 2026-04-24

Initial public release of the Soroban Starter Kit.

### Added

#### Token Contract (`contracts/token`)
- `initialize` — sets admin, name, symbol, decimals, and optional supply cap; guards against double-initialization
- `mint` — admin-only minting with overflow protection via `checked_add`
- `burn` / `burn_from` — self-service and allowance-based token burning
- `admin_burn` — admin-initiated burn from any address
- `transfer` / `transfer_from` — SEP-41 / `TokenInterface`-compliant transfers with allowance enforcement
- `approve` — time-bounded allowances stored in temporary storage; emits revocation event when amount is zero
- `balance` / `balance_of` — `balance` returns `0` for unknown addresses; `balance_of` returns `Option<i128>` to distinguish unknown from zero-balance addresses
- `total_supply` — returns current circulating supply
- `propose_admin` / `accept_admin` / `cancel_admin_transfer` — two-step admin handover to prevent accidental loss of admin access
- `set_admin` — single-step admin transfer kept for backwards compatibility (deprecated)
- `version` — returns the git commit hash baked in at compile time via `build.rs`
- `pausable` feature flag — adds `pause` / `unpause` entry points (admin only); blocks `mint`, `burn`, `transfer`, and `transfer_from` while paused
- `upgradeable` feature flag — adds `propose_upgrade` / `execute_upgrade` with a ~24-hour timelock (17 280 ledgers) before a WASM upgrade can be applied
- `capped-supply` feature flag — adds `max_supply` entry point and enforces a hard cap on `mint`
- Automatic TTL extension for instance and persistent storage entries

#### Escrow Contract (`contracts/escrow`)
- `initialize` — sets buyer, seller, arbiter, token contract, amount, and deadline; validates token address by calling `decimals()`; enforces distinct party addresses and a minimum deadline buffer
- `fund` — buyer transfers tokens to the contract, advancing state from `Created` to `Funded`
- `mark_delivered` — seller signals delivery, advancing state to `Delivered`
- `approve_delivery` — buyer releases escrowed funds to the seller
- `request_refund` — buyer reclaims funds after the deadline has passed
- `raise_dispute` — buyer or seller escalates to `Disputed` state
- `resolve_dispute` — arbiter resolves a dispute, releasing funds to either party
- `cancel` — buyer cancels an unfunded escrow (`Created` state only)
- `bump` — public TTL extension so any party can keep an active escrow alive
- `get_escrow_info` — returns full escrow details as an `EscrowInfo` struct
- `get_state` — returns `Option<EscrowState>` (returns `None` before initialization)
- `is_deadline_passed` — convenience predicate for deadline checks
- State machine: `Created → Funded → Delivered → Completed`, with exits to `Refunded` and `Cancelled`
- Checks-effects-interactions pattern enforced on all token transfer paths
- `pausable` feature flag — adds `pause` / `unpause` (admin only); blocks `fund`, `mark_delivered`, `approve_delivery`, `request_refund`, and `raise_dispute` while paused
- `upgradeable` feature flag — adds `propose_upgrade` / `execute_upgrade` with a ~24-hour timelock

#### Shared `common` Crate (`contracts/common`)
- `AdminKey` storage key enum for consistent admin address storage
- `get_admin` / `try_get_admin` — panic and `Option`-returning admin accessors
- `get_instance` — generic typed instance-storage getter
- `extend_ttl_instance` / `extend_ttl_persistent` — reusable TTL extension helpers

#### Testing
- Unit test suites for both contracts (8+ cases each) covering happy paths, error conditions, and edge cases
- Property-based tests via `proptest` for fuzz-style validation of token and escrow invariants
- Test snapshots under `test_snapshots/` for deterministic ledger state verification
- Integration test crate under `tests/`

#### CI / Tooling
- GitHub Actions workflow with test, build, and WASM artifact upload jobs
- `cargo audit` security scanning
- Benchmark suite (`benches/`) for escrow and token operations using `criterion`
- `build.rs` in each contract crate to embed `GIT_HASH` at compile time
- Docker Compose setup for a local Stellar node with Soroban RPC
- Dev container configuration (`.devcontainer/`) for reproducible development environments
- Deployment scripts (`scripts/deploy.sh`) for testnet and local network

#### Documentation
- Architecture Decision Records (ADRs) covering storage tiers, error handling, admin model, and escrow state machine
- `README.md` with quick-start guide, contract template table, and error reference
- `CONTRIBUTING.md` with dev setup, test instructions, code style, and PR process
- `SECURITY.md` with vulnerability disclosure policy

[Unreleased]: https://github.com/Fidelis900/soroban-starter-kit/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Fidelis900/soroban-starter-kit/releases/tag/v0.1.0
