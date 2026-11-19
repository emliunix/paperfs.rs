# Module verification

**Status:** Observed (2026-11-19). **The repository is the verification authority.** `paper-graph/scripts/verify.sh` runs every module check locally over the crate; GitHub Actions only mirrors that script. A check that exists only in CI is not a check, because nobody can run it on a laptop, in a worktree, or offline.

## Gate

```sh
bash paper-graph/scripts/verify.sh     # all module checks; non-zero on any failure
cargo test --workspace                 # full crate suite, including stage tests
```

A change is verified when both pass on the current tree. The scripts are self-contained: no network, no GitHub, no credentials.

## Module matrix

| Module | Contract it owns | Verification | Seeded defect that must fail | Owner |
|---|---|---|---|---|
| `zotero.rs` | Read a Zotero SQLite snapshot; resolve attachments; never write the library | `cargo test --test zotero --test existing_database` | dangling attachment row; missing local file | W6 |
| `store.rs` | Metadata rows are derived; rebuild is destructive; derived counts must survive sync | `check_sync_count_guard` | derived rows dropped by a sync | W4 |
| `pipeline.rs` | Sync is manual, fail-closed, and count-guarded | `check_sync_count_guard`, `check_migration_oracle` | startup DROP TABLE/VIEW; count mismatch | W4 |
| `parse.rs` | PDF -> typed page records (paper_id, page, text, parser version) | `cargo test --test parse` | corrupt PDF; empty page marked ok | W6 |
| `chunk.rs` | Pages -> typed chunks (offsets, hash, section path, ordinal) | `cargo test --test chunk` | offset drift; hash drift; skipped ordinal | W6 |
| admission (W6 stage) | Corpus papers admitted by bytes and signature only | `check_corpus_admission` | invalid `%PDF-`; missing file | W6 |
| identity (corpus) | `itemID` <-> `zotero_key`/DOI mapping | `check_corpus_identity` | duplicate identity; missing key | W4 |
| evidence | A claim quote is an exact substring with a resolvable locator | `check_evidence_locator` | paraphrased quote; wrong character offset | W4 |
| evaluation | A run proves execution and matches its corpus | `check_eval_manifest` | skipped case; zero-case run reporting pass | W4 |
| gold | Expected answers never move without a cited decision | `check_gold_diff` | silent expected-answer edit | W4 |
| questions | No outcome-presupposing wording; no schema-field conflation | `check_question_lint` | "what does X prove" style prompt | W4 |
| budgets | context / max_tokens / timeout / retries satisfy the inequalities | `check_budget_preflight` | timeout below p95; retries > 2 | W4 |
| migrations | No destructive startup migration; count oracle on a copy | `check_migration_oracle` | `DROP TABLE` in startup | W4 |
| `embeddings.rs` | 1024-dimensional responses preserve input order across bounded batches; transport and shape failures are explicit | `check_embeddings_contract` | 1023-dimensional row; unreachable router silently returning a zero vector | W4 |
| `ui.rs` | Offline `/api/stats` and `/api/work` return stable status codes, JSON shapes, and cache headers | `check_ui_api_contract` | missing work returns 200; required stats field omitted | W4 |
| `cli.rs` | clap parsing and exit codes; `--limit` is restricted to destructive sync | `check_cli_contract` | invalid arguments exit 0; `--limit` accepted by non-destructive commands | W4 |

## Rules

1. Every module check has a seeded defect; a check that cannot turn red is documentation, not verification.
2. Checks are typed where possible: the same `serde` structs that validate production output validate test fixtures.
3. Judgement-only subjects (L06, L09, L13) carry a named human owner instead of a check and are listed in `docs/rule-check-matrix.md`; they are not silently green.
4. CI is a mirror: `.github/workflows/rust.yml` calls the same commands. Removing CI must not remove verification.
5. A failing check blocks the merge locally; a green CI with a failing local script is a defect in the CI wiring, not a pass.
