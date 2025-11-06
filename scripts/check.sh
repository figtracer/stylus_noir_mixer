#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CRATES=(poseidon imt mixer)

# defaults for e2e tests if not already set
export RPC_URL="${RPC_URL:-http://localhost:8547}"
export DEPLOYER_ADDRESS="${DEPLOYER_ADDRESS:-0xcEcba2F1DC234f70Dd89F2041029807F8D03A990}"

# arg parsing
RUN_CHECKS=true
RUN_TESTS=true
for arg in "$@"; do
  case "$arg" in
    --checks-only)
      RUN_TESTS=false
      ;;
    --tests-only)
      RUN_CHECKS=false
      ;;
    -h|--help)
      echo "usage: $0 [--checks-only | --tests-only]"
      exit 0
      ;;
    *)
      echo "unknown option: $arg" >&2
      echo "usage: $0 [--checks-only | --tests-only]" >&2
      exit 1
      ;;
  esac
done

if [ "$RUN_CHECKS" = true ]; then
  for crate in "${CRATES[@]}"; do
    echo "==> contracts/${crate}: cargo stylus check"
    (cd "${ROOT_DIR}/contracts/${crate}" && cargo stylus check) || \
      echo "[warn] stylus check failed for ${crate} (continuing)"
  done
fi


# run e2e tests per crate, per test file (ignore failures)
if [ "$RUN_TESTS" = true ]; then
  for crate in "${CRATES[@]}"; do
    echo "==> contracts/${crate}: cargo test --features e2e (per test file)"
    (
      cd "${ROOT_DIR}/contracts/${crate}" || exit 1
      shopt -s nullglob
      for tf in tests/*.rs; do
        test_name="$(basename "${tf%.rs}")"
        echo "---- running: cargo test --features e2e --test ${test_name}"
        cargo test --features e2e --test "${test_name}" || \
          echo "[warn] test ${crate}/${test_name} failed (continuing)"
      done
    )
  done
fi

