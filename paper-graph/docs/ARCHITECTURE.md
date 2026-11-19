# Architecture — Paper Knowledge System

**Status:** Decided · Observed · Specified (baseline `paper-graph` at `f1e46d9`, 2026-11-19; status vocabulary in [method.md](../knowledge-system/method.md#status-vocabulary)).
**Owns:** Layer ownership (`Canonical` vs `Derived`), trust boundaries, runtime vs target component topology, and the paper/praxis pipeline split ([owner-map.md](../knowledge-system/docs/owner-map.md)).
**Governed by:** [CONSTITUTION.md Invariant 1](../knowledge-system/CONSTITUTION.md#1-canonical-rebuildable-and-distilled-layers-stay-separate) and [ADR 0003](../knowledge-system/ADR/0003-separate-derived-store-for-extraction-and-review.md).
**Check pointer:** `check_metadata_sync_guard` (`Specified`, planned in W4); durable promotion uses `judgment (human)` (`knowledge-system/PRAXIS.md`).

## Storage Layers and Ownership

**Canonical** state is the authoritative source from which downstream indexes can be reconstructed; **Derived** state is computed from canonical inputs and remains disposable only within its declared rebuild boundary ([GLOSSARY.md](../knowledge-system/GLOSSARY.md)).

1. **Canonical bibliographic store (`Observed`):** The user's local Zotero SQLite database (`~/Zotero/zotero.sqlite`) and attachment directory (`~/Zotero/storage`). [`src/zotero.rs`](../src/zotero.rs) opens Zotero with `SQLITE_OPEN_READ_ONLY` and copies a consistent snapshot into a temporary directory via `rusqlite::backup::Backup` (`read_library`, lines 148–173). `paper-graph` never mutates Zotero.
2. **Rebuildable metadata index (`Observed`):** An embedded HawDB 0.34 database at `data/library.hawdb` ([`src/cli.rs`](../src/cli.rs) lines 194–208) storing the SQL `works` table, BM25 index `works_search`, 1024-dimensional `bge-m3` vectors, and six bibliographic graph labels ([DATA_MODEL.md](DATA_MODEL.md)). This store is disposable by design: `store::rebuild` deletes all `works` rows and executes `MATCH (n) DETACH DELETE n` in one transaction ([`src/store.rs`](../src/store.rs) lines 107–109).
3. **Derived extraction and review store (`Decided · Specified`, unbuilt):** A separate derived HawDB store ([ADR 0003](../knowledge-system/ADR/0003-separate-derived-store-for-extraction-and-review.md)) isolating parsed page/chunk coordinates, extracted concepts/claims/evidence spans, human review verdicts, and per-run provenance outside the metadata wipe target. Today `data/` contains only `library.hawdb` ([engineering.md](../knowledge-system/engineering.md)). Cross-store queries join through durable identifiers owned by [DATA_MODEL.md](DATA_MODEL.md); fail-closed sync guards, single-writer recovery, and additive migrations belong to `docs/OPERATIONS.md`.
4. **Distilled layer (`Specified`):** Promoted reading notes and reusable engineering lessons carrying pointers back to verified evidence rather than automatic copies of working context; owned by `knowledge-system/PRAXIS.md`.

## Trust Boundaries and Current Runtime (`Observed`)

At `f1e46d9`, [`src/lib.rs`](../src/lib.rs) exposes six modules: `cli`, `zotero`, `embeddings`, `store`, `pipeline`, and `ui`.

- **Process and database boundary:** `ui::serve` binds an `axum` HTTP server to loopback (`127.0.0.1:8765` default, [`src/cli.rs`](../src/cli.rs) lines 99–103) and serializes embedded HawDB access through a single `Mutex<hawdb::Database>` handle ([`src/ui.rs`](../src/ui.rs) lines 33–55), respecting HawDB's single-writer lease. To avoid blocking queries on network I/O, `prepare_sync` snapshots Zotero and computes embeddings before locking `db` for `rebuild` ([`src/pipeline.rs`](../src/pipeline.rs) lines 54–78; [`src/ui.rs`](../src/ui.rs) lines 229–240). Search requests append audit entries to `search-history.jsonl` alongside the database ([`src/ui.rs`](../src/ui.rs) lines 249–282).
- **Model and network boundary:** [`src/embeddings.rs`](../src/embeddings.rs) (lines 7–10) posts metadata batches to an OpenAI-compatible `llama.cpp` `/v1/embeddings` endpoint (`https://llama.gw.emliunix.cc/v1` default; LAN `http://192.168.50.18:8080/v1` keeps metadata on the local network). Target extraction, OCR fallback, privacy-lane routing, and cross-family review routes follow [models.md](../knowledge-system/models.md) and [method.md](../knowledge-system/method.md#reviewer-independence). Nowledge Mem is a separate external application, not a runtime or pipeline dependency.

## Current vs Target: The Two Pipelines

| Pipeline | Current state (`Observed`) | Target topology (`Specified`) |
|---|---|---|
| **Paper pipeline** | Manual metadata/abstract sync only: `zotero::read_library -> EmbeddingClient::embed -> store::rebuild`, plus BM25/vector/RRF search over `library.hawdb`. No PDF parser, chunker, or derived claim store exists. | Metadata sync continues to refresh `library.hawdb`, while admitted PDFs traverse the staged document chain ([PIPELINE.md](PIPELINE.md)) into the separate derived store, scored under `docs/EVALUATION.md` and adjudicated under `docs/REVIEW.md` before approved assertions project into the query graph. |
| **Praxis pipeline** | 18 mined threads and 14 prose/JSON audit records in [`knowledge-system/threads.json`](../knowledge-system/threads.json), [`round-2-rescan.json`](../knowledge-system/round-2-rescan.json), and [lessons.md](../knowledge-system/lessons.md). | Normalized `Episode -> Lesson -> Check` entities in `knowledge-system/PRAXIS.md` (`lessons.jsonl`), kept in a distinct namespace from scholarly `Claim` nodes and wired into executable `check_*` enforcement. |
