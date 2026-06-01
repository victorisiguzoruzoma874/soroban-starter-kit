# List available recipes
default:
    @just --list

# Build all contracts
build:
    cargo build --release

# Run unit tests
test:
    cargo test

# Run unit tests with all features enabled
test-all-features:
    cargo test --all-features

# Run Clippy lints
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Format source code
fmt:
    cargo fmt --all

# Deploy contracts to testnet
deploy-testnet:
    ./scripts/deploy.sh testnet

# Deploy contracts to local node
deploy-local:
    ./scripts/deploy.sh local

# Run benchmarks
bench:
    cargo bench

# Remove build artifacts
clean:
    cargo clean
