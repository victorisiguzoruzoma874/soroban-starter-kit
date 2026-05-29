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

---

## Code style

- Format: `cargo fmt --all`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Tests: `cargo test --workspace`
