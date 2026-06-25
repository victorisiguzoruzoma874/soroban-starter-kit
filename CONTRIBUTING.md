# Contributing

## Pre-commit hooks

This project ships a `.pre-commit-config.yaml` that runs `cargo fmt` and `cargo clippy` before every commit so you catch issues locally instead of in CI.

### Setup

```bash
pip install pre-commit          # or: brew install pre-commit
pre-commit install              # wire the hook into .git/hooks/pre-commit
```

From now on, every `git commit` will automatically run:

- **`cargo fmt --check`** — rejects commits with unformatted Rust code. Run `cargo fmt` to fix.
- **`cargo clippy`** — rejects commits that introduce Clippy warnings treated as errors.

To run the hooks manually without committing:

```bash
pre-commit run --all-files
```

---

## CI checks

### cargo-machete (unused dependencies)

The `machete` CI job runs `cargo machete --workspace` to detect unused entries in `Cargo.toml`.
If the job fails, remove the flagged dependencies and push again.

Install locally:

```bash
cargo install cargo-machete
cargo machete --workspace
```

### cargo-udeps (unused dev-dependencies)

The `udeps` CI job runs `cargo +nightly udeps --workspace --all-targets` using nightly Rust.
It catches unused `[dev-dependencies]` that `cargo-machete` may miss.

Install locally:

```bash
rustup toolchain install nightly
cargo install cargo-udeps --locked
cargo +nightly udeps --workspace --all-targets
```

### cargo-semver-checks (breaking API changes)

The `semver` CI job runs `cargo semver-checks` on every PR to detect breaking public API changes in `soroban-token-template` and `soroban-escrow-template`.

