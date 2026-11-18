# 0005 — Evaluation first with one preregistered axis

- **Status:** accepted
- **Date:** 2026-11-19
- **Owner:** emliunix

## Context

Evaluation cannot rescue a question after predictions are visible. `paper-graph/knowledge-system/CONSTITUTION.md` cites shifting resolvers and single-score reporting (L08, L09), harnesses that executed no cases or converted exhaustion into success (L10, L11), and missing usage coerced to zero (L12). The same document states that no extraction experiment or gold annotations exist. Consequently, no effectiveness claim is available now; the immediate need is a protocol that makes execution, failure, and provenance visible before any comparison begins.

## Decision

We will preregister each experiment before opening test predictions: frozen questions and gold, corpus and input hashes, one primary comparison axis with its resolver, a secondary metric panel, and call-budget inequalities. One experiment varies only its declared axis; other routing, prompts, data, and resolver settings remain frozen. The run must prove `executed == corpus_size`. Skipped cases, unasked atoms, scorer failures, timeouts, and budget exhaustion will produce `BLOCKED`, never `PASS`, and partial execution will not support a verdict. We will retain per-call run identifiers, model and prompt provenance, outcomes, and usage; absent usage is `unreported`, not zero. The metric panel will expose complementary error modes without replacing the preregistered primary decision. Humans author gold and final verdicts, and model output cannot move an expected answer.

## Alternatives rejected

- Optimize one aggregate F1 headline — rejected: it can conceal high-risk errors and encourage post-hoc selection (L08, L09).
- Choose the resolver after seeing predictions — rejected: the oracle can be tuned to the observed outcome (L10).
- Report a best-effort partial run — rejected: unexecuted cases are not evidence (L10, L11).
- Store missing token or cost usage as zero — rejected: absence would be misrepresented as measured consumption (L12).

## Consequences

- Experiment results become reproducible comparisons with explicit failure states rather than retrospective narratives.
- Authors must freeze artifacts and budgets before execution, adding setup work and preventing opportunistic metric changes.
- Any incomplete harness blocks claims until corrected and rerun; documentation may describe protocol only, not gains.
- Check pointers: `check_eval_manifest`, `check_budget_preflight`, and `check_gold_diff` (planned) gate manifests, execution counts, failure semantics, and gold integrity; spend and verdict remain human approvals.
