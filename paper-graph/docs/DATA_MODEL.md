# Data Model — Paper Knowledge System

**Status:** Decided · Observed · Specified (baseline `paper-graph` at `f1e46d9`, 2026-11-19; status vocabulary in [method.md](../knowledge-system/method.md#status-vocabulary)).
**Owns:** Physical SQL/Cypher schemas, durable `Work` identity, `Assertion` / `Claim` / `Evidence` / `EvidenceSpan` structures, review state values, and visible `unknown` representation ([owner-map.md](../knowledge-system/docs/owner-map.md)).
**Governed by:** [CONSTITUTION.md Invariant 2](../knowledge-system/CONSTITUTION.md#2-durable-identity-and-evidence-coordinates-survive-rebuilds), [Invariant 5](../knowledge-system/CONSTITUTION.md#5-evidence-is-atomic-and-unknown-stays-visible), and [ADR 0004](../knowledge-system/ADR/0004-atomic-claim-evidence-with-quote-and-locator.md).
**Check pointers:** `check_corpus_identity` (`Specified`, planned in W4; re-import durability `Untested`) and `check_evidence_locator` (`Specified`, planned in W4).

## Current Metadata Schema — Generated from Code (`Observed`)

Derived directly from [`src/store.rs`](../src/store.rs) and [`src/zotero.rs`](../src/zotero.rs) in `library.hawdb` ([ARCHITECTURE.md](ARCHITECTURE.md)):

- **SQL table `works`** (`src/store.rs:10-24, 50-58`):
  `item_id BIGINT PRIMARY KEY`, `zotero_key TEXT NOT NULL`, `item_type TEXT NOT NULL`, `title TEXT NOT NULL`, `date TEXT`, `year BIGINT` (first `19xx`/`20xx` parsed in `src/zotero.rs:133-145`), `venue TEXT`, `abstract TEXT`, `authors TEXT` (`"; "`-joined), `collections TEXT` (`" | "`-joined), `tags TEXT` (`" | "`-joined), `search_text TEXT` (newline-joined title, authors, venue, abstract, tags; `src/zotero.rs:99-112`), and `embedding VECTOR(1024)`.
- **Indexes and retrieval queries:** BM25 index `works_search ON works (search_text)` queried via `text_search('works_search', $1, top_k => $2)` (`src/store.rs:33-38, 56`); exact cosine scan `cosine_distance(embedding, $1)` (`src/store.rs:356-363`); hybrid reciprocal-rank fusion `weight / (60.0 + rank)` (`src/store.rs:369-406`).
- **Cypher nodes** (`src/store.rs:114-186, 256-272`): `:Work {id, zoteroKey, title, year}`, `:Person {id, name}`, `:Collection {id, name}`, `:Tag {id, name}`, `:Venue {name}`, `:Attachment {id, zoteroKey, path, contentType}`.
- **Cypher edges** (`src/store.rs:231-289`): `(:Person)-[:AUTHORED {authorOrder}]->(:Work)`, `(:Work)-[:IN_COLLECTION]->(:Collection)`, `(:Work)-[:TAGGED]->(:Tag)`, `(:Work)-[:PUBLISHED_IN]->(:Venue)`, `(:Work)-[:HAS_ATTACHMENT]->(:Attachment)`.

No `Concept`, `Claim`, `Chunk`, or `EvidenceSpan` table or label exists in `src/` today.

## Durable Identity vs Local Join Keys (`Observed · Decided · Specified`)

- **Observed gap:** `src/zotero.rs:180-236` reads Zotero `itemID`, `key`, `title`, `date`, `publicationTitle`, and `abstractNote` (DOI is not imported yet). `src/store.rs` keys SQL rows (`item_id`) and Cypher matches (`w.id`) on numeric `itemID`; `zotero_key` (`zoteroKey`) is stored passively without a join index.
- **Specified contract (`check_corpus_identity`):** A **Work** is durably identified by `(zotero_key, doi)`—never by title or content hash ([GLOSSARY.md](../knowledge-system/GLOSSARY.md)). Numeric `itemID` remains a local join key to a single Zotero snapshot only. Derived extraction tables bind to `zotero_key` (plus `doi` when present) and maintain an explicit `item_id <-> zotero_key` mapping so Zotero merges or restores cannot orphan claims. Unresolved citations remain `ExternalWork` records with `cited_work_item_id = null` rather than nearest-title guesses ([ontology.md](../knowledge-system/ontology.md)).

## Target Derived Schema: Claims, Evidence, Review States, and `Unknown` (`Specified`)

Domain concept kinds and relation predicates (`MENTIONS`, `USES`, `INTRODUCES`, etc.) belong to [ontology.md](../knowledge-system/ontology.md). The separate derived store ([ADR 0004](../knowledge-system/ADR/0004-atomic-claim-evidence-with-quote-and-locator.md)) specifies:

- **`Chunk`:** `chunk_id` derived from `(attachment_id, parser_version, ordinal, content_hash)`, plus `page_range`, `section_path`, `raw_text`, `start_char`, and `end_char` ([PIPELINE.md](PIPELINE.md)).
- **`Assertion` and `Claim`:** An **Assertion** is a candidate statement pending evidence and review; a **Claim** binds one atomic proposition (keeping necessary premises attached) with `claim_id`, `work_zotero_key`, `normalized_statement`, `claim_type`, `polarity`, `qualifiers`, `attribution`, `run_id`, `schema_version`, and `review_state`.
- **`Evidence` and `EvidenceSpan` (`check_evidence_locator`):** **Evidence** is a supporting or challenging source observation; **EvidenceSpan** is its leaf record persisting `attachment_item_id`, attachment `zotero_key`/`sha256`, `chunk_id`, `page`, verbatim `quote`, `start_char`, `end_char`, `quote_hash`, `stance` (`supports` | `refutes` | `qualifies`), `quote_validation_state`, and `locator_validation_state`. `HAS_EVIDENCE` links claims and spans many-to-many in a DAG with spans at the leaves; identical text across overlapping chunks deduplicates to one support.
- **Review states** (adjudication rules in `docs/REVIEW.md`): `unreviewed` (initial state for all model outputs), `needs_revision`, `approved`, and `rejected`.
- **Visible `unknown`:** Ambiguous or absent values (such as `citation_function`, unavailable pre-parser locators, or unreported API token usage currently coerced to `0` at [`src/embeddings.rs`](../src/embeddings.rs) lines 109–112) persist explicit `unknown` / `unreported` markers and keep dependent claims `unreviewed` rather than coercing defaults.
