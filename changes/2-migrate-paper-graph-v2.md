# 2 — Migration plan v2 (review corrections)

Supersedes `changes/1-migrate-paper-graph.md` where they differ. The original
stays append-only; this records the independent review verdict
(APPROVE WITH CHANGES) and the corrections that were applied.

## Review blockers and disposition

1. **Default DB path was wrong.** HawDB paths are directories, not files.
   `default_db_path` now checks `is_dir()`; the previous `is_file()` made the
   manifest fallback silently create an empty database. Verified from the
   workspace root: `cargo run -p paper-graph -- stats` reads the moved database
   and reports 324/171/347/5/24/10/161.
2. **Backup before moving.** A full copy now exists at
   `/Users/emliunix/Documents/paper-graph-backup-2026-11-18` (278 files,
   79 MB including `data/`, checksums of analysis/experiments/ui.html match).
   `data/` (232 files, 79 MB) was then moved to `paper-graph/data/` and the
   source path verified gone. The reviewed counts and sha256 comparison gate is
   complete before any source deletion.
3. **HTTP parity.** `ui.rs` now extracts `Result<Query<T>, QueryRejection>` and
   maps rejections to `500` JSON `{"error": ...}`, adds a JSON 404 fallback,
   and a `map_response` middleware that sets `Cache-Control: no-store` and
   `Content-Type: application/json; charset=utf-8` on JSON responses.
   Verified with curl: missing `q`, bad `top_k`, bad `id`, unknown mode and
   unknown path all return the Python-compatible status/body; valid stats keeps
   the headers.
4. **Toolchain/CI/Docker.** `rust-toolchain.toml` pins `1.100.0`.
   `Dockerfile` uses `rust:1.100-trixie` (Docker Hub tag checked).
   GitHub Actions uses `cargo build --workspace` and
   `cargo test --workspace`. `.dockerignore` excludes
   `/paper-graph/data`, `/paper-graph/.venv`, and `/.git`.
   `Cargo.lock` is the workspace lockfile and includes `paper-graph`.

## Corrections to the original design text

- `sync_into(&mut Database, ...)` was replaced by `prepare_sync()` (Zotero
  read + embeddings) followed by a short `rebuild` under the UI mutex, so the
  lock is never held across network I/O.
- Hybrid search sorts by `(-rrf_score, item_id)` **and** truncates to `top_k`,
  matching Python's `[:top_k]`.
- Absent `score`/`distance` keys are omitted from JSON (`skip_serializing_if`),
  not serialized as null, matching Python dict rows.
- The work field is stored as `abstract_note` but serialized as `abstract`.
- Relationship directions are
  `(p)-[:AUTHORED]->(w)`, `(w)-[:IN_COLLECTION]->(c)`,
  `(w)-[:TAGGED]->(t)`, `(w)-[:PUBLISHED_IN]->(v)`,
  `(w)-[:HAS_ATTACHMENT]->(a)`.
- `Vector::new(&[f32]) -> Result<Vector>`; vector SQL params use
  `Value::Vector` while graph batches use `query_with_params(&BTreeMap)` and
  SQL batches use `query_sql_with_params(&[Value])`.
- Embedding URL precedence is documented: `--embedding-url` (clap default from
  `PAPER_GRAPH_EMBEDDING_URL`) then `https://llama.gw.emliunix.cc/v1`.
- Probe output is exactly
  `{zotero, works, attachments, embedding_model, embedding_url,
  embedding_dimensions, embedding_tokens}`.
- Search history timestamps use `SecondsFormat::Micros` and `+00:00`, matching
  Python's `datetime.isoformat()`.
- `embeddings.rs` counts Unicode characters, not bytes, in error enrichment.
- `paper-graph` sets `publish = false` and `rust-version = "1.97.1"`;
  target discovery uses the conventional `src/lib.rs` and `src/main.rs`.

## Additional tests added after review

- `tests/zotero_fixture.rs`: builds a synthetic Zotero SQLite database and
  checks the snapshot import, field mapping, author/collection/tag/attachment
  mapping, `search_text`, truncation, and the missing-database error.
- `tests/store.rs`: adds `work_detail` graph-shape/serialization assertions and
  verifies BM25/vector JSON omit the absent scoring key.
- `tests/existing_database.rs`: opt-in `#[ignore]` test that opens a real
  database via `PAPER_GRAPH_REAL_DB` and asserts the 324/171/347/5/24/10/161
  counts. It passed against the pre-move database copy.

## Remaining steps

- Re-copy the latest `analysis/` (a concurrent writer added
  `sutra-os-research.md` at 08:54 after the first copy).
- Remove the Python-only files and the old `../paper-graph` tree after the
  backup and moved-asset comparison; keep a symlink at the old path pointing to
  `paperfs.rs/paper-graph` so existing commands continue to work.
- Run `cargo fmt --all`, `cargo clippy --workspace`, `cargo test --workspace`,
  and a final full-sync dry run only against the copy.
