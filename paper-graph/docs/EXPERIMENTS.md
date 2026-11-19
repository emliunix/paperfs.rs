# Experiments — Paper Knowledge System

**Status:** Observed · Decided · Specified · Contested (baseline `paper-graph` at `f1e46d9`, 2026-11-19; status vocabulary in [method.md](../knowledge-system/method.md#status-vocabulary)).
**Owns:** Experiment round hypotheses, `Arm` / `Variant` / `Gate` definitions, reconciled gate ledger state, and experiment artifact paths ([owner-map.md](../knowledge-system/docs/owner-map.md)).
**Governed by:** [CONSTITUTION.md Invariant 6](../knowledge-system/CONSTITUTION.md#6-evaluation-is-pre-registered-executed-and-recorded), [Invariant 7](../knowledge-system/CONSTITUTION.md#7-human-owns-gold-verdict-spend-and-the-claim), and [ADR 0005](../knowledge-system/ADR/0005-evaluation-first-with-one-preregistered-axis.md).
**Check pointer:** `check_eval_manifest` (`Specified`, planned in W4). Metric panel formulas, single-resolver rules, and numeric gate thresholds belong to `docs/EVALUATION.md`.

## Experiment Vocabulary Owned Here

- **Arm:** A pre-registered experimental condition compared under one frozen protocol (`U2`, `S2`, `P2`), distinct from a **Variant**, which is a concrete version of a prompt, document set, or configuration (such as docs-eval `A`/`B`/`C`) that need not define an ontology axis ([GLOSSARY.md](../knowledge-system/GLOSSARY.md)).
- **Gate:** A decision boundary that permits progression, blocks execution, or marks a round `inconclusive` from named `check_*` results and human judgments; a `no` or stale row is never a pass.

## Round 001 (`ontology-extraction`) — Hypotheses and Arms (`Decided · Specified · Contested`)

- **Stratified corpus (`Observed · Decided`):** [`experiments/ontology-extraction/round-001-corpus.json`](../experiments/ontology-extraction/round-001-corpus.json) fixes 15 Zotero works across `type_inference`, `recursion_schemes`, `effects_control`, and `dependent_lean`: `4` development papers (`3154`, `713`, `49`, `6`) and `11` disjoint held-out test papers (`2777`, `3229`, `41`, `18`, `2314`, `1483`, `675`, `1490`, `1348`, `637`, `774`). Four candidates without local PDFs (`60`, `47`, `5`, `8`) were replaced on 2026-11-18 ([decisions.md](../experiments/ontology-extraction/decisions.md)).
- **Arms (`Specified · Contested`):** [`round-001-plan.md`](../experiments/ontology-extraction/round-001-plan.md) compares `U2` (free-form labels), `S2` (generic scholarly schema), and `P2` (`S2` plus versioned PL dictionary, domain/range rules, and negative examples) in a fixed two-pass envelope, plus human-adjudicated `P2-reviewed` for downstream QA. Two **Contested** design conflicts remain open ([owner-map.md](../knowledge-system/docs/owner-map.md)): (1) `round-001-plan.md` varies alias resolution across `U2`/`S2`/`P2` rather than sharing one resolver (`human (evaluation owner)`, `docs/EVALUATION.md`); (2) [ontology.md](../knowledge-system/ontology.md) drafts 17 relations vs the 5-relation pilot target (`human (ontology owner)`).
- **Claims under test and falsifiers (`Specified`, [grounding.md](../experiments/ontology-extraction/grounding.md)):**
  - `C1` (*Domain grounding*): `P2` beats `S2` on held-out canonical concept F1 and per-relation macro-F1 without exceeding high-risk semantic violation gates.
  - `C2` (*Evidence trust*): Deterministic quote/locator validation ([PIPELINE.md](PIPELINE.md)) curbs fabricated coordinates while human review remains necessary for semantic sufficiency.
  - `C3` (*Compositional QA*): `P2-reviewed` improves evidence-supported compositional answers over metadata/abstract hybrid search without regressing simple retrieval controls.
  - `C4` (*Conservative resolution*): Alias-aware clustering beats exact-name merging without false merges across distinct PL notions (e.g., free vs freer monads).

## Reconciled Gate Ledger (`Observed`, 2026-11-19)

Reconciling [`experiments/ontology-extraction/gates.md`](../experiments/ontology-extraction/gates.md) with [STATUS.md](../knowledge-system/STATUS.md): **zero PDFs have been parsed, zero gold annotations exist, and zero extraction runs or spend have occurred.**

- **Completed (`yes`, `Observed`):** Primary survey ([llm-document-graph-research.md](../analysis/llm-document-graph-research.md)); secondary critique ([deepseek-ontology-critique.md](../analysis/deepseek-ontology-critique.md), used as uncited critique only); Zotero attachment lookup, dangling-row exclusion, and manual 15/15 `%PDF-` verification ([round-001-corpus.json](../experiments/ontology-extraction/round-001-corpus.json)); four exploratory `/api/search` hybrid probes returning rank-1 works `3154`, `2314`, `1490`, `774` (non-regression controls only, [gates.md](../experiments/ontology-extraction/gates.md)); two-token reachability smoke test across seven subagent routes ([`_runs/2026-11-19/model-smoke.json`](../_runs/2026-11-19/model-smoke.json), not an extraction freeze).
- **Incomplete / Blocking (`no`):** Executable corpus-structure/admission check (`gates.md` cited `uv run pytest`, which is stale after the Rust port; `cargo test -p paper-graph --test experiment_corpus` checks split counts and static status string only); independent plan review (`round-001-review.md` absent); PDF parse smoke test (`parse-report.md` absent); reviewed v0 annotation manual; human gold annotations and agreement; reviewed 20-question `questions.json`; executable seeded defects ([round-001-rubric.md](../experiments/ontology-extraction/round-001-rubric.md)); T0.6 structured-output model freeze and price; `U2`/`S2`/`P2` runs; deterministic scorer; blind human scoring; `round-001-verdict.md` (`inconclusive` allowed).

## Artifact and Result Paths

All paths resolve from `paper-graph/`:

- **Observed specifications and baseline logs:** `experiments/ontology-extraction/{README.md,grounding.md,decisions.md,gates.md,round-001-corpus.json,round-001-plan.md,round-001-rubric.md}` and `_runs/2026-11-19/{docs.sha256,model-smoke.json,spine-audit.md}`.
- **Specified run outputs (absent):** `_runs/<date>/pilot/{manifest.json,raw/,metrics.json,report.md}`, `_runs/<date>/docs-eval/{config.json,raw/,report.md}`, and `experiments/ontology-extraction/{round-001-review.md,questions.json,round-001-verdict.md}`.
