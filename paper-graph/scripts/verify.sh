#!/usr/bin/env bash
set -euo pipefail

CRATE_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
TESTS=(
  check_sync_count_guard
  check_corpus_admission
  check_corpus_identity
  check_evidence_locator
  check_eval_manifest
  check_gold_diff
  check_question_lint
  check_budget_preflight
  check_migration_oracle
)

for test_name in "${TESTS[@]}"; do
  printf '\n==> %s\n' "$test_name"
  cargo test --manifest-path "$CRATE_ROOT/Cargo.toml" --test "$test_name"
done
