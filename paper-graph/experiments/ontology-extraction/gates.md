# Gate ledger

A `complete: yes` row records work actually performed. A `complete: no` row is
not a pass and has no inferred result.

| Track | complete | Evidence / current state |
|---|---|---|
| Research survey | yes | `analysis/llm-document-graph-research.md`; primary sources and official docs fetched 2026-11-18 |
| DeepSeek secondary critique | yes | `analysis/deepseek-ontology-critique.md`; configured credential passed without disclosure; generic ontology text only |
| Initial DeepSeek attempt | yes | Failed with `MISSING_CREDENTIAL`; parent later supplied the configured `DEEPSEEK_API_KEY` securely and succeeded |
| Corpus attachment rows | yes | Read-only HawDB queries over selected Zotero `itemID` values |
| Corpus local PDF admission | yes | 15/15 admitted, 0 failed, with existing PDFs and `%PDF-` signatures; evidence: [`admission.json`](../../_runs/2026-11-19/admission.json) |
| Dangling attachment detection | yes | `60`, `47`, `5`, `8` lacked the referenced local PDF and were excluded |
| Corpus structure | no | Historical `uv run pytest`: 5 passed on 2026-11-18, but that command is stale on this Rust tree; no current executable corpus-structure result is recorded |
| Exploratory metadata baseline | yes | Four simple hybrid queries each returned a sensible rank-1 work; these are not formal round results |
| Independent plan review | no | `round-001-review.md` absent; required before model extraction |
| PDF parse smoke test | yes | 4/4 development PDFs parsed, 157 page records, 0 failures; evidence: [`parse-report.json`](../../_runs/2026-11-19/parse-report.json) and [`pages.jsonl`](../../_runs/2026-11-19/pages.jsonl) |
| Quote recovery and locator validity | no | 4/4 chunking and 226 chunks are recorded in [`chunk-report.json`](../../_runs/2026-11-19/chunk-report.json) and [`chunks.jsonl`](../../_runs/2026-11-19/chunks.jsonl), but quote recovery and locator validity were not measured; `_runs/2026-11-19/validation-report.json` is absent |
| Ontology/annotation manual | no | Research draft exists; reviewed v0 manual absent |
| Gold annotations and agreement | no | No gold files; no inter-annotator measurements |
| Downstream gold questions | no | Question design only; no reviewed `questions.json` |
| Executable seeded defects | no | Required cases listed in the rubric; fixtures/scorer absent |
| Model/provider freeze and price | no | No extraction model snapshot or dated run price selected |
| Extraction arms `U2`/`S2`/`P2` | no | No hosted extraction run and no spend incurred for an arm |
| Deterministic scorer | no | Not implemented |
| Human blind scoring | no | Not performed |
| Round verdict | no | `round-001-verdict.md` absent |

## Exploratory baseline, not a formal experiment

The existing UI's balanced metadata/abstract hybrid search returned:

- “papers applying bidirectional typechecking to higher-rank polymorphism” →
  rank 1: `3154`.
- “unifying structured recursion schemes and fixed points” → rank 1: `2314`.
- “free monads and higher-order effect handlers” → rank 1: `1490`.
- “elaboration and instance parameters in Lean” → rank 1: `774`.

These queries have little headroom and must not be used to claim an ontology
improvement. They remain simple-retrieval non-regression checks.
