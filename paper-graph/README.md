# Paper graph

A local proof of work for exploring a Zotero library with HawDB graph queries,
BM25, exact vector search, and reciprocal-rank-fusion hybrid search.

This is now a Rust workspace member of `paperfs.rs` and is built with the
native `hawdb` 0.34 crate. The database format is unchanged: an existing
`library.hawdb` created by the earlier Python implementation opens as-is.

The current pipeline indexes **metadata and abstracts only**. It does not yet
extract PDF full text, citations, or OCR text. Zotero numeric `itemID` values are
the work identities; no content-hash identity is introduced.

## Components

- Read-only Zotero SQLite import through a consistent temporary snapshot
  (`rusqlite` backup API).
- HawDB 0.34 SQL table for metadata, search text, and 1024-dimensional vectors.
- HawDB Cypher graph for works, people, collections, tags, venues, attachments,
  and their relationships.
- `bge-m3` embeddings through llama.cpp's OpenAI-compatible `/v1/embeddings`.
- A loopback-only axum web UI with search-mode, top-K, and hybrid weight
  controls, work details, search history, and a manual full-sync button.

## Setup

The workspace pins the stable Rust toolchain in `rust-toolchain.toml`.
`hawdb` 0.34 requires rustc 1.97.1 or newer.

```bash
cd ~/Documents/paperfs.rs
cargo build -p paper-graph
```

The default database is `paper-graph/data/library.hawdb`. Override it with
`--db` or `PAPER_GRAPH_DB`. The default Zotero database is
`~/Zotero/zotero.sqlite`; override it with `--zotero`.

Use the LAN llama.cpp endpoint to keep paper metadata off the public route:

```bash
export PAPER_GRAPH_EMBEDDING_URL=http://192.168.50.18:8080/v1
```

The CLI falls back to `https://llama.gw.emliunix.cc/v1` when neither the
environment variable nor `--embedding-url` is set.

### llama.cpp input limit

The `bge-m3` preset must accept complete metadata records. On host `desktop`,
`/home/liu/Documents/llama-cpp/presets.ini` contains:

```ini
[bge-m3]
model = ./ggml-org/bge-m3-Q8_0-GGUF/bge-m3-q8_0.gguf
n-gpu-layers = 0
embeddings = true
device = none
split-mode = none
batch-size = 32768
ubatch-size = 32768
```

The previous physical batch limit of 512 tokens rejected Zotero `itemID` 1303,
whose metadata uses 522 tokens. The 32K setting was verified against that work;
no client-side truncation is used.

## Manual sync

A sync embeds every work and replaces all derived SQL and graph data in one
HawDB transaction. There is no watcher or background scheduler.

```bash
cargo run -p paper-graph -- sync \
  --embedding-url http://192.168.50.18:8080/v1
```

Useful development options are `--batch-size 16` and `--limit 4`. Do not use a
limited sync against the real database unless you intend to replace it with
that subset.

The completed 2026-11-17 full sync took 35.238 seconds and 26,466 embedding
tokens. It produced 324 works, 171 unfiled works, 347 people, 5 collections,
24 tags, 10 venues, and 161 attachments.

## Search and graph queries

```bash
cargo run -p paper-graph -- search 'higher-rank polymorphism' \
  --mode bm25 --top-k 5 \
  --embedding-url http://192.168.50.18:8080/v1

cargo run -p paper-graph -- search 'higher-rank polymorphism' \
  --mode vector --top-k 5 \
  --embedding-url http://192.168.50.18:8080/v1

cargo run -p paper-graph -- search 'higher-rank polymorphism' \
  --mode hybrid --top-k 5 --bm25-weight 1 --vector-weight 1 \
  --embedding-url http://192.168.50.18:8080/v1

cargo run -p paper-graph -- author 'Kiselyov'
cargo run -p paper-graph -- stats
```

BM25 is best for exact terminology. Vector search can find semantically related
works with weak or generic titles. Hybrid search combines the two rank lists as
`weight / (60 + rank)`. With 324 works, exact cosine search is sufficient; no
approximate nearest-neighbor index is needed.

## UI

```bash
cargo run -p paper-graph -- ui \
  --embedding-url http://192.168.50.18:8080/v1
```

Open <http://127.0.0.1:8765>. The server binds to loopback by default. Search
history is appended to `data/search-history.jsonl`. **Sync** asks for
confirmation before rebuilding all embeddings and graph data.

The UI opens HawDB once and serializes requests through one handle, so it does
not fight the database's single-writer lease. Do not open the same database
from a concurrent CLI process while the UI is running.

## Tests

```bash
cargo test -p paper-graph
```

The tests use synthetic works and vectors; they do not contact Zotero,
llama.cpp, or any network service.

## Migration note

This crate was ported from the Python `paper_graph` package on 2026-11-18.
The module split mirrors the original (`zotero`, `embeddings`, `store`,
`pipeline`, `cli`, `ui`), the CLI and HTTP API shapes are preserved, and the
on-disk HawDB format is identical.

## Current limitations and next experiments

- Only 117 of 324 works have abstracts, and many records have missing years or
  authors. Search quality is bounded by Zotero metadata.
- Generic titles such as `PDF` can rank highly in vector search. Metadata
  cleanup and title-aware ranking are immediate evaluation candidates.
- PDF extraction should start with a local native parser, using OCR only for
  documents that need it. LlamaParse would upload paper content and therefore
  requires a separate explicit opt-in.
- A useful next evaluation is 12–20 preselected questions comparing BM25,
  vector, and weighted hybrid results before adding full-text ingestion.
- Citation extraction and the Lean 4 proof-search integration are not started.

## Research and experiment records

- `analysis/llm-document-graph-research.md` surveys primary research and
  official tools for schema-constrained, evidence-bearing document graphs.
- `analysis/deepseek-ontology-critique.md` records the secondary DeepSeek
  design critique and which suggestions were not adopted.
- `experiments/ontology-extraction/` contains the round-1 ontology-specificity
  plan, a 15-paper locally verified corpus, metric rubrics, and an honest gate
  ledger. No extraction arm has run; independent plan review, PDF parsing,
  human gold annotation, and the scoring harness are still pending.
