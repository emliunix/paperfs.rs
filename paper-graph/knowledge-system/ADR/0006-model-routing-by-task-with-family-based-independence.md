# 0006 — Model routing by task with family-based independence

- **Status:** accepted
- **Date:** 2026-11-19
- **Owner:** emliunix

## Context

Extraction, review, scoring, and adjudication have different failure modes. `paper-graph/knowledge-system/CONSTITUTION.md` requires human ownership of gold, verdicts, spend, and paper claims, and identifies correlated blind spots when reviewer and producer come from the same model family. L08 and L10 warn against outcome-presupposing evaluation and oracle tuning; L11 and L12 require explicit failure and usage provenance. Routing therefore cannot be an undocumented convenience or a second chance to obtain a desired answer. No route is assumed superior without a preregistered evaluation, and this decision asserts no measured quality or cost difference.

## Decision

We will configure model routes by task, recording provider, model, family, version or snapshot when available, decoding settings, prompt hash, run identifier, and per-call outcome for every invocation. Extraction, independent review, and scoring routes will be named separately in the experiment manifest and frozen before test predictions. A review counts as model-independent only when its reviewer belongs to a different declared family from the producer; a human must adjudicate final verdicts and every same-family review. Route selection may use task requirements and prior preregistered evidence, never inspection of the current expected answer or desired verdict. Hosted calls require human spend approval and the budget gates owned by evaluation. Missing usage remains `unreported`, and failed or exhausted calls remain blocked rather than silently rerouted until one passes.

## Alternatives rejected

- Use one model for production, review, and scoring — rejected: shared family blind spots can make agreement look like validation.
- Label same-family review independent — rejected: a separate prompt or instance does not remove correlated assumptions.
- Change routes after seeing difficult cases — rejected: adaptive routing can tune the oracle and invalidate the preregistered axis (L08, L10).
- Automatically retry across models until agreement — rejected: failures disappear and agreement becomes an outcome-selected artifact.

## Consequences

- Review provenance will distinguish corroboration from genuinely cross-family review, while humans retain adjudication authority.
- Operators must maintain a family map and task-route configuration; unavailable cross-family capacity can block a verdict.
- Comparative routing claims require their own frozen evaluation and cannot be inferred from successful pipeline completion.
- Check pointers: `check_model_route` and `check_reviewer_independence` (planned) will validate route manifests, provenance, and family separation; `check_gold_diff`, budget preflight, spend, and final verdict retain their existing human gates.
