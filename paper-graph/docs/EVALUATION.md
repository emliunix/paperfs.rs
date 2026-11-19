# Evaluation — Paper Knowledge System

**Status:** Specified · Contested · Transferred · Unexecuted (baseline `paper-graph` at `f1e46d9`, 2026-11-19).
**Owns:** Evaluation protocol and evaluation integrity ([owner-map.md](../knowledge-system/docs/owner-map.md) rows 24–25; [CONSTITUTION.md](../knowledge-system/CONSTITUTION.md) Invariant 6; [ADR 0005](../knowledge-system/ADR/0005-evaluation-first-with-one-preregistered-axis.md)).
**Checks:** `check_eval_manifest` (planned, W4); `check_question_lint` (planned, W4).

Higher invariants live in [CONSTITUTION.md](../knowledge-system/CONSTITUTION.md) and status tags in [method.md](../knowledge-system/method.md#governance). Round-specific hypotheses and the active gate ledger belong to [EXPERIMENTS.md](EXPERIMENTS.md); call budgets belong to [OPERATIONS.md](OPERATIONS.md); gold authority and review verdicts belong to [REVIEW.md](REVIEW.md); historical design discussion lives in [methodology.md](../knowledge-system/methodology.md).

## Current State: Observed vs Specified

- **Observed:** Zero extraction or downstream QA evaluations have run ([gates.md](../experiments/ontology-extraction/gates.md) lines 16–26). `tests/experiment_corpus.rs` (lines 6–68) verifies only the static corpus layout (4 development, 11 held-out test across four PL domains in [round-001-corpus.json](../experiments/ontology-extraction/round-001-corpus.json)) and the literal string `"pdf_verified_gold_pending"`. Four exploratory hybrid search queries ran manually as non-regression baselines ([gates.md](../experiments/ontology-extraction/gates.md) lines 28–39).
- **Specified (unexecuted):** Pre-registered manifests, pilot arms `U2`/`S2`/`P2`, structural-versus-semantic scorer split, 19-category seeded-defect suite, and metric panel ([round-001-plan.md](../experiments/ontology-extraction/round-001-plan.md), [round-001-rubric.md](../experiments/ontology-extraction/round-001-rubric.md)).
- **Transferred (unexecuted here):** Three-layer evaluator differential validation from `doc-verify` ([methodology.md](../knowledge-system/methodology.md) lines 95–106; [lessons.md](../knowledge-system/lessons.md) `L10`, `L11`).

## Pilot Arms and Frozen Protocol

Per [CONSTITUTION.md](../knowledge-system/CONSTITUTION.md) Invariant 6, every evaluation freezes an immutable manifest (`manifest.json`) before opening predictions: corpus and chunk hashes, competency questions, prompt and schema hashes, single extraction model route ([models.md](../knowledge-system/models.md)), decoding parameters, call-budget preflight proof ([OPERATIONS.md](OPERATIONS.md)), and scorer version.

The specified pilot compares three unreviewed extraction arms on identical parsed chunks and call budgets ([round-001-plan.md](../experiments/ontology-extraction/round-001-plan.md) lines 23–35):

- **`U2`** — unconstrained free-form concept categories and relation labels inside the shared outer JSON envelope.
- **`S2`** — generic scholarly concept kinds and predicates (`MENTIONS`, `USES`, own-contribution, extension, generalization, dependency) with no PL vocabulary.
- **`P2`** — `S2` extended with the versioned PL concept dictionary, definitions, domain/range rules ([ontology.md](../knowledge-system/ontology.md)), negative examples, and curated aliases.

All model outputs remain `unreviewed`; a separate `P2-reviewed` condition evaluates the human-in-the-loop workflow and is never compared against unreviewed arms to claim raw model superiority ([round-001-plan.md](../experiments/ontology-extraction/round-001-plan.md) lines 31–34). Development (4 papers) permits at most one logged prompt revision and one logged chunk revision before the 11 held-out papers are opened once ([round-001-plan.md](../experiments/ontology-extraction/round-001-plan.md) lines 56–60, 78–81).

## Structural vs Semantic Check Split

Evaluation separates shape verification from meaning verification and reports them on distinct tracks ([round-001-rubric.md](../experiments/ontology-extraction/round-001-rubric.md) lines 8–22, 77–99):

1. **Structural checks (deterministic, pre-unblinding):** JSON outer-envelope parse rate on first pass and after bounded retry, schema field completeness, allowed ontology predicate and endpoint kind conformance ([ontology.md](../knowledge-system/ontology.md)), exact verbatim substring presence in the cited raw chunk/page, attachment/chunk/page/offset locator resolution ([DATA_MODEL.md](DATA_MODEL.md)), and overlapping-span deduplication. Passing structural checks proves well-formedness and coordinate resolution only, never factual support.
2. **Semantic checks (blinded gold and human rubric):** Canonical concept linking and pairwise/B-cubed clustering, graded role accuracy (`MENTIONS` vs `USES` vs non-priority `INTRODUCES`), relation direction confusion matrices, atomic claim correctness (`0`/`1`/`2`) with premise and qualifier preservation, whole-evidence-set sufficiency (`0`/`1`/`2`, where canonical acceptance requires `2`), downstream compositional QA answer/citation scores, and abstention on the four unanswerable questions ([round-001-rubric.md](../experiments/ontology-extraction/round-001-rubric.md) lines 23–114; human authority in [REVIEW.md](REVIEW.md)).

## OPEN HUMAN DECISION: Axis, Resolver, and Headline Metric

**Status:** Contested — **OPEN HUMAN DECISION** ([owner-map.md](../knowledge-system/docs/owner-map.md) line 62; [methodology.md](../knowledge-system/methodology.md) lines 38–48, 62; [lessons.md](../knowledge-system/lessons.md) `L08`, `L09`). **Decider:** `human (evaluation owner)` before the evaluation manifest freezes. Agents must not choose among these options:

1. **Manipulated axis and resolver:** [round-001-plan.md](../experiments/ontology-extraction/round-001-plan.md) (lines 18–30) declares ontology constraint the sole axis while assigning three different merge rules (`U2` exact normalized-name merge, `S2` generic known-alias rules, `P2` curated PL aliases). The human owner must decide whether to share one resolver across `U2`/`S2`/`P2` where only the alias table varies, or redefine the axis as *vocabulary plus curated aliases* and add an arm isolating resolution.
2. **Headline criterion:** [round-001-rubric.md](../experiments/ontology-extraction/round-001-rubric.md) (lines 3–6, 55–57) specifies a co-headline metric panel (canonical concept F1, per-relation macro-F1, high-risk violation rate, coverage, evidence validity, downstream QA, and review burden) with strict evidenced relation F1 as diagnostic, whereas [llm-document-graph-research.md](../analysis/llm-document-graph-research.md) (lines 23, 643) retains strict evidenced relation F1 as the primary headline. The human owner must ratify the panel or state explicit single-F1 exception rules before scoring.

## Evaluation Integrity and Scorer Validation

`check_eval_manifest` and `check_question_lint` enforce harness and oracle integrity ([CONSTITUTION.md](../knowledge-system/CONSTITUTION.md) Invariant 6; [GUIDELINES.md](../knowledge-system/GUIDELINES.md) `GP-02`):

- **Execution accounting:** Every run records `executed`, `skipped`, and `failed` counts verified against an independent case count; `executed != corpus_size` or any skipped case exits non-zero.
- **Failure mapping:** Scorer crashes, timeouts, truncated generations, budget exhaustion, and unasked atoms map to `BLOCKED` (or `INVALID` for harness faults), never `PASS` or empty-graph agreement. Reserve `NEEDS_REVIEW` solely for complete executions requiring human semantic interpretation.
- **Evaluator validation:** Before test scoring, the deterministic scorer must pass differential checks against a naive reference implementation, invariant property tests, and the 19-category seeded-defect suite ([round-001-rubric.md](../experiments/ontology-extraction/round-001-rubric.md) lines 149, 156–178). Competency questions and schemas must pass `check_question_lint` prior to gold freeze.
