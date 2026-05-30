# Adding a New Contract Template

This checklist walks you through every step required to add a new contract to the starter kit so it integrates cleanly with the workspace, CI, and deployment tooling.

---

## 1. Directory Structure

Create the following layout under `contracts/<name>/`:

```
contracts/<name>/
├── Cargo.toml
├── build.rs
├── scripts/
│   └── deploy.sh
└── src/
    ├── lib.rs          # contract entry points
    ├── storage.rs      # DataKey enum and storage helpers
    ├── errors.rs       # contract-specific error enum
    ├── events.rs       # event emission helpers
    ├── admin.rs        # admin auth helpers
    ├── test.rs         # unit tests (minimum 8 cases)
    ├── prop_test.rs    # property-based tests
    └── bin/
        └── deploy.rs   # CLI deploy binary
```

```bash
mkdir -p contracts/<name>/src/bin contracts/<name>/scripts
```

---

## 2. Cargo.toml Template

Copy this template and replace `<name>` and `<description>`:

```toml
[package]
name = "soroban-<name>-template"
version = "0.1.0"
edition = "2021"
authors.workspace = true
description = "<description>"
license.workspace = true

[features]
pausable = []
upgradeable = []

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
soroban-sdk = { workspace = true }
soroban-common = { workspace = true }

[dev-dependencies]
soroban-sdk = { workspace = true, features = ["testutils"] }
proptest = { workspace = true }

[[bin]]
name = "deploy"
path = "src/bin/deploy.rs"
doc = false

[lints]
workspace = true
```

### Required feature flags

| Feature | Purpose |
|---------|---------|
| `pausable` | Adds an emergency pause mechanism |
| `upgradeable` | Allows the contract WASM to be upgraded by the admin |

Add domain-specific features as needed, but always include `pausable` and `upgradeable` so consumers have a consistent opt-in surface.

---

## 3. Add to the Workspace

Open the root `Cargo.toml` and add the new crate to `members`:

```toml
[workspace]
members = [
    "contracts/common",
    "contracts/token",
    "contracts/escrow",
    "contracts/<name>",   # add this line
    "tests",
    "fuzz",
]
```

Verify the workspace resolves cleanly:

```bash
cargo check --workspace
```

---

## 4. Minimum Test Coverage

`src/test.rs` must contain **at least 8 unit test cases** covering:

- [ ] Successful happy-path flow
- [ ] Each error variant the contract can return
- [ ] Authorization checks (wrong caller is rejected)
- [ ] Boundary / edge-case inputs (zero amounts, max values)

`src/prop_test.rs` must contain at least one property-based test using `proptest`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_<name>_property(amount in 1u64..=u64::MAX) {
        // assert invariants hold for arbitrary inputs
    }
}
```

Run all tests before opening a PR:

```bash
cargo test -p soroban-<name>-template
cargo test -p soroban-<name>-template --features pausable,upgradeable
```

---

## 5. Deploy Script

`scripts/deploy.sh` must accept a network argument (`testnet` or `mainnet`) and use the `stellar` CLI:

```bash
#!/usr/bin/env bash
set -euo pipefail

NETWORK=${1:-testnet}
WASM=target/wasm32-unknown-unknown/release/soroban_<name>_template.wasm

stellar contract build --manifest-path contracts/<name>/Cargo.toml

stellar contract deploy \
  --wasm "$WASM" \
  --network "$NETWORK" \
  --source "$STELLAR_SECRET_KEY"
```

Make it executable:

```bash
chmod +x contracts/<name>/scripts/deploy.sh
```

---

## 6. Update README.md

Add a row to the contract template table in `README.md`:

```markdown
| **<Name>** | One-line description | Use case A, use case B | ✅ Complete |
```

Also add a `### <Name> Contract Features` section listing the key capabilities.

---

## 7. Final Checklist

- [ ] Directory structure matches the layout in section 1
- [ ] `Cargo.toml` uses workspace-inherited fields and includes `pausable` / `upgradeable` features
- [ ] Crate added to root `Cargo.toml` `members`
- [ ] `cargo check --workspace` passes
- [ ] At least 8 unit tests and 1 property-based test
- [ ] `cargo test -p soroban-<name>-template` passes (with and without features)
- [ ] `scripts/deploy.sh` is present and executable
- [ ] Row added to the template table in `README.md`
- [ ] Entry added under `[Unreleased]` in `CHANGELOG.md`
