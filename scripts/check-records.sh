#!/bin/sh
# Development-only checks; no package install or user configuration writes.
set -eu
cd "$(dirname "$0")/.."
node --test tests/records/*.test.mjs
node scripts/check-product-records.mjs
