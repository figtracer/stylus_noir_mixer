#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CRATES=(poseidon imt mixer)

for crate in "${CRATES[@]}"; do
  echo "==> contracts/${crate}: cargo stylus check"
  (cd "${ROOT_DIR}/contracts/${crate}" && cargo stylus check)
done

echo "All Stylus checks passed."


