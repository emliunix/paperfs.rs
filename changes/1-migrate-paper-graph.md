# 1 — Fold paper-graph into this workspace and port it to Rust

## Facts

- `paperfs.rs` is a single binary crate (`paperfs_rs`, edition 2021) that builds
  cleanly with `cargo +stable check` on rustc 1.100.0. It already depends on
  axum 0.8, tokio, reqwest, serde_json, anyhow, tracing.
- `../paper-graph` is a Python 3.13 project (about 900 LOC + 6 KB UI HTML):
  - `src/paper_graph/zotero.py` (233 lines): read-only Zotero SQLite import
    through the SQLite backup API into a temp snapshot.
  - `store.py` (310): HawDB DDL, full rebuild, BM25/vector/hybrid search,
    stats, author, work detail.
  - `embeddings.py` (88): OpenAI-compatible `/v1/embeddings` client with
    retries, dimension/finiteness validation, token accounting.
  - `pipeline.py` (33): full manual sync.
  - `cli.py` (134): `sync`, `probe`, `search`, `stats`, `author`, `ui`.
  - `ui.py` (106) + `ui.html`: loopback stdlib HTTP server, search history
    JSONL, manual sync endpoint.
  - `tests/`: store, zotero, and experiment-corpus tests.
  - Its git repository has **zero commits**; every file is untracked.
- The project uses `hawdb` 0.34 Python bindings. `data/library.hawdb` (65 MB)
  is a HawDB 0.34 database holding 324 works plus graph data.
- Rust `hawdb = "0.34"` is published on crates.io and exposes the same engine
  (SQL, Cypher, vector type, search indexes, transactions). Its `rust-version`
  is 1.97.1; the default nightly toolchain here is 1.95, so the workspace must
  build with stable 1.100 (already installed).
- HawDB holds one writer lease per database path per process: opening the same
  path twice fails. The Python UI opens per request and warns against a
  concurrent CLI. The Rust UI must own a single handle instead.
- Assets that must survive: `analysis/` (research notes), `experiments/`
  (round-1 corpus JSON is read by a test), `README.md`, runtime `data/`
  (65 MB, currently untracked), `data/search-history.jsonl`.

## Design

### Workspace layout

- Root `Cargo.toml` keeps its `[package]` and `[patch.crates-io]`, and gains:
  ```toml
  [workspace]
  members = [".", "paper-graph"]
  resolver = "2"
  ```
