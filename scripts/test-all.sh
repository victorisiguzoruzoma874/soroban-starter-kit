#!/usr/bin/env bash
set -euo pipefail

PASS=0
FAIL=0

run_suite() {
    local name="$1"
    shift
    if "$@" 2>&1; then
        echo "[PASS] $name"
        PASS=$((PASS + 1))
    else
        echo "[FAIL] $name"
        FAIL=$((FAIL + 1))
    fi
}

echo "=== Running all test suites ==="

run_suite "unit tests"              cargo test
run_suite "unit tests (all-features)" cargo test --all-features
run_suite "integration tests"       cargo test --test '*'
run_suite "property-based tests"    cargo test --features proptest
run_suite "benchmarks (dry-run)"    cargo bench --no-run

echo ""
echo "=== Summary ==="
echo "  Passed: $PASS"
echo "  Failed: $FAIL"

[ "$FAIL" -eq 0 ]
