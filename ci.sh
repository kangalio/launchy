#!/bin/bash
set -euo pipefail

cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo doc --no-deps
