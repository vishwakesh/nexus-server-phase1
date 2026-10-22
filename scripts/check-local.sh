#!/usr/bin/env bash
set -euo pipefail
cargo fmt --check
cargo check
cargo test
if command -v node >/dev/null 2>&1; then
  node --check frontend/app.js
else
  echo "Skipping JavaScript syntax check: Node.js is not installed."
fi
