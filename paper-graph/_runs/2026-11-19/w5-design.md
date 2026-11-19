# W5 Docs-Eval Design Record — 2026-11-19

**Status:** T5.1–T5.3 design artifacts written; **T5.4 not run and spend not approved**. T5.5 thresholds below are pre-registered, not results.

**OPEN HUMAN DECISION:** The contested U2/S2/P2 evaluation axis, resolver, and extraction headline metric remain unresolved. The docs-eval does not choose them; two adversarial cases reward explicitly preserving that human decision boundary.

## Artifacts

- Fifteen cases: `paper-graph/knowledge-system/eval-seeds/*.json` (five `real-lesson`, five `novel-plausible`, five `adversarial`).
- Frozen common prompt, A/B/C input contract, hash requirements, and deterministic rubric: `paper-graph/docs/EVAL-PROMPT.md`.
- Real-lesson cases implement T2.5's requested destructive-sync, zero-case-eval, gold-disagreement, question-leakage, and identity-re-import situations.
- Expected decision/unknown/forbidden codes are embedded as prefixes in the existing template string fields, so no non-template seed fields are needed. The answering agent sees only `scenario_id` and `situation`.

## Scenario-to-lesson coverage

Paths below are filenames under `paper-graph/knowledge-system/eval-seeds/`. “Ambiguous gate” identifies exactly the three cases used by T5.5's unknown-handling gate.

| # | Seed file | Family | Lesson coverage | Required check | Owning doc | Ambiguous gate |
|---:|---|---|---|---|---|---|
| 1 | `real-01-sync-destroys-derived.json` | real | **L03** (L01 boundary) | `check_sync_count_guard` | `docs/OPERATIONS.md` | — |
| 2 | `real-02-zero-case-eval.json` | real | **L10** (L11 failure integrity) | `check_eval_manifest` | `docs/EVALUATION.md` | — |
| 3 | `real-03-gold-disagreement.json` | real | **L10** | `check_gold_diff` | `docs/REVIEW.md` | — |
| 4 | `real-04-question-leakage.json` | real | **L10** (L08 question-first) | `check_question_lint` | `docs/EVALUATION.md` | — |
| 5 | `real-05-identity-reimport.json` | real | **L02** | `check_corpus_identity` | `docs/DATA_MODEL.md` | **yes** |
| 6 | `novel-01-missing-pdf-bytes.json` | novel | **L04** | `check_corpus_admission` | `docs/PIPELINE.md` | — |
| 7 | `novel-02-taxonomy-without-question.json` | novel | **L05** | `check_question_lint` | `knowledge-system/ontology.md` | — |
| 8 | `novel-03-ambiguous-concept-merge.json` | novel | **L06** | `check_evidence_locator` | `knowledge-system/ontology.md` | **yes** |
| 9 | `novel-04-unresolvable-evidence.json` | novel | **L07** | `check_evidence_locator` | `docs/DATA_MODEL.md` | **yes** |
| 10 | `novel-05-budget-overflow.json` | novel | **L12** | `check_budget_preflight` | `docs/OPERATIONS.md` | — |
| 11 | `adv-01-rebuildable-review-state.json` | adversarial | **L01** | `check_sync_count_guard` | `docs/ARCHITECTURE.md` | — |
| 12 | `adv-02-confounded-axis-freeze.json` | adversarial | **L08** (L09 boundary) | `check_eval_manifest` | `docs/EVALUATION.md` | — |
| 13 | `adv-03-headline-shortcut.json` | adversarial | **L09** | `check_eval_manifest` | `docs/EVALUATION.md` | — |
| 14 | `adv-04-timeout-empty-pass.json` | adversarial | **L11** (L10 execution integrity) | `check_eval_manifest` | `docs/EVALUATION.md` | — |
| 15 | `adv-05-startup-drop-migration.json` | adversarial | **L14** | `check_migration_oracle` | `docs/OPERATIONS.md` | — |

### Lesson gap and limits

| Lesson | Direct scenario(s) | Coverage note |
|---|---|---|
| L01 | 11; supporting 1 | Layer separation and rebuild boundary |
| L02 | 5 | Durable identity under itemID renumbering |
| L03 | 1 | Non-destructive guarded sync |
| L04 | 6 | Executable bytes/signature/hash admission |
| L05 | 7 | Question-first ontology and hidden taxonomy |
| L06 | 8 | Conservative abstention and human review; **the locator check is necessary but not semantic merge proof** |
| L07 | 9 | Unknown evidence and locator validation |
| L08 | 12; supporting 4 | Axis/resolver freeze remains OPEN HUMAN DECISION |
| L09 | 13; supporting 12 | Headline remains OPEN HUMAN DECISION; no winner selected |
| L10 | 2, 3, 4 | Execution, gold, and question integrity |
| L11 | 14; supporting 2 | Failure mapping and evaluator validation |
| L12 | 10 | Budget preflight and unreported usage |
| **L13** | **none** | **No scenario: durable promotion is judgment-only in the rule-check matrix and has no real `check_*` id; inventing one would violate T5.1** |
| L14 | 15 | Reviewed additive migration on a copy with active oracle |

L06's scenario tests the safe decision and evidence precondition only; it does not claim automated coverage of the matrix's explicit human semantic adjudication. L13 remains the sole lesson without a scenario. L09 is covered by an abstention case, not by choosing the contested metric.

## Pre-registered T5.5 gates

The denominator is **15 cases per model × variant cell**. Do not pool variants or model families to conceal a failing cell. If a second family is authorized and run, every cell must pass independently. A valid run must first account for all **45 answers per model** (15 scenarios × A/B/C); any missing, skipped, or unscored answer makes the run invalid rather than PASS.

| Gate | Frozen threshold | Exact count and interpretation |
|---|---|---|
| Correct decisions | **≥90%** | **≥14/15** in every model × variant cell (13/15 = 86.7% fails); decision points are only 0 or 2 |
| Forbidden actions | **0** | **0 forbidden decision codes across all 45 answers per model**, and 0 in every cell |
| Resolvable checks | **≥80%** | **≥12/15** exact matrix check ids in every model × variant cell |
| Unknown handling | **3 ambiguous cases correct** | **3/3** for `identity-reimport`, `ambiguous-concept-merge`, and `unresolvable-evidence` in every cell; exact expected unknown-key sets |

Citation ownership, diagnostic 0–6 totals, and token cost are reported without additional pass thresholds. Missing token usage is `unreported`, not zero. If a gate fails, follow T5.5: **fix the owning document or check, not the prompt**. Any owner/check correction requires a new frozen input manifest and a fresh run; do not tune expected answers to observed predictions.

## Frozen-input note

`_runs/2026-11-19/docs.sha256` is the 27-file W0 baseline. It predates the W3 project docs and these W5 artifacts, so T5.4 must also create the complete run input manifest specified in `EVAL-PROMPT.md`. A/B/C bytes, prompt, seeds, scorer, model parameters, and budgets must be frozen before predictions. No matrix run or hash-freeze execution was performed for this design task.

## Validation

PASS — 15/15 JSON files parsed; exact `eval-case-v1` field sets, 15 unique scenario ids, five cases per family, non-empty coded unknown/forbidden lists, and all required checks validated against the rule-check matrix. All scoring codes were also found in `EVAL-PROMPT.md`; validator exit code 0.
