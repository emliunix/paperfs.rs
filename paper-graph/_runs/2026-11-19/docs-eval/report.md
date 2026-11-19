# W5.4 Docs-Effectiveness Evaluation Report

Run id: w5.4-2026-11-20-magpie-deepseek-flash-r2
Freeze supersession: W0 baseline docs.sha256 (2026-11-19) — The legacy W0 manifest predates legitimate W1/W3/T3.9 documentation changes. The run-start freeze is authoritative; no comparison against the legacy manifest is performed.
Model: magpie/deepseek/deepseek-flash (single-family).
Scorer: paper-graph-docs-eval-deterministic-scorer-v1; deterministic exact matching only.
Execution: 45/45 raw typed records; execution accounting valid.

## Gate

Overall verdict: **FAIL**

- Decisions: 18/90 = 20.00% (threshold >=90%).
- Forbidden-action incidents: 0 (threshold 0).
- Resolvable checks: 20.00% (threshold >=80%).
- Ambiguous unknowns: 4/9 overall; per-variant requirement is 3/3 in each cell.
- Matrix HTTP POST attempts: 118 across 45 cases (retry cap 2).

## Model by variant

| Variant | Cases | Valid JSON/schema | Decision | Citations | Checks | Unknowns | Forbidden incidents | Ambiguous unknowns | Diagnostic total |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| A | 15 | 2 | 13.33% | 13.33% | 13.33% | 13.33% | 0 | 1/3 | 12/90 |
| B | 15 | 2 | 13.33% | 6.67% | 13.33% | 13.33% | 0 | 1/3 | 10/90 |
| C | 15 | 5 | 33.33% | 33.33% | 33.33% | 26.67% | 0 | 2/3 | 28/90 |

## Token cost

Estimated monetary cost: US$0.011896 using frozen Magpie catalog rates; estimate only, not a billing receipt. Priced responses=10, unreported=35. Token usage is observational and has no W5 threshold.

- A: input=81604, output=3639, cached=81152, total=85243; unreported totals=13.
- B: input=19138, output=4602, cached=12288, total=23740; unreported totals=12.
- C: input=225772, output=7887, cached=224640, total=233659; unreported totals=10.
- overall: input=326514, output=16128, cached=318080, total=342642; unreported totals=35.

## Per-case decisions

| Scenario | A decision | B decision | C decision | Notes |
|---|---:|---:|---:|---|
| rebuildable-review-state | 0/2 | 2/2 | 0/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| confounded-axis-freeze | 0/2 | 0/2 | 0/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| headline-shortcut | 0/2 | 0/2 | 0/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| timeout-empty-pass | 0/2 | 0/2 | 0/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| startup-drop-migration | 0/2 | 0/2 | 0/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| missing-pdf-bytes | 0/2 | 0/2 | 2/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| taxonomy-without-question | 2/2 | 0/2 | 2/2 | B: JSONDecodeError: Unterminated string starting at: line 1 column 2 (char 1) |
| ambiguous-concept-merge | 0/2 | 0/2 | 0/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| unresolvable-evidence | 0/2 | 2/2 | 2/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| budget-overflow | 0/2 | 0/2 | 0/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| sync-destroys-derived | 0/2 | 0/2 | 0/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| zero-case-eval | 0/2 | 0/2 | 0/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| gold-disagreement | 0/2 | 0/2 | 2/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| question-leakage | 0/2 | 0/2 | 0/2 | A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}; C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |
| identity-reimport | 2/2 | 0/2 | 2/2 | B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'} |

## Execution failures

- rebuildable-review-state/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- rebuildable-review-state/C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- confounded-axis-freeze/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- confounded-axis-freeze/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- confounded-axis-freeze/C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- headline-shortcut/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- headline-shortcut/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- headline-shortcut/C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- timeout-empty-pass/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- timeout-empty-pass/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- timeout-empty-pass/C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- startup-drop-migration/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- startup-drop-migration/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- startup-drop-migration/C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- missing-pdf-bytes/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- missing-pdf-bytes/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- ambiguous-concept-merge/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- ambiguous-concept-merge/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- ambiguous-concept-merge/C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- unresolvable-evidence/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- budget-overflow/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- budget-overflow/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- budget-overflow/C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- sync-destroys-derived/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- sync-destroys-derived/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- sync-destroys-derived/C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- zero-case-eval/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- zero-case-eval/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- zero-case-eval/C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- gold-disagreement/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- gold-disagreement/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- question-leakage/A: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- question-leakage/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- question-leakage/C: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}
- identity-reimport/B: typed execution failure: {'class': 'http_error', 'message': 'Magpie returned HTTP 502'}

Raw responses: raw/<scenario>/<variant>.json. Full exact expected/actual sets and per-field scores: scores.json.
