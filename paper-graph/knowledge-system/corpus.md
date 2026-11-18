# Thread corpus

## Selection

Four retrieval scouts searched Nowledge Mem with these query families:

- HawDB / paper-graph / Zotero library / paper knowledge graph
- ontology extraction / PL ontology / knowledge-graph schema / claims and evidence
- LLM graph extraction / GraphRAG / typed extraction / document entity graph
- extraction evaluation rubric / annotation agreement / pre-registered experiment / downstream answer quality
- vector database experiment / graph database benchmark / hybrid search / embedded database
- extraction failure timeout / empty structured output / embedding batch limit / database rebuild delete
- Zotero library / paper metadata / literature review workflow / reading notes
- distill thread memory / durable lessons / knowledge capture / memory curation

Inclusion required a concrete decision, experiment, failure, measurement, or methodology that transfers to the paper knowledge system. Generic coding sessions, greetings, and threads connected only by a shared keyword were excluded.

The result is 18 threads. One additional candidate (21506b3e, an episode-close ritual) was excluded because its principle-extraction practice was already covered by 7172fd66. The retained set is not a random sample; it is the lineage of this system plus the evaluation and database incidents that shaped its rules.

## The 18 threads

| Thread | Date | Msgs | Role | What it contributes |
|---|---|---:|---|---|
| 71ffc1b7-6183-43a3-b95e-7f103170e669 | 2026-11-17 | 200 | Core lineage | Zotero library analysis; HawDB smoke test; metadata graph and sync; embedding 512-to-32K fix; the "PDF" title ranking defect; corpus admission failure; evaluation ceiling; ontology and evidence design; DeepSeek critique; metric panel; destructive rebuild finding. |
| 78dd3c13-7698-4bfc-ad30-b3005f2aa7bf | 2026-10-16 | 353 | Eval integrity | A harness that silently executed zero cases while reporting success; the fix and the first real run. |
| e1f865e1-07c8-4d21-89e7-a7118f9097df | 2026-10-16 | 146 | Gold leakage | Question wording presupposed answers; schema definitions moved six expected answers; the stop rule when a definition tweak trades wins for losses. |
| d64e9502-1a93-45cf-8f19-9f4befc28b17 | 2026-10-18 | 416 | Run traceability | Scores without model/request fields; "not done" records containing blast radius; the owner's ruling that gold is the spec. |
| fc1cc7be-a64b-4e8c-b8c0-ebfc54f51d50 | 2026-10-07 | 2493 | Batch budgets | A partitioner that respected bytes but exceeded the judge's token budget; two operators got opposite call counts; fixtures shaped to a layout production never emits. |
| d0e2d0bd-d957-43dd-92d8-3a594578217e | 2026-10-14 | 1360 | Evaluator validation | Gold witnesses, seeded defects, brute-force references, property tests, and the bug where budget exhaustion returned PASS instead of BLOCKED. |
| 28b5b22b-ae23-4d14-883f-32f13722ca15 | 2026-10-11 | 27 | Evaluator invariants | The first 27 messages of the doc-verify session (same conversation as d0e2d0bd, not independent): judge as fact source, unknown propagation, soundness against false PASS. |
| cb2db461-7165-4291-8607-5e57e973dd81 | 2026-04-29 | 226 | Evidence schema | Atomic evidence, evidence/claim many-to-many, only evidence is a leaf, topological order, schema drift between SKILL.md and FORMAT.md. |
| e943b0c4-16f1-4d8a-9e52-be1097b3cf75 | 2026-03-24 | 64 | Paper index | The per-paper line-number index convention and sed recipes; deterministic provenance anchors. |
| 3099ed1f-a355-4ed5-a6ca-526deaabf295 | 2026-03-22 | 108 | Paper layout | A .txt per paper, an index.md for the txt, and reading.md as the grand index; merge new notes into the per-paper index. |
| 5509b0c1-64da-43ed-81b1-c2a72f6da6ab | 2026-03-04 | 39 | Distillation idea | Working context vs durable memory; memory as an index; summary plus archive pointer; explicit no-write mode. An analogy, not paper-system evidence. |
| 7172fd66-4eb3-4819-aea3-02629fcc7029 | 2026-07-25 | 145 | Hard lessons | A design note must record decisions, rejected alternatives, and hard lessons — not temporary test scaffolding or process narrative. |
| f8ea8d0d-00d5-48d0-ae66-e382c9ef3a92 | 2026-09-28 | 22 | Extraction budget | output_budget 16,384 vs provider max_tokens 8,192 and 60 s timeout; 16 of 17 attempts failed; dead-lettered memories. |
| c6422ed2-d314-4330-a896-33873a5062c0 | 2026-10-06 | 191 | Timeout mechanics | finish_reason length with content_len 0; 51 tokens/s means 16,384 tokens needs 321 s; 500s were self-inflicted server restarts; user raised timeout to 600 s. |
| 05a7de1d-2a8f-49cd-a93d-0aa1ab261f8e | 2026-02-20 | 54 | Backend choice | LanceDB failed on numpy/libstdc++; SQLite FTS5/BM25 indexed 286 sections and shipped. |
| d9cbb41d-773e-4917-a187-1d83c30991eb | 2026-05-05 | 141 | Migration safety | Automatic table-rebuild migration rejected; one-off SQL; final fix was one additive ALTER with a default. |
| dcca45f9-766e-479c-9a7e-de14d82bdef4 | 2026-02-11 | 91 | Extension ABI | pg_search built for ICU 72 failed after the base image moved to ICU 76; version pinning and re-test after base-image changes. |
| c59ca24c-7c5e-4bd6-8217-d759e703e79e | 2026-05-01 | 636 | Embedded DB concurrency | SQLite is synchronous at the C level; the async boundary is a wrapper choice (aiosqlite worker thread), not native async. |

## What each group is for

- **Core lineage (71ffc1b7).** The only thread that designs the paper system end-to-end. Everything else either supplies a method or an incident.
- **Evaluation integrity (78dd3c13, e1f865e1, d64e9502, fc1cc7be).** Four independent projects showing how a green eval can be empty, untraceable, or bent by the question.
- **Evaluator validation (d0e2d0bd / 28b5b22b).** One doc-verify session, captured twice. Counts once as a source, but supplies the strongest worked example of validating an evaluator against gold and seeded defects.
- **Evidence and paper addressability (cb2db461, e943b0c4, 3099ed1f).** The claim/evidence DAG and the line-index reading workflow.
- **Distillation (5509b0c1, 7172fd66).** When to write durably and what a durable record must contain.
- **Operations (f8ea8d0d, c6422ed2, 05a7de1d, d9cbb41d, dcca45f9, c59ca24c).** Budgets, timeouts, native dependencies, ABI drift, SQLite concurrency, migration safety.

## Limits

- One user, one Zotero library, one PL/type-theory corpus.
- One model family in the eval incidents (DeepSeek variants).
- The core thread is 200 messages and ends with the experiment planned, not run.
- The paper-index and distillation threads are from other projects; they transfer as practice, not as measured results for this system.
- 28b5b22b and d0e2d0bd are the same conversation; the corpus has 17 independent sessions, not 18.
