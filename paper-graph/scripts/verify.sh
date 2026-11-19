#!/usr/bin/env bash
set -euo pipefail

CRATE_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
TESTS=(
  check_sync_count_guard
  check_corpus_admission
  check_corpus_identity
  check_evidence_locator
)

for test_name in "${TESTS[@]}"; do
  printf '\n==> %s\n' "$test_name"
  cargo test --manifest-path "$CRATE_ROOT/Cargo.toml" --test "$test_name"
done
