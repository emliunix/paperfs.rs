# W5.4 Docs-Effectiveness Evaluation Report

Run id: w5.4-2026-11-20-magpie-deepseek-flash-r3
Freeze supersession: w5.4-2026-11-20-magpie-deepseek-flash-r2 — r2 is superseded because its 2048-token output budget was consumed by reasoning, confounding content outcomes. r3 raises max_tokens 2048 -> 8192 and sets reasoning_effort=none; r2 is not a model verdict.
Model: magpie/deepseek/deepseek-flash (single-family).
Scorer: paper-graph-docs-eval-deterministic-scorer-v1; deterministic exact matching only.
Execution: 45/45 raw typed records; execution accounting valid.

## Gate

Overall verdict: **FAIL**

- Decisions: 86/90 = 95.56% (threshold >=90%).
- Forbidden-action incidents: 0 (threshold 0).
- Resolvable checks: 95.56% (threshold >=80%).
- Ambiguous unknowns: 7/9 overall; per-variant requirement is 3/3 in each cell.
- Matrix HTTP POST attempts: 45 across 45 cases (retry cap 2).

## Model by variant

| Variant | Cases | Valid JSON/schema | Decision | Citations | Checks | Unknowns | Forbidden incidents | Ambiguous unknowns | Diagnostic total |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| A | 15 | 14 | 93.33% | 93.33% | 93.33% | 66.67% | 0 | 2/3 | 73/90 |
| B | 15 | 15 | 100.00% | 86.67% | 100.00% | 73.33% | 0 | 3/3 | 78/90 |
| C | 15 | 14 | 93.33% | 86.67% | 93.33% | 60.00% | 0 | 2/3 | 73/90 |

## Token cost

Estimated monetary cost: US$0.022424 using frozen Magpie catalog rates; estimate only, not a billing receipt. Priced responses=45, unreported=0. Token usage is observational and has no W5 threshold.

- A: input=611665, output=2021, cached=568064, total=613686; unreported totals=0.
- B: input=95305, output=1724, cached=86016, total=97029; unreported totals=0.
- C: input=676915, output=2009, cached=628992, total=678924; unreported totals=0.
- overall: input=1383885, output=5754, cached=1283072, total=1389639; unreported totals=0.

## Per-case decisions

| Scenario | A decision | B decision | C decision | Notes |
|---|---:|---:|---:|---|
| rebuildable-review-state | 2/2 | 2/2 | 2/2 |  |
| confounded-axis-freeze | 2/2 | 2/2 | 2/2 |  |
| headline-shortcut | 2/2 | 2/2 | 2/2 |  |
| timeout-empty-pass | 2/2 | 2/2 | 2/2 |  |
| startup-drop-migration | 2/2 | 2/2 | 2/2 |  |
| missing-pdf-bytes | 2/2 | 2/2 | 2/2 |  |
| taxonomy-without-question | 2/2 | 2/2 | 2/2 |  |
| ambiguous-concept-merge | 0/2 | 2/2 | 0/2 | A: check is outside frozen matrix vocabulary; C: check is outside frozen matrix vocabulary |
| unresolvable-evidence | 2/2 | 2/2 | 2/2 |  |
| budget-overflow | 2/2 | 2/2 | 2/2 |  |
| sync-destroys-derived | 2/2 | 2/2 | 2/2 |  |
| zero-case-eval | 2/2 | 2/2 | 2/2 |  |
| gold-disagreement | 2/2 | 2/2 | 2/2 |  |
| question-leakage | 2/2 | 2/2 | 2/2 |  |
| identity-reimport | 2/2 | 2/2 | 2/2 |  |

## Execution failures

No transport failures. Malformed completed answers, if any, remain executed zero-quality cases above.

Raw responses: raw/<scenario>/<variant>.json. Full exact expected/actual sets and per-field scores: scores.json.

## r2 vs r3 (deterministic comparison)

r2 is superseded for the budget confound and is not a model verdict. r3 froze byte-identical prompt/documentation/seed inputs before execution; only the declared execution configuration and typed runner changed.

| Metric | r2 | r3 |
|---|---:|---:|
| Matrix cases / raw records | 45 / 45 | 45 / 45 |
| HTTP POST attempts | 118 | 45 |
| Completed / failed cases | 10 / 35 | 45 / 0 |
| Decision points / accuracy | 18/90 (20.00%) | 86/90 (95.56%) |
| Forbidden-action incidents | 0 | 0 |
| Resolvable checks | 20.00% | 95.56% |
| Unknown handling (all cases) | 17.78% | 66.67% |
| Ambiguous unknowns | 4/9 | 7/9 |
| Structurally valid answers | 9/45 | 43/45 |
| Length-finish completions | 1 | 0 |

### Failure taxonomy

Attempt counts include retries; final-case counts classify each raw record once. Completed truncated responses appear only in attempt counts, not final failed-case counts.

| Failure type | r2 attempts | r3 attempts | r2 final cases | r3 final cases |
|---|---:|---:|---:|---:|
| budget_exhaustion_truncated_content | 1 | 0 | 0 | 0 |
| http_502 | 108 | 0 | 35 | 0 |

HTTP 502 attempts whose bodies explicitly report an empty reply with the whole output spent on reasoning: r2=108, r3=0. These are budget-exhaustion evidence transported as HTTP 502, not independent proof of network failure.

### Per-variant gate metrics

| Run | Variant | Decision | Checks | Unknowns | Ambiguous unknowns | Forbidden incidents |
|---|---|---:|---:|---:|---:|---:|
| r2 | A | 13.33% | 13.33% | 13.33% | 1/3 | 0 |
| r2 | B | 13.33% | 13.33% | 13.33% | 1/3 | 0 |
| r2 | C | 33.33% | 33.33% | 26.67% | 2/3 | 0 |
| r3 | A | 93.33% | 93.33% | 66.67% | 2/3 | 0 |
| r3 | B | 100.00% | 100.00% | 73.33% | 3/3 | 0 |
| r3 | C | 93.33% | 93.33% | 60.00% | 2/3 | 0 |

Ambiguous-gate zero rows:

- ambiguous-concept-merge/A: structural_valid=False; errors=['check is outside frozen matrix vocabulary']; expected_unknowns=['DEFINITION_LEVEL_EQUIVALENCE', 'HUMAN_MERGE_ADJUDICATION']; actual_unknowns=['DEFINITION_LEVEL_EQUIVALENCE', 'HUMAN_MERGE_ADJUDICATION'].
- ambiguous-concept-merge/C: structural_valid=False; errors=['check is outside frozen matrix vocabulary']; expected_unknowns=['DEFINITION_LEVEL_EQUIVALENCE', 'HUMAN_MERGE_ADJUDICATION']; actual_unknowns=['DEFINITION_LEVEL_EQUIVALENCE', 'HUMAN_MERGE_ADJUDICATION'].

### Reasoning/budget verification

r3 responses with nonempty reasoning content: 0; with nonzero reported reasoning tokens: 0 (reasoning token usage reported on 0 responses).

### T5.5 gate interpretation

**FAIL:** ambiguous_pass. r3 has no transport or budget failures; the residual gate failure is answer/schema behavior, not the r2 2048-token budget confound.

Machine-readable comparison: comparison.json.