- Add `rust-toolchain.toml` with `channel = "stable"` so the workspace (and
  hawdb's MSRV 1.97.1) builds with the installed 1.100 stable toolchain.
- Move the project into `paper-graph/`:
  ```
  paper-graph/
    Cargo.toml          # package paper-graph, lib + [[bin]] paper-graph
    README.md           # updated for Rust/workspace
    src/
      lib.rs            # re-exports, DEFAULT constants
      zotero.rs
      embeddings.rs
      store.rs
      pipeline.rs
      cli.rs
      ui.rs
      ui.html
      main.rs
    tests/
      store.rs
      zotero.rs
      experiment_corpus.rs
    analysis/           # moved unchanged
    experiments/        # moved unchanged
    data/               # moved, gitignored (existing hawdb DB + history)
    .gitignore          # data/, search-history.jsonl (kept out of git)
  ```
- Root `.gitignore` gains `paper-graph/data/` and `paper-graph/.venv/`.
- Python-only files are removed after the port is verified: `pyproject.toml`,
  `uv.lock`, `.python-version`, `src/paper_graph/*.py`, `tests/*.py`,
  `.venv`, `.pytest_cache`, the empty `../paper-graph/.git`. The old
  `../paper-graph` directory is removed once all untracked content has been
  moved; `data/` is moved, not deleted.

### Module design

**`zotero.rs`** — same data model as Python:
- `Person { id, name, order_index }`, `NamedId { id, name }`,
  `Attachment { id, key, parent_item_id, path, content_type }`,
  `Work { id, key, item_type, title, date, venue, abstract_note, authors,
  collections, tags }` with `year()`, `author_names()`,
  `collection_names()`, `tag_names()`, `search_text()`.
- `year_from_date()`: manual scan for a four-digit `19xx`/`20xx` run,
  matching the Python regex, no regex dependency.
- `default_zotero_db()`: `$HOME/Zotero/zotero.sqlite`; expand a leading
  `~` in user-supplied paths.
- `read_library(path)`: open read-only with rusqlite, copy through
  `rusqlite::backup::Backup` into a tempfile directory, then run the same four
  field/author/collection/tag queries and the attachment query as Python.
  Field mapping: title, date, publicationTitle -> venue,
  abstractNote -> abstract_note. Author name is `"first last"`, or last only
  when first is empty. Attachments keep only non-deleted rows whose parent is
  an imported work.
- SQL text copied verbatim from Python (itemTypes join, deletedItems filter,
  creatorTypeID = 8, collectionItems, itemTags, itemAttachments).

**`embeddings.rs`**:
- `EmbeddingClient { base_url, model, timeout }`, defaults
  `https://llama.gw.emliunix.cc/v1`, `bge-m3`, 120 s; base URL override from
  `PAPER_GRAPH_EMBEDDING_URL`.
- async `embed(&self, texts: &[String], batch_size: usize)` returns
  `(Vec<Vec<f32>>, u64 tokens)`.
- POST `{base}/embeddings` with `{"model": ..., "input": [...]}` through
  reqwest; 3 attempts with 0.5 * 2^attempt backoff on HTTP and transport
  errors. Sort `data` by `index`; require exactly one row per input; require
  1024 coordinates, all finite; sum `usage.total_tokens`; enrich errors with
  batch start offset and per-input character lengths, like Python.
- f32 coordinates because HawDB vectors are f32 (`Vector::new(&[f32])`).

**`store.rs`**:
- `open_database(path) -> hawdb::Database`.
- `initialize`: probe `information_schema.tables` for `works`; create the
  same table (`VECTOR(1024)`, `TEXT` columns) and
  `CREATE SEARCH INDEX IF NOT EXISTS works_search ON works (search_text)`.
- `rebuild(&mut Database, &Library, &[Vec<f32>])`: length check, initialize,
  one transaction:
  - `DELETE FROM works WHERE item_id IS NOT NULL`
  - `MATCH (n) DETACH DELETE n`
  - per-row parameterized INSERT (same 13 columns)
  - graph nodes and edges batched with
    `UNWIND $rows AS row CREATE (...)` / `MATCH ... CREATE` (works, people,
    collections, tags, venues, attachments, AUTHORED/IN_COLLECTION/TAGGED/
    PUBLISHED_IN/HAS_ATTACHMENT), preserving authorOrder.
- Search functions return `Vec<serde_json::Value>` rows with the same keys as
  Python: `item_id, title, year, authors, venue, collections` plus `score`
  (BM25) or `distance` (vector); hybrid adds `bm25_rank`, `vector_rank`,
  `rrf_score`, keeps distance for vector hits, sorts by
  `(-rrf_score, item_id)`.
- `vector_search` validates `1 <= top_k <= 10000`; cosine SQL unchanged.
- `hybrid_search`: `candidate_k = min(10000, max(20, top_k * 5))`; RRF
  `weight / (60 + rank)`; same tie-break.
- `stats`, `author_works`, `work_detail`: same SQL/Cypher and same output
  shape (including `graph: {people, collections, tags, attachments}`).

**`pipeline.rs`**:
- `sync_library(db_path, zotero_path, embedding_url, model, batch_size,
  limit) -> SyncReport` (CLI path): read library, truncate works and orphan
  attachments when `limit` is set, embed all `search_text` values, open the
  DB once, rebuild, read stats, close; report
  `{database, embedding_model, embedding_url, embedding_tokens,
  elapsed_seconds, ...stats}`.
- `sync_into(db: &mut Database, ...)` (UI path): same work but rebuilds the
  already-open shared handle and takes the same report fields; keeps HawDB's
  single-lease rule.

**`cli.rs`** (clap derive, binary `paper-graph`):
- `sync` (`--batch-size 16`, `--limit`), `probe`, `search QUERY`
  (`--mode bm25|vector|hybrid`, `--top-k 10`, `--bm25-weight 1`,
  `--vector-weight 1`), `stats`, `author NAME`, `ui`
  (`--host 127.0.0.1`, `--port 8765`).
- Shared options `--db`, `--zotero`, `--embedding-url`, `--model`.
- Default DB: `PAPER_GRAPH_DB` if set, else
  `<crate manifest dir>/data/library.hawdb` (stable regardless of cwd; the
  moved data lives there). Default Zotero path as above.
- Every command prints pretty JSON (`serde_json::to_string_pretty`), with
  `serde_json` `Value` for both rows and reports.

**`ui.rs`** (axum, tokio):
- `AppState { db: Arc<std::sync::Mutex<Database>>, db_path, zotero_path,
  embedding_url, model }`.
- Routes: `GET /` (embedded `ui.html` via `include_str!`), `GET
  /api/stats`, `GET /api/search` (q, mode, top_k clamped 1..=50, weights),
  `GET /api/work?id=`, `GET /api/history` (last 20 lines), `POST
  /api/sync`.
- Search: BM25 runs entirely under the lock. Vector/hybrid embed the query
  (await) first, then lock for the DB read. Sync: read Zotero + embed first,
  then lock for rebuild. This serializes DB access while network waits do not
  hold the lock.
- History: append `{ts, query, mode, top_k, item_ids}` to
  `<db_dir>/search-history.jsonl`; timestamp RFC3339 UTC.
- Errors return `{"error": "..."}` with 500, unknown work returns 404 and
  `{"error":"Work not found"}`, matching Python.
- Bind loopback by default; log the URL like Python.
- If `hawdb::Database` turns out not to be `Send`, fallback: a dedicated
  owner thread with mpsc request/response so the handle never crosses threads.
  (Probe will confirm.)

**`main.rs`**: `#[tokio::main]`, parse and dispatch.

### Tests (ported)

- `tests/store.rs`: in-memory `Database::new()`, same sample library and
  vectors; asserts graph counts, BM25 ids, vector distance 0, hybrid result,
  author lookup, idempotent rebuild.
- `tests/zotero.rs`: year variants and search_text composition.
- `tests/experiment_corpus.rs`: parse the moved
  `experiments/ontology-extraction/round-001-corpus.json` with serde_json and
  assert status, 15 unique ids, split sizes 4/11, disjoint, domain sets, titles.

### Docs

- `paper-graph/README.md` updated: `cargo run -p paper-graph -- ...`
  commands, workspace note, Rust/HawDB note, data location, tests
  `cargo test -p paper-graph`. Root README gets a short sub-project section.

## Why it works

- The Rust `hawdb` 0.34 crate is the same engine and version as the Python
  extension, so DDL, SQL, Cypher, BM25, exact cosine search, transactions and
  the on-disk format are unchanged: the existing 65 MB database opens as-is.
- Stable 1.100 satisfies hawdb's 1.97.1 MSRV, and the root crate already
  builds on stable, so one workspace toolchain covers both members.
- A single shared `Database` handle respects HawDB's per-path write lease;
  a mutex serializes requests; query embedding (network) happens before taking
  the lock.
- rusqlite's backup API reproduces Python's consistent Zotero snapshot without
  touching Zotero.
- axum/tokio/reqwest/serde_json are already proven in the root crate, and the
  UI HTML and API shapes are preserved, so the front end needs no changes.

## Files

Create:
- `changes/1-migrate-paper-graph.md`
- `rust-toolchain.toml`
- `paper-graph/Cargo.toml`
- `paper-graph/src/{lib,main,zotero,embeddings,store,pipeline,cli,ui}.rs`
- `paper-graph/src/ui.html` (moved)
- `paper-graph/tests/{store,zotero,experiment_corpus}.rs`
- `paper-graph/README.md` (moved + rewritten)
- `paper-graph/.gitignore`

Modify:
- `Cargo.toml` (add `[workspace]`)
- `.gitignore` (ignore paper-graph data/venv)
- `README.md` (sub-project note)

Move:
- `../paper-graph/analysis -> paper-graph/analysis`
- `../paper-graph/experiments -> paper-graph/experiments`
- `../paper-graph/data -> paper-graph/data` (untracked runtime data, 65 MB)

Delete after verification:
- `../paper-graph/src/paper_graph/*.py`, `../paper-graph/tests/*.py`,
  `pyproject.toml`, `uv.lock`, `.python-version`, `.venv`,
  `.pytest_cache`, `../paper-graph/.git`, the emptied
  `../paper-graph` directory.

## Risks / assumptions to review

1. hawdb Rust parameter binding accepts `Value::Vector` for `VECTOR(1024)`
   and `text_search('index', $1, top_k => $2)` with positional params.
2. `UNWIND $rows AS row CREATE (...)` accepts a list of maps and creates one
   node per row inside the rebuild transaction.
3. `Database` is `Send`; otherwise the dedicated-thread fallback is needed.
4. rusqlite's `Backup` API works against a live Zotero DB with WAL.
5. Root crate and the new member compile together with stable; hawdb's build
   time is acceptable.