**Semver policy:** This repository follows [Semantic Versioning](https://semver.org/). Any change that removes, renames, or changes the signature of a public contract entry point, error type, or event is a **breaking change** and requires a major version bump. Adding new public items is backwards-compatible and requires only a minor bump. Bug fixes with no API change require a patch bump.

Install locally:

```bash
cargo install cargo-semver-checks --locked
cargo semver-checks -p soroban-token-template
cargo semver-checks -p soroban-escrow-template
```

---

## Code style

- Format: `cargo fmt --all`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Tests: `cargo test --workspace`
## Documentation updates

- When upgrading `soroban-sdk` or changing the Soroban protocol version, verify
  and update `docs/gas-costs.md`.
- Include a `Last verified` date and protocol version in the document header.
- Confirm that Protocol 22 fee schedule values are still correct and update any
  stale network fee assumptions.

# Contributing to Soroban Starter Kit

Thanks for taking the time to contribute. This guide covers everything you need to get set up, write good code, and get your changes merged.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Dev Environment Setup](#dev-environment-setup)
- [Running Tests](#running-tests)
- [Code Style](#code-style)
- [Adding a New Contract Template](#adding-a-new-contract-template)
- [PR Checklist](#pr-checklist)
- [Issue Labelling Conventions](#issue-labelling-conventions)
- [PR Review Process](#pr-review-process)

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust | **1.82.0** (pinned) | [rustup.rs](https://rustup.rs/) |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI | latest | [docs](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) |
| Docker | 24+ | [docker.com](https://www.docker.com/) |

```bash
# Install Rust (rustup automatically installs 1.82.0 via rust-toolchain.toml)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the WASM compilation target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install stellar-cli
```

### Updating the pinned Rust version

The Rust toolchain is pinned in `rust-toolchain.toml` to ensure reproducible builds across all developers and CI. To update it:

1. Edit `rust-toolchain.toml` and change `channel` to the new version (e.g. `"1.83.0"`).
2. Run `cargo check --workspace` to confirm everything compiles on the new version.
3. Update the version reference in `README.md` and this file to match.
4. Open a PR with the toolchain bump — CI will validate it against all targets.

---

## Dev Environment Setup

### Option A — Local

```bash
git clone https://github.com/Fidelis900/soroban-starter-kit.git
cd soroban-starter-kit

# Start a local Stellar node with Soroban RPC
docker compose up stellar-node
```

### Option B — Dev Container

Open the repo in VS Code and select **Reopen in Container** when prompted. The `.devcontainer/devcontainer.json` configuration installs all prerequisites automatically.

### Environment Variables

Copy the example env file and fill in any values you need for local deployment:

```bash
cp .env.example .env
```

---

## Running Tests

### Unit tests for a single contract

```bash
cd contracts/token   # or contracts/escrow
cargo test
```

### All workspace tests

```bash
cargo test --workspace
```

### Tests with a specific feature flag

Both contracts support optional Cargo features (`pausable`, `upgradeable`, `capped-supply` for token). To test with a feature enabled:

```bash
cargo test -p soroban-token-template --features pausable
cargo test -p soroban-escrow-template --features pausable,upgradeable
```

### Property-based tests

Property tests live in `prop_test.rs` inside each contract and run as part of the normal `cargo test` suite. They use the `proptest` crate and run a configurable number of random cases.

### Benchmarks

```bash
cargo bench -p benches
```

### Build WASM artifacts

```bash
cargo build --target wasm32-unknown-unknown --release -p soroban-token-template
cargo build --target wasm32-unknown-unknown --release -p soroban-escrow-template
```

---

## Code Style

Follow standard Rust conventions. Run these before every commit:

```bash
# Format
cargo fmt --all

# Lint (warnings are treated as errors in CI)
cargo clippy --all-targets -- -D warnings
```

Additional conventions:

- `unsafe` code is forbidden workspace-wide (`unsafe_code = "forbid"`).
- Public functions must have doc comments explaining parameters, errors, and preconditions.
- Use `Result<T, ContractError>` for all fallible contract entry points — never `unwrap` in production paths.
- Keep storage key enums in `storage.rs`; keep event helpers in `events.rs`.
- Prefer `checked_add` / `checked_sub` over raw arithmetic to avoid silent overflow.

### XDR ABI stability

`#[contracttype]` structs (e.g. `EscrowInfo`) are serialised on-chain as XDR maps
keyed by **field name**.  The field names — and their types — are therefore part of
the **public on-chain ABI**.

- **Do not rename, add, or remove fields** without a migration plan and a contract
  version bump.
- The exact set of field names is pinned by an XDR snapshot test in
  `contracts/escrow/src/storage.rs` (`test_escrow_info_xdr_snapshot`).  If you
  intentionally change the struct, update the snapshot constant in that test,
  document the breaking change in `CHANGELOG.md`, and increment the on-chain
  contract version.

---

## DataKey Variant Stability

`DataKey` enums (in `contracts/*/src/storage.rs`) define the on-chain storage layout for each contract. In Soroban, `#[contracttype]` enums use the **variant name** as the XDR storage discriminant. Changing any variant in an incompatible way corrupts storage for any live deployment.

### Rules

| Operation | Effect | Allowed? |
|-----------|--------|----------|
| Rename a variant | Changes its XDR key — existing storage entries become unreachable | **Never** |
| Remove a variant | Same as rename from the runtime's perspective | **Never** |
| Reorder variants | Changes the numeric fallback index in some SDK versions | **Never** |
| Add a variant | Appending at the **end** is safe; inserting in the middle is not | **Append only** |

### How to add a new storage key

1. Open `contracts/<name>/src/storage.rs`.
2. Add the new variant at the **bottom** of the `DataKey` enum, after all existing variants.
3. Update the exhaustive `match` in `discriminant_tests::*_data_key_index` to include the new variant with the next sequential index.
4. Run the tests: `cargo test -p <package>`.

### Why the tests use an exhaustive match

The `discriminant_tests` module in each `storage.rs` contains an exhaustive `match` over `DataKey`. This is intentional:

- **Compile error** if a variant is renamed or removed (the old name no longer exists).
- **Non-exhaustive warning** (treated as an error in CI) if a variant is added without updating the match.
- **Runtime assertions** document the expected position of each variant and serve as a human-readable snapshot of the storage layout.

---

## Adding a New Contract Template

For the full step-by-step guide, see [docs/adding-a-contract.md](docs/adding-a-contract.md).

Follow these steps to add a contract that fits the existing project structure:

1. **Scaffold the crate**

   ```bash
   mkdir -p contracts/<name>/src/bin
   ```

   Create a `Cargo.toml` modelled on `contracts/token/Cargo.toml`. Add the new crate to the workspace `members` list in the root `Cargo.toml`.

2. **Required source files**

   | File | Purpose |
   |------|---------|
   | `src/lib.rs` | Contract entry points (`#[contract]`, `#[contractimpl]`) |
   | `src/storage.rs` | `DataKey` enum and storage helper types |
   | `src/errors.rs` | Contract-specific error enum |
   | `src/events.rs` | Event emission helpers |
   | `src/admin.rs` | Admin auth helpers (can re-export from `soroban-common`) |
   | `src/test.rs` | Unit tests (minimum 8 cases) |
   | `src/prop_test.rs` | Property-based tests using `proptest` |
   | `src/bin/deploy.rs` | CLI deploy binary |
   | `scripts/deploy.sh` | Shell deploy script using the `stellar` CLI |
   | `build.rs` | Build script that bakes `GIT_HASH` into the binary |

3. **Test snapshots**

   Run `cargo test` once to generate initial snapshots under `test_snapshots/`. Commit them alongside the contract.

4. **README update**

   Add a row to the contract template table in `README.md` with the contract name, a one-line description, and a link to the contract directory.

5. **CHANGELOG update**

   Add an entry under `[Unreleased]` in `CHANGELOG.md` describing the new template.

---

## PR Checklist

Before opening a pull request, confirm all of the following:

- [ ] `cargo fmt --all` passes with no changes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] New functionality is covered by tests (unit + property-based where applicable)
- [ ] Public API changes are documented with doc comments
- [ ] `CHANGELOG.md` has an entry under `[Unreleased]`
- [ ] `README.md` is updated if the change affects usage or the template list
- [ ] The PR title is concise (≤ 70 characters) and follows the format `type: short description` (e.g. `feat: add vesting contract template`)
- [ ] The PR description references the issue it closes (`Closes #NNN`)

---

## Issue Labelling Conventions

| Label | Meaning |
|-------|---------|
| `bug` | Something is broken or behaves incorrectly |
| `enhancement` | New feature or improvement to existing functionality |
| `documentation` | Changes to docs, comments, or README only |
| `good first issue` | Well-scoped task suitable for new contributors |
| `help wanted` | Maintainers welcome outside contributions |
| `question` | Clarification needed before work can begin |
| `duplicate` | Already tracked by another issue |
| `wontfix` | Out of scope or intentionally not addressed |
| `security` | Security-related finding — follow `SECURITY.md` for disclosure |
| `ci` | Changes to CI/CD workflows or tooling |
| `breaking change` | Introduces a backwards-incompatible change |

---

## PR Review Process

1. A maintainer will be assigned to review within **3 business days** of opening.
2. CI must be green before review begins. Fix any failing checks first.
3. Reviewers may request changes. Address each comment and re-request review when ready.
4. Once approved by at least one maintainer and CI is green, the PR will be squash-merged into `main`.
5. The merge commit message becomes the CHANGELOG entry, so keep the PR title accurate.

For security issues, do **not** open a public PR. Follow the process in [SECURITY.md](SECURITY.md).
