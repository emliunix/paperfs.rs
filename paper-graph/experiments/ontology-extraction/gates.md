# Gate ledger

A `complete: yes` row records work actually performed. A `complete: no` row is
not a pass and has no inferred result.

| Track | complete | Evidence / current state |
|---|---|---|
| Research survey | yes | `analysis/llm-document-graph-research.md`; primary sources and official docs fetched 2026-11-18 |
| DeepSeek secondary critique | yes | `analysis/deepseek-ontology-critique.md`; configured credential passed without disclosure; generic ontology text only |
| Initial DeepSeek attempt | yes | Failed with `MISSING_CREDENTIAL`; parent later supplied the configured `DEEPSEEK_API_KEY` securely and succeeded |
| Corpus attachment rows | yes | Read-only HawDB queries over selected Zotero `itemID` values |
| Corpus local PDF admission | yes | 15/15 revised candidates have an existing PDF and `%PDF-` signature |
| Dangling attachment detection | yes | `60`, `47`, `5`, `8` lacked the referenced local PDF and were excluded |
| Corpus structure | yes | `uv run pytest`: 5 passed on 2026-11-18; development/test IDs disjoint and all four domains present in both splits |
| Exploratory metadata baseline | yes | Four simple hybrid queries each returned a sensible rank-1 work; these are not formal round results |
| Independent plan review | no | `round-001-review.md` absent; required before model extraction |
| PDF parse smoke test | no | No PDF parsed; notation, layout, and quote recovery unknown |
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
