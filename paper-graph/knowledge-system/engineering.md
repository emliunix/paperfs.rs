# Engineering constraints

**Baseline:** paper-graph at f1e46d9, 2026-11-18. HawDB 0.34.0, Rust 1.97.1 or newer. The crate builds a SQL works table plus six graph labels; it does not build Concept, Claim, Chunk, or EvidenceSpan, and it does not parse PDFs.

## HawDB facts

Verified in this repository:

- One embedded engine with graph/Cypher, relational/SQL, BM25 full-text search, and exact vector search (smoke test, 2026-11-17).
- CREATE TABLE IF NOT EXISTS is rejected because it hides schema drift. Bootstrap checks information_schema.tables before creating works.
- At 324 works, exact cosine scan is sufficient; no approximate nearest-neighbor index is needed.
- Transactions cover SQL and Cypher writes. A sync deletes and reinserts in one transaction.
- The UI opens HawDB once and serializes requests through one handle, consistent with a single-writer lease. Do not open the same database from a concurrent CLI process while the UI is running.

Documented or observed but not verified in this codebase:

- Reads have explicit row/payload budgets and fail over-budget rather than silently truncating.
- Read transactions pin snapshots; copy-on-write agent branches cost metadata plus WAL, not data copies.
- The single-writer lease itself was not stress-tested.

The smoke test did not cover snapshot or branch behavior. Those claims remain unverified here.

## Current store

- SQL works table: item_id PRIMARY KEY, zotero_key, item_type, title, date, year, venue, abstract, authors, collections, tags, search_text, embedding VECTOR(1024).
- Graph labels: Work, Person, Collection, Tag, Venue, Attachment.
- Search: BM25 via text_search on works_search; exact vector via cosine_distance; hybrid as weighted reciprocal-rank fusion, weight / (60 + rank).
- No Concept, Claim, EvidenceSpan, Chunk, Experiment, Tool, or BenchmarkSuite node. No chunk/page/offset table. No PDF text.

This is a metadata-and-abstract index, not the evidence graph.

## Sync and rebuild

Sync is a manual command shared by the CLI and the UI. There is no watcher or daemon. A sync:

1. Reads Zotero through a consistent temporary SQLite snapshot (rusqlite backup API).
2. Embeds every work's search text through llama.cpp.
3. Opens HawDB and runs the full rebuild in one transaction.
4. Writes all works, people, collections, tags, venues, attachments, and edges.

Measured on 2026-11-17: 35.238 s, 26,466 bge-m3 tokens, 324 works, 171 unfiled, 347 people, 5 collections, 24 tags, 10 venues, 161 attachments.

The rebuild is destructive: DELETE FROM works, then MATCH (n) DETACH DELETE n, then reinsert. Table and index definitions survive; all content is replaced. The recorded mitigation is a separate derived database for extraction, but data/ contains only library.hawdb. A limited sync (--limit) can overwrite a full database with a subset; the README warns about this.

What is missing: a non-destructive upsert path, a schema version, an ALTER-based migration, a guard that fails when non-metadata labels are present, and a test that a metadata sync leaves a derived store intact.

## Embeddings

- Model: bge-m3, 1024 dimensions, served by llama.cpp's OpenAI-compatible /v1/embeddings.
- Default URL: https://llama.gw.emliunix.cc/v1; fallback LAN: http://192.168.50.18:8080/v1.
- The first full sync failed on Zotero itemID 1303: 2,113 characters, 522 tokens, against a 512-token physical batch limit, HTTP 500.
- The fix was not truncation. Long metadata is split into chunks, each chunk is embedded, and the vectors are mean-pooled to one 1024-dimensional vector per itemID.
- The server's batch-size and ubatch-size were raised to 32K on the desktop preset; the formerly failing 522-token input then embedded successfully.
- The embedding client silently coerces a missing usage.total_tokens to 0. "Server reported no usage" and "zero tokens" are indistinguishable in the SyncReport.

At 324 works, chunking and pooling are acceptable. At a larger corpus, the number of requests and the pooling policy become a cost and reproducibility issue; record both.

## Corpus admission

- An attachment row is not a readable paper. Four of the first 15 candidates (itemIDs 60, 47, 5, 8) had dangling attachment records; the corpus was replaced with four works that had local PDFs.
- Admission currently requires: Zotero attachment row, at least one referenced local PDF, and a literal %PDF- signature. The corpus JSON records the status pdf_verified_gold_pending and the verification prose, but tests/experiment_corpus.rs:15 only asserts the status string.
- No PDF parser exists. "Usable" is established to file and signature level, not text recoverability. PL papers put hard content in notation, theorems, tables, and two-column layouts; this is untested.
- Metadata is sparse: 117 of 324 works have abstracts; 235 have no year; 171 are unfiled. One work is titled only "PDF" and outranks correctly titled papers in semantic search for "fixed points and recursion schemes".

