# Paper knowledge system — distilled design knowledge

A local, evidence-first knowledge system over the user's Zotero library: canonical metadata in Zotero, parsed text and line indexes as the evidence substrate, a rebuildable HawDB index for graph plus BM25 plus vector search, a reviewed Concept/Claim/Evidence graph in a separate derived store, and distilled reading notes that point back to evidence.

This folder records what three rounds of thread mining and code reconciliation established, what remains specification, and what the system cannot yet claim.

## Status

- **18 threads** mined, 17 independent sessions (28b5b22b is the first 27 messages of d0e2d0bd).
- **14 lessons**, each tagged Observed / Decided / Specified / Transferred / Contested.
- **3 rounds**: broad collection, point-level rescan, adversarial audit. Round 3 found 17 contradictions or overclaims and regraded the first two rounds.
- **No extraction arm has run.** No PDF has been parsed. No gold annotations exist. The gate ledger marks plan review, PDF parse, annotation, gold, seeds, model freeze, U2/S2/P2, scorer, blind scoring, and verdict all "no".
- Code baseline: paper-graph at f1e46d9 (2026-11-18). The Rust crate builds only Work, Person, Collection, Tag, Venue, Attachment; there is no Concept or Claim node, no chunk/page/offset table, no PDF parser, and no scorer.

Treat every outcome-shaped sentence as a hypothesis to test, not a result.

## The system in one diagram

~~~
Zotero (canonical metadata, PDFs, annotations)
  -> parsed text + line index + chunk/page/offset coordinates
  -> HawDB derived index (SQL rows, graph, BM25, vectors)
  -> reviewed Concept/Claim/Evidence graph (separate derived store)
  -> distilled reading notes and lessons (pointers, not copies)
~~~

The lifecycle is: **admit -> parse -> chunk -> extract -> validate -> review -> project -> retrieve -> distil**. Each arrow is a place where provenance or review state can be lost; the lessons are mostly about not losing it.

## What is here

| File | Contents |
|---|---|
| [lessons.md](lessons.md) | The 14 rules with evidence, boundary, and an operational check. Start here. |
| [corpus.md](corpus.md) | The 18 threads, why each was selected, and what it contributes. |
| [threads.json](threads.json) | Machine-readable thread manifest: id, title, date, message count, role. |
| [method.md](method.md) | The three-round mining method: scan queries, point refinement, rubric, adversarial audit. |
| [extraction-plan.md](extraction-plan.md) | Per-point extraction questions, evidence standard, rubric, and distillation target. |
| [rescan.md](rescan.md) | Round 2 verdicts and the Round 3 corrections, including the contradictions that changed the wording. |
| [ontology.md](ontology.md) | Paper knowledge ontology v0: nodes, 17 relations, evidence model, negative distinctions, and open issues. |
| [methodology.md](methodology.md) | Evaluation-first protocol: arms, frozen inputs, metric panel, gates, integrity checks, evaluator validation. |
| [engineering.md](engineering.md) | HawDB facts, sync/rebuild, embeddings, corpus admission, run provenance, backend selection. |
| [models.md](models.md) | Model inventory: authorized subagent route, local llama.cpp models, nmem provider, role matrix, smoke-test plan. |
| [open-questions.md](open-questions.md) | The six gaps this corpus cannot close and the next pilot. |
| [worklog/2026-11-19/todo.md](../../worklog/2026-11-19/todo.md) | The work plan with dependency graph, parallel waves, per-workstream flowcharts, subagent plan, and workflow contracts (relocated to the repo worklog 2026-11-19). |
| [worklog/2026-11-19/todo-model-worktree.md](../../worklog/2026-11-19/todo-model-worktree.md) | Model routing matrix, per-task assignment, worktree creation/merge/cleanup, and risk register. |
| [round-2-rescan.json](round-2-rescan.json) | Raw Round 2 lesson records with verbatim quotes and code checks. |
| [round-3-audit.json](round-3-audit.json) | Raw Round 3 findings: contradictions, 35 code claims, replication audit, top gaps. |

## The fourteen rules

1. **L01 Layer ownership.** Zotero is canonical; the HawDB metadata graph is rebuildable; extraction and review state need a separate store.
2. **L02 Identity.** itemID is a local key; persist zotero_key/DOI and evidence coordinates, not just the row id.
3. **L03 Rebuild safety.** Manual sync; never let a metadata rebuild delete derived knowledge; scope or isolate, then prove with a count oracle.
4. **L04 Admission.** Resolve the file and check the signature; a status string is not an admission gate; metadata coverage caps retrieval claims.
5. **L05 Ontology scope.** Start from competency questions; no named IS_A is not enough if a closed kind enum hides the taxonomy.
6. **L06 Relation strength.** MENTIONS / USES / INTRODUCES by licensing evidence; INTRODUCES is a work-scoped contribution claim, not priority.
7. **L07 Evidence.** Atomic claims, atomic evidence, many-to-many links, verbatim quote plus locator, unknown stays visible.
8. **L08 Evaluation first.** Smallest end-to-end artifact, one pre-registered axis, frozen inputs, questions before scorer.
9. **L09 Metric panel.** Per-relation + high-risk + coverage + evidence + downstream QA + review burden; strict F1 as one diagnostic.
10. **L10 Eval integrity.** Prove execution count, record the run, keep gold intact, lint questions and schema; validate the accounting against an independent oracle.
11. **L11 Evaluator validation.** Invariants, independent reference, seeded defects; timeouts and unasked atoms map to BLOCKED, never PASS.
12. **L12 Extraction ops.** Derive the budget chain; classify empty/truncated by finish reason; log per-call provenance; missing usage is unreported, not 0.
13. **L13 Distillation.** Working context is not durable memory; promote on an explicit decision; record decision, why, evidence, rejected alternative.
14. **L14 Backend and migration.** Fewest native moving parts; prove version/capabilities; reviewed one-off additive migrations on a copy; count oracle before publish.

## What the corpus can and cannot support

It can support: the architecture and identity decisions, the destructive-rebuild finding, the corpus-admission failure, the evaluation-integrity failure modes, the budget/timeout diagnosis, and the distillation practice.

It cannot support: that the curated PL ontology beats a generic one, that the metric panel beats strict F1, that 15 papers are enough, that the evidence graph improves downstream answers, or that itemID survives a Zotero restore. Those are the next experiments, not conclusions.

## Provenance

- Threads: Nowledge Mem, ids and message indexes in corpus.md and the raw JSON.
- Code: /Users/emliunix/Documents/paperfs.rs/paper-graph at f1e46d9.
- Analysis: analysis/llm-document-graph-research.md, analysis/deepseek-ontology-critique.md.
- Experiment records: experiments/ontology-extraction/.
- Audit data: round-2-rescan.json, round-3-audit.json.

Paths in this folder are relative to the paper-graph repository unless they name another project (doc-verify, bub_sf).