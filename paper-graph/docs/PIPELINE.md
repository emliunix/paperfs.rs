# Pipeline — Paper Knowledge System

**Status:** Observed · Specified (baseline `paper-graph` at `f1e46d9`, 2026-11-19; status vocabulary in [method.md](../knowledge-system/method.md#status-vocabulary)).
**Owns:** Source admission (`Ingestion` vs `Admission`), the typed document stage chain, structural-vs-semantic validation boundaries, stage failure states, and provenance fields ([owner-map.md](../knowledge-system/docs/owner-map.md)).
**Governed by:** [CONSTITUTION.md Invariant 4](../knowledge-system/CONSTITUTION.md#4-sources-enter-only-by-verified-bytes-and-signature), [Invariant 5](../knowledge-system/CONSTITUTION.md#5-evidence-is-atomic-and-unknown-stays-visible), and [Invariant 6](../knowledge-system/CONSTITUTION.md#6-evaluation-is-pre-registered-executed-and-recorded).
**Check pointers:** `check_corpus_admission` and `check_evidence_locator` (`Specified`, planned in W4).

## Current Metadata Ingestion (`Observed`)

**Ingestion** imports metadata or bytes without granting active-corpus standing; **Admission** accepts a source only after executable byte, signature, and digest checks ([GLOSSARY.md](../knowledge-system/GLOSSARY.md)).

Today, [`src/pipeline.rs`](../src/pipeline.rs) implements metadata ingestion only: `prepare_sync` snapshots Zotero via [`src/zotero.rs`](../src/zotero.rs), embeds `Work::search_text()` batches via [`src/embeddings.rs`](../src/embeddings.rs) (3 attempts with exponential backoff), and `sync_library` calls `store::rebuild` ([ARCHITECTURE.md](ARCHITECTURE.md)). No PDF parser, chunker, or executable admission check exists in `src/`. `zotero::read_library` (`src/zotero.rs:325-350`) imports `itemAttachments` rows without checking local files: four initial Round-1 candidates (`60`, `47`, `5`, `8`) had dangling attachment rows ([decisions.md](../experiments/ontology-extraction/decisions.md)), and [`tests/experiment_corpus.rs`](../tests/experiment_corpus.rs) line 15 checks only the static string `"pdf_verified_gold_pending"`. In the observed 2026-11-17 sync (`324` works, `161` attachments; [engineering.md](../knowledge-system/engineering.md)), only `117/324` works carried abstracts and `235` lacked years, bounding what metadata-only search can retrieve.

## Typed Stage Chain: `admit -> parse -> chunk -> validate -> index` (`Specified`)

Each stage emits a schema-typed artifact gated before the next stage executes. **Types constrain structure; semantic checks are separate**: type and schema validators guarantee well-formed records and coordinates, whereas semantic validators independently check reading order, quote fidelity, and domain meaning so structurally valid defects remain visible.

1. **`admit` -> `admission.json` (`check_corpus_admission`):** Resolves each candidate's Zotero attachment path, `stat`s byte length, checks a literal five-byte `%PDF-` header, and computes `sha256`.
   - *Typed output:* `AdmissionRecord { item_id, zotero_key, attachment_id, path, bytes, signature_ok, sha256, status }`.
   - *Structural boundary:* Required fields present; `bytes > 0`, `signature_ok == true`, and 64-hex `sha256` on admitted items; missing or non-PDF files emit a typed `AdmissionFailure` record rather than being skipped.
   - *Separate semantic check:* Attachment matches the target `Work` (not supplemental/wrong PDF), and corpus metadata completeness is recorded alongside retrieval scope.
2. **`parse` -> `pages.jsonl`:** Runs a native PDF text parser first, reserving `glm-ocr` for pages where native extraction fails after PL notation verification ([models.md](../knowledge-system/models.md)).
   - *Typed output:* `PageRecord { paper_id, attachment_id, page, text, order_metadata, parser_version, empty_page, replacement_char_count, status }`.
   - *Structural boundary:* Contiguous 1-based `page` numbers, pinned `parser_version`, and explicit `empty_page` / failure records preserved in-stream rather than cleaned away.
   - *Separate semantic check:* Two-column reading order, mathematical notation survival, and bibliography boundary accuracy.
3. **`chunk` -> `chunks.jsonl`:** Normalizes page text into deterministic units consumed by indexing and extraction ([DATA_MODEL.md](DATA_MODEL.md)).
   - *Typed output:* `ChunkRecord { chunk_id, attachment_id, parser_version, chunker_version, page_range, section_path, ordinal, content_hash, raw_text, start_char, end_char }`.
   - *Structural boundary:* `0 <= start_char < end_char`, deterministic `chunk_id` and `content_hash`, valid `page_range`, non-empty `raw_text`.
   - *Separate semantic check:* Theorem premises stay attached to conclusions; overlapping windows are tracked so shared sentences deduplicate during evidence counting.
4. **`validate` -> `validation-report.json` (`check_evidence_locator`):** Runs boundary validators and pre-registered corpus quality checks across `admission.json`, `pages.jsonl`, and `chunks.jsonl`.
   - *Typed output:* `ValidationReport { run_id, structural_pass_rate, locator_resolution_rate, quote_recovery_rate, domain_samples, failures[] }`.
   - *Structural boundary:* 100% schema conformance, hash integrity, and exact `pages.jsonl` slice equality for every `(page_range, start_char, end_char)`.
   - *Separate semantic check:* Sampled verbatim quote recovery per PL domain and locator usability against pre-registered gates (`docs/EVALUATION.md`), reported separately from structural pass rates.
5. **`index` -> Derived HawDB index:** Inserts validated `ChunkRecord` rows, BM25 entries, and `1024`-d `bge-m3` embeddings into the derived store ([ARCHITECTURE.md](ARCHITECTURE.md)).
   - *Typed output:* `IndexManifest { run_id, snapshot_id, indexed_chunks, embedding_model, token_usage }`.
   - *Structural boundary:* Finite `f32[1024]` vectors and exact post-commit count parity with validated chunks.
   - *Separate semantic check:* Non-regression retrieval probes ([EXPERIMENTS.md](EXPERIMENTS.md)).

## Downstream Extraction Handoff and Provenance (`Specified`)

Indexed chunks feed `extract -> validate -> review -> project`: frozen-arm LLM extraction ([ontology.md](../knowledge-system/ontology.md), `docs/EVALUATION.md`) emits `unreviewed` claims/spans -> deterministic quote/locator/ontology checks -> human adjudication (`docs/REVIEW.md`) -> transactional projection of `approved` assertions keyed by `run_id`.

Unlike `SyncReport` ([`src/pipeline.rs`](../src/pipeline.rs) lines 21–29), every stage and model call must record `run_id`, `commit`, input/prompt/schema hashes, `parser_version`, `chunker_version`, `executed/skipped/failed` counts, `finish_reason`, content vs reasoning lengths, latency, and token usage with absent usage stored as `"unreported"` rather than `0` (budget preflight inequalities owned by `docs/OPERATIONS.md`).
