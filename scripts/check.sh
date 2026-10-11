#!/bin/sh
# Same credential-free gate locally and in CI.
set -eu
cd "$(dirname "$0")/.."
sh scripts/check-records.sh
cargo test --locked
cargo build --release --locked