Admission must become an executable script: resolve attachment, stat file, check %PDF-, record SHA-256, fail on any missing file. Then a parser smoke test on the four development PDFs.

## Run provenance

Current SyncReport: database, embedding_model, embedding_url, embedding_tokens, elapsed_seconds, and flattened stats. It has no run id, timestamp, attempt, finish reason, per-batch error, or raw output. Embedding usage is summed and missing usage becomes 0.

Required before extraction:

- run id, purpose, code commit, model snapshot, prompt/schema/ontology hashes;
- per call: input/output/cached tokens, attempt, finish_reason, content length, reasoning length, truncation flag, wall time, error class;
- per paper: parser version, chunker version, page count, text coverage, empty pages, replacement characters;
- run-level: executed/skipped/failed counts, p50/p95 latency, spend, review minutes;
- missing usage stored as "unreported", never 0.

## The budget inequality

The failure signature is a composition, not a single bad setting:

- output_budget=16,384 exceeded the provider max_tokens=8,192 and timeout=60 s;
- at ~51 tokens/s, a 16K generation alone needs ~321 s;
- the model spent its budget in reasoning_content: finish_reason length, content_len 0, reasoning_len 2,836, completion_tokens 1,024, prompt_tokens 121;
- prompts also reached ~19.8K tokens against a 16K context;
- server restarts produced 500s that looked like extraction failures;
- 16 of 17 KG extraction attempts failed, with usage_unreported=true; 10 memories were dead-lettered at failure_count=3.

Pre-flight:

~~~
context_window >= p95_prompt + output_budget
max_tokens    >= output_budget
timeout       >= output_budget / measured_tokens_per_second
batch_limit   >= largest_input_record
~~~

If any inequality fails, fail the configuration before the job. Classify an empty result by finish_reason and the content/reasoning split before retrying; do not count a restart as a model error.

## Backend selection

- At a few hundred works, exact scan plus an in-process full-text index is sufficient. The decisive variable is how many native layers can drift, not search quality.
- A LanceDB prototype died on a numpy/libstdc++ ABI conflict typical of PyTorch environments; the fallback was SQLite FTS5/BM25, which indexed 286 sections from 45 files and shipped.
- A PostgreSQL pg_search extension built against ICU 72 failed after the base image moved to ICU 76 (undefined symbol uloc_canonicalize_72). Symlinking the old library did not work; the fix was a rebuild against the new ICU. Pin extension binaries to the database image's ABI and re-run an extension smoke test after any base-image change.
- SQLite is synchronous at the C level. Async is a wrapper choice (asyncio.to_thread plus a lock, then aiosqlite's worker thread), not native async. Use one serialized boundary; do not let concurrent extraction workers open separate connections.
- Migrations: an automatic table-rebuild that dropped views was rejected. The accepted fix was a single additive ALTER TABLE ADD COLUMN with a default, as a reviewed one-off script. Paper-graph still rebuilds destructively; the additive remedy is unproven on HawDB.
- The only real-database test is #[ignore]d, asserts seven hardcoded counts, and never calls rebuild or applies a migration. It is a snapshot assertion, not a publication gate.

## Failure modes and checks

| Failure | Signature | Check |
|---|---|---|
| Destructive rebuild | All works and nodes replaced on sync | Count non-metadata labels before sync; fail closed; test a populated derived DB |
| Dangling attachment | Attachment row resolves to no file | Stat file; check %PDF-; record SHA-256 |
| Generic title wins vector search | Work titled "PDF" ranks first | Title-quality flag; title-aware weighting; adversarial query set |
| Embedding batch overflow | HTTP 500 on a 522-token input | Server ubatch >= largest record; chunk+mean-pool if not |
| Empty structured output | content_len 0, finish_reason length | Read finish_reason and reasoning tokens; budget inequality |
| Usage unreported | usage.total_tokens absent | Store "unreported", not 0; fail the manifest |
| Server restart misread as model error | 500s cluster around a restart | Restart marker in the run log; re-issue the same request id |
| Migration drops objects | DROP TABLE/VIEW in startup code | Forbid in CI; one-off reviewed script; additive first |
| Count oracle skipped | #[ignore]d test with fixed counts | Run it in CI on a copy after migrate; assert before/after |

## Operating checklist

Before sync: confirm the target DB; count non-metadata labels; confirm the Zotero snapshot is consistent; record the source hash.

Before extraction: freeze the model snapshot, prompt/schema/ontology hashes, parser version, chunker, and gates; run the budget inequality; run the seeded defects.

Before publishing: executed == corpus size; every claim has a verbatim quote and a valid locator; evidence score 2 for canonical claims; high-risk precision and merge precision pass; cost and review burden recorded; gate ledger re-executed.
