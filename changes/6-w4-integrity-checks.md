# W4 checks 6-10 and rule coverage

## Facts
- Existing integration tests are self-contained specifications under `paper-graph/tests/check_*.rs`; no production hooks exist for these rules.
- `paper-graph/scripts/verify.sh` enumerates the four existing test targets, which cover checks 1-5.
- The owner map already has a check column and names the five new check IDs; only that column may change in this task.
- The user forbids git operations and edits to `src/*.rs` and `Cargo.toml`.
- HawDB supports on-disk databases through `open_database(Some(path))`, enabling a real recursive-copy migration oracle.

## Design
1. Add `check_eval_manifest.rs`: deserialize a strict manifest, validate required run provenance, reconcile unique per-case results with independent corpus size, reject skips/failures, and map rejected runs to exit code 1. Seed missing manifest fields, skipped cases, and count mismatch.
2. Add `check_gold_diff.rs`: diff expected answers by case ID and require a human commit, deciding-sentence citation, and second reviewer for every mutation. Seed an uncited answer move and incomplete authorization.
3. Add `check_question_lint.rs`: reject unconditional what/which-file creation questions and schema fields combining stage/status with decision/outcome; accept neutral conditional wording and split fields. Seed both oracle-bending defects.
4. Add `check_budget_preflight.rs`: enforce context, max-token, p95/throughput-plus-cold-start timeout, batch, and 1-3 total-attempt inequalities with finite numeric inputs. Seed each violated inequality and excessive attempts.
5. Add `check_migration_oracle.rs`: token-lint actual startup sources and SQL fixtures for DROP TABLE/VIEW; recursively copy a populated temporary HawDB database, run additive SQL only on the copy, and compare work/claim/evidence/review counts. Seed startup DROP statements and copied-row deletion.
6. Add `docs/rule-check-matrix.md` with all L01-L14, check type, owner, enforcement path, and explicit judgment-only rows where semantics or contested choices cannot be automated.
7. Add the five test target names to `scripts/verify.sh`. Leave all owner-map columns unchanged except check IDs where a correction is needed.

## Why it works
- Integration-test-local validators follow the established W4 pattern and require no production or manifest changes.
- Passing fixtures demonstrate behavior while explicit seeded-defect tests prove each validator rejects the targeted failure.
- Real temporary files and HawDB copies exercise the migration boundary without touching operator data.
- The matrix distinguishes deterministic coverage from human semantic/contested judgments rather than claiming prose is a check.

## Files
- Create `changes/6-w4-integrity-checks.md` (this plan).
- Create `paper-graph/tests/check_eval_manifest.rs`.
- Create `paper-graph/tests/check_gold_diff.rs`.
- Create `paper-graph/tests/check_question_lint.rs`.
- Create `paper-graph/tests/check_budget_preflight.rs`.
- Create `paper-graph/tests/check_migration_oracle.rs`.
- Create `paper-graph/docs/rule-check-matrix.md`.
- Edit `paper-graph/scripts/verify.sh`.
- Edit only the check column in `paper-graph/knowledge-system/docs/owner-map.md` if needed.

## Self-review
- No call sites or production APIs change; all new functions are test-local.
- All five owner-map check IDs match the requested filenames.
- Seeded defects are assertions of validator errors in passing tests, so the clean suite remains green while defect detection is exercised.
- Independent reviewer delegation is unavailable at this subagent depth; plan consistency was checked directly against the cited docs and existing test conventions.
