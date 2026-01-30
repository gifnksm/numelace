#!/usr/bin/env bash
set -euo pipefail

echo "==> Local CI (mirrors GitHub Actions)"

echo "==> cargo check"
cargo check --all-targets --all-features

echo "==> cargo doc"
cargo doc --no-deps

echo "==> cargo test"
cargo test

echo "==> cargo bench (build only)"
cargo bench --no-run

echo "==> cargo fmt"
cargo fmt --all -- --check

echo "==> cargo clippy (host)"
cargo clippy --all-targets --all-features -- -D warnings

echo "==> cargo clippy (wasm32)"
cargo clippy --all-features --target wasm32-unknown-unknown --lib --bins -- -D warnings

echo "==> trunk build"
trunk build

echo "==> typos"
typos

echo "All checks passed."
