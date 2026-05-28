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
| Rust | latest stable | [rustup.rs](https://rustup.rs/) |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI | latest | [docs](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) |
| Docker | 24+ | [docker.com](https://www.docker.com/) |

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the WASM compilation target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install stellar-cli
```

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

---

## Adding a New Contract Template

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
