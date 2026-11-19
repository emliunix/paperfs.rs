# Operations — Paper Knowledge System

**Status:** Decided · Observed · Specified · Transferred (baseline `paper-graph` at `f1e46d9`, 2026-11-19).
**Owns:** Rebuild safety and manual metadata sync; operations, backend selection, and additive schema migration; extraction call budgets and per-call provenance ([owner-map.md](../knowledge-system/docs/owner-map.md) rows 19–20, 26; [CONSTITUTION.md](../knowledge-system/CONSTITUTION.md) Invariants 3 and 6; [ADR 0003](../knowledge-system/ADR/0003-separate-derived-store-for-extraction-and-review.md)).
**Checks:** `check_sync_count_guard` (planned, W4); `check_migration_oracle` (planned, W4); `check_budget_preflight` (planned, W4).

Layer topology lives in [ARCHITECTURE.md](ARCHITECTURE.md), pipeline stage schemas in [PIPELINE.md](PIPELINE.md), and live model endpoints in [models.md](../knowledge-system/models.md). Empirical background appears in [engineering.md](../knowledge-system/engineering.md) and [lessons.md](../knowledge-system/lessons.md) (`L03`, `L12`, `L14`).

## Current State: Observed vs Specified

- **Observed:** Manual synchronization runs through `paper-graph sync` (`src/cli.rs`:28, 109–120) and `POST /api/sync` (`src/ui.rs`:86), invoking `sync_library` (`src/pipeline.rs`:80–110). `zotero::read_library` snapshots `~/Zotero/zotero.sqlite` via `rusqlite::backup::Backup` before reading (`src/zotero.rs`:4). `store::rebuild` (`src/store.rs`:61–112) runs `DELETE FROM works` and `MATCH (n) DETACH DELETE n` in one HawDB 0.34 transaction; passing `--limit` overwrites the target database with that subset (`src/pipeline.rs`:64–66; [README.md](../README.md) lines 78–80). A full sync measured 35.238 s and 26,466 `bge-m3` tokens across 324 works on 2026-11-17 ([README.md](../README.md) lines 82–84). `EmbeddingClient` (`src/embeddings.rs`:25, 109–142) hardcodes a 120 s timeout, runs an unlogged `0..3` retry loop (max 2 retries with 0.5 s / 1.0 s backoff), and coerces absent `/usage/total_tokens` to `0`. `tests/existing_database.rs`:9–21 is `#[ignore]`d.
- **Specified (unexecuted):** Separate derived HawDB store for extraction and review, pre-sync non-metadata count guard (`check_sync_count_guard`), active migration count oracle (`check_migration_oracle`), `p95`-sized timeouts, budget preflight (`check_budget_preflight`), typed failure records, and `"unreported"` usage provenance.
- **Transferred (untested on HawDB):** Additive `ALTER TABLE` one-off migration discipline and ABI-pinned single-engine deployment ([engineering.md](../knowledge-system/engineering.md) lines 101–108; `L03`, `L14`).

## Manual Sync, Single-Writer Lease, and Backup

There is no file watcher or background sync daemon. Operators trigger metadata rebuilds explicitly after confirming the target database path (`PAPER_GRAPH_DB` or `--db`, default `paper-graph/data/library.hawdb`). Under `check_sync_count_guard`, `sync` queries non-metadata tables and graph labels before opening a rebuild transaction and fails closed when any derived extraction, claim, or review state exists ([CONSTITUTION.md](../knowledge-system/CONSTITUTION.md) Invariant 3).

HawDB enforces a single-writer lease. `ui::serve` (`src/ui.rs`:34, 71–78) holds one `Mutex<hawdb::Database>` handle across HTTP requests after `prepare_sync` finishes embedding outside the lock (`src/pipeline.rs`:50–78). Operators must not open the same database directory from a concurrent CLI command while `paper-graph ui` runs. Before any sync or schema operation on a populated derived store, copy the database directory on disk and record the Zotero source digest; recovery restores that directory snapshot rather than running inverse Cypher deletes.

## Backend Selection and Additive Schema Migration

The crate standardizes on embedded `hawdb` 0.34 (rustc >= 1.97.1) to keep SQL, Cypher, BM25, and exact 1024-d cosine scan inside one process without external server ABIs ([engineering.md](../knowledge-system/engineering.md) lines 6–14, 101–106; [GUIDELINES.md](../knowledge-system/GUIDELINES.md) `GP-06`). `store::initialize` (`src/store.rs`:50–58) queries `information_schema.tables` rather than `CREATE TABLE IF NOT EXISTS`.

Startup code never executes `DROP TABLE`, `DROP VIEW`, or automatic table-rebuild migrations. Every schema change requires an explicit schema-version bump and a reviewed, additive one-off script executed against a database copy, gated by a non-ignored `check_migration_oracle` test verifying pre- and post-migration work, claim, evidence, and review counts before swapping the database path.

## Timeouts Sized from p95, Bounded Retries, and Typed Failures

Model and embedding calls are nondeterministic distributions and must pass `check_budget_preflight` before execution ([engineering.md](../knowledge-system/engineering.md) lines 79–99; `worklog/2026-11-19/todo.md` lines 193–195, 549):

1. **Call-budget inequalities and `p95` timeouts:** Pre-flight checks enforce `context_window >= p95_prompt_tokens + output_budget`, `max_tokens >= output_budget`, and `batch_limit >= largest_input_record`. Request timeouts are never arbitrary constants; they are sized from measured `p95` latency on the frozen T0.6 benchmark (`timeout >= max(measured_p95_latency, output_budget / measured_tokens_per_second) + cold_start_margin`, accounting for on-demand `llama.cpp` model loads in [models.md](../knowledge-system/models.md) lines 46–58).
2. **Bounded retries (max 2) and no silent retries:** Transient transport or structural parse retries are capped at **at most 2 retries** (3 total attempts; structural repair in extraction rubrics permits at most 1 structural retry per [round-001-rubric.md](../experiments/ontology-extraction/round-001-rubric.md) lines 12–14). Every attempt—including intermediate failures—is logged with `run_id`, `attempt` index, elapsed wall time, HTTP status or `finish_reason`, and content/reasoning token lengths. Silent retry loops and cross-model fallback retries until pass are forbidden.
3. **Typed failure records and `unreported` usage:** Exhausted retries, `finish_reason == "length"` truncations, zero-length content with reasoning overflow, upstream 429/500 responses, and server restarts emit explicit typed failure records (`error_class`, `attempt_count`, `raw_response_ref`, `restart_marker`) rather than partial objects, empty gold graphs, or model-quality penalties. Missing token usage is persisted literally as `"unreported"`, replacing the `unwrap_or(0)` coercion in `src/embeddings.rs`:109–112.
