# Roadmap — Paper Knowledge System

**Status:** Decided · Observed · Planned (baseline `paper-graph` at `f1e46d9`, 2026-11-19).
**Owns:** Paper claim scope and feature roadmap ([owner-map.md](../knowledge-system/docs/owner-map.md) row 32; [CONSTITUTION.md](../knowledge-system/CONSTITUTION.md) Invariant 7).
**Checks:** `judgment (human)` by `human (author)` for every paper claim and scope expansion; milestone promotions require passing checks in [EXPERIMENTS.md](EXPERIMENTS.md) and [EVALUATION.md](EVALUATION.md).

Task-level scheduling lives in `worklog/2026-11-19/todo.md`; observed repository status lives in [STATUS.md](../knowledge-system/STATUS.md); open research gaps live in [open-questions.md](../knowledge-system/open-questions.md).

## Current State and Defensible Paper Claim Boundary

- **Observed baseline:** `paper-graph` ships a working local proof of work over 324 Zotero records in HawDB 0.34: read-only SQLite backup import, SQL metadata + 1024-d `bge-m3` vectors, six metadata graph labels (`Work`, `Person`, `Collection`, `Tag`, `Venue`, `Attachment`), BM25/vector/RRF-hybrid search, CLI, and loopback `axum` UI ([README.md](../README.md) lines 14–24, 82–84). Fourteen design lessons (`L01`–`L14`) were distilled across three audit rounds from 18 threads ([method.md](../knowledge-system/method.md) lines 1–67), and a 15-paper PL corpus passed manual `%PDF-` signature checks ([gates.md](../experiments/ontology-extraction/gates.md) lines 11–14).
- **Specified / not yet observed:** No PDF has been parsed, no `Chunk`/`Concept`/`Claim`/`EvidenceSpan` table exists in `src/`, no structural `check_*` binary exists, no gold annotations exist, and no extraction arm has run ([STATUS.md](../knowledge-system/STATUS.md) lines 20–32, 42–50).

Per [CONSTITUTION.md](../knowledge-system/CONSTITUTION.md) Invariant 7, **pre-verdict paper claims are strictly restricted to:**
1. The local-first architecture separating canonical Zotero storage, rebuildable HawDB metadata indexes, and isolated derived extraction/review stores ([ARCHITECTURE.md](ARCHITECTURE.md)).
2. The pre-registered evaluation and review protocol for ontology specificity, atomic quote-plus-locator evidence, and oracle integrity ([EVALUATION.md](EVALUATION.md), [REVIEW.md](REVIEW.md)).
3. Operational findings already observed in engineering runs: destructive sync wipe hazards, dangling Zotero attachment rows, metadata-only retrieval ceilings (117/324 abstracts; generic `"PDF"` title ranking), embedding batch limits, call-budget/timeout failure compositions, and zero-case/oracle-leakage evaluation traps ([open-questions.md](../knowledge-system/open-questions.md) lines 63–70).

No document or draft may claim ontology superiority, extraction accuracy, downstream QA gains, or durable re-import survival until the corresponding held-out gate or drill passes and `human (author)` authorizes the claim ([open-questions.md](../knowledge-system/open-questions.md) lines 72–79). No extraction UI work is scheduled before evidence quality is proven.

## Horizon 1 — Now: Structural Guards and Evidence Substrate (W2, W4–W6, T0.6)

Every item is gated by an executable artifact rather than prose completion:

- **Model reliability freeze (T0.6):** Run `N=5` structured-output trials per candidate route on three PL excerpts; freeze extraction model, decoding, and `p95` timeout/retry policy (`_runs/2026-11-19/t06-reliability.json`, [OPERATIONS.md](OPERATIONS.md)).
- **Praxis normalization (W2):** Encode `Episode`, `Lesson`, and `Check` in `PRAXIS.md` and `lessons.jsonl` with verified thread quotes (`check_praxis_schema`).
- **Structural check suite (W4):** Implement all registered `check_*` guards in CI, each verified against a red-turning seeded defect ([owner-map.md](../knowledge-system/docs/owner-map.md) lines 12–32).
- **Executable admission and PDF parse substrate (W6):** Emit schema-validated `admission.json` (15/15 byte + `%PDF-` + SHA-256), `pages.jsonl`, and `chunks.jsonl` on the 4 development PDFs; measure exact quote and character-offset locator recovery ([PIPELINE.md](PIPELINE.md), `check_corpus_admission`, `check_evidence_locator`).
- **Docs-effectiveness benchmark (W5):** Score 15 decision scenarios against a frozen docs manifest (`>=90%` correct decisions, `0` forbidden actions).

## Horizon 2 — Next: Ontology-Specificity Pilot, Held-Out Verdict, and HawDB Drills (W7–W9)

- **Development pilot (W7):** Resolve open human decisions on the 5-relation vocabulary cut ([ontology.md](../knowledge-system/ontology.md)) and single-axis resolver/panel ([EVALUATION.md](EVALUATION.md)); freeze 20 competency questions (`check_question_lint`), pilot two-annotator gold agreement (`alpha >= .80`), implement the deterministic scorer + seeded-defect gate, and run `U2`/`S2`/`P2` on the 4 development papers.
- **Held-out run and downstream QA (W8):** Execute `U2`/`S2`/`P2` once on the 11 held-out test papers under frozen hashes and approved spend (cap `<= US$25`, [REVIEW.md](REVIEW.md)); run blind human review and 20-question compositional QA against the metadata-hybrid baseline; publish `round-001-verdict.md` (explicitly permitting an **inconclusive** verdict if bootstrap intervals or gates fail).
- **Identity, rebuild, and HawDB utilization (W9):** Prove `zotero_key`/DOI claim survival across forced `itemID` renumbering (`check_corpus_identity`), fail-closed derived-DB sync (`check_sync_count_guard`), snapshot-pinned COW branches, exact-vs-ANN scaling (1K/10K/100K chunks), Cypher path plans, read payload budgets, and idempotent staging-to-canonical promotion ([DATA_MODEL.md](DATA_MODEL.md)).

## Horizon 3 — Later: Post-Verdict Extensions (W10+)

Contingent on positive W8/W9 gates and new frozen competency questions:

- **Paper framing and pre-mortem (W10):** Finalize paper outline, empirical/negative results, and threat-to-validity ledger from recorded W8/W9 artifacts; synchronize `STATUS.md` and `README.md`.
- **Earned schema and modality expansion:** Add `CITES`, `EXTENDS_*`, `GENERALIZES`, `Experiment`, or `BenchmarkSuite` only when blocked by a frozen competency question ([GUIDELINES.md](../knowledge-system/GUIDELINES.md) `GP-05`). Evaluate `glm-ocr` fallback on scanned/math-heavy pages, non-destructive metadata upserts, review-queue UI surfaces, and Lean 4 formalization links only after prose evidence gates hold.
