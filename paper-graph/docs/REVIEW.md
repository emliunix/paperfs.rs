# Review — Paper Knowledge System

**Status:** Decided · Observed · Specified · Unexecuted (baseline `paper-graph` at `f1e46d9`, 2026-11-19).
**Owns:** Gold authority and expected-answer immutability; review verdicts, definition of done, and hosted spend approval ([owner-map.md](../knowledge-system/docs/owner-map.md) rows 28–29; [CONSTITUTION.md](../knowledge-system/CONSTITUTION.md) Invariant 7; [ADR 0006](../knowledge-system/ADR/0006-model-routing-by-task-with-family-based-independence.md)).
**Checks:** `check_gold_diff` (planned, W4); `judgment (human)` by `human (annotator)` and `human (adjudicator)`.

The cross-family reviewer-independence rule and review-round stop conditions are governed by [method.md](../knowledge-system/method.md#reviewer-independence) and [method.md](../knowledge-system/method.md#review-rounds-and-stop-rules). Callable routes and model families live in [models.md](../knowledge-system/models.md); scoring rubrics live in [EVALUATION.md](EVALUATION.md); paper-claim approval lives in [ROADMAP.md](ROADMAP.md).

## Current State: Observed vs Specified

- **Observed:** One spine review round (`T1.8`) executed and resolved 8/8 high-severity findings in [spine-audit.md](../_runs/2026-11-19/spine-audit.md), verified with Magpie cross-family routes recorded in [model-smoke.json](../_runs/2026-11-19/model-smoke.json). No round-001 experimental plan review (`round-001-review.md`), no gold annotation file, no inter-annotator agreement measurement, no `check_gold_diff` binary, and no blind extraction review exist yet ([STATUS.md](../knowledge-system/STATUS.md) lines 22, 25, 31, 44).
- **Specified (unexecuted):** Two-annotator gold workflow, `check_gold_diff` enforcement, blind randomized extraction review, review-state transitions, and pre-run spend gates ([round-001-plan.md](../experiments/ontology-extraction/round-001-plan.md) lines 61–65, 82–85; [round-001-rubric.md](../experiments/ontology-extraction/round-001-rubric.md) lines 89–99, 125–152).

## Definition of Done and What Counts as Evidence

An agent draft, self-report, or static status string is never completion. A task or gate is **done** only when all four conditions hold ([method.md](../knowledge-system/method.md) lines 80–90; `worklog/2026-11-19/todo.md` lines 10–11, 544–545):

1. **Named artifact and command output:** The target file, run log under `paper-graph/_runs/<date>/`, or test binary exists at a repo-relative path and exits `0` from a clean checkout. Stale commands (such as the retired `uv run pytest` entry in [gates.md](../experiments/ontology-extraction/gates.md) line 14, corrected to `no` in [STATUS.md](../knowledge-system/STATUS.md) line 20) count as `no`.
2. **Honest status tagging:** Statements distinguish `Observed` measurements from `Decided`, `Specified`, `Transferred`, `Contested`, and `Untested` claims ([method.md](../knowledge-system/method.md#status-vocabulary)). An `inconclusive` experimental gate is a valid final result, never an invitation to relax thresholds.
3. **Qualified review record:** Every governing doc, check suite, and experimental plan carries a committed review artifact recording frozen input hashes, reviewer model route and declared family ([models.md](../knowledge-system/models.md) lines 20–29), atomic findings, and explicit dispositions. Per [method.md](../knowledge-system/method.md#reviewer-independence), same-family reviews (for example, Gemini/Gemma, DeepSeek V4/Flash, or `zcode/GLM-5.3` vs `devin/glm-5-3` vs `auto/glm-5.3`) must be labelled *same-family*, never count as independent, and require human adjudication. Reviewers receive the target artifact and [CONSTITUTION.md](../knowledge-system/CONSTITUTION.md), not the author's private rationale.
4. **Human gate sign-off where required:** Gold creation, contested dispositions, hosted spend, final round verdicts, and durable memory promotions carry explicit human sign-off.

## Gold Authority and Expected-Answer Immutability

Per [CONSTITUTION.md](../knowledge-system/CONSTITUTION.md) Invariant 7, `human (annotator)` authors all gold concept spans, relations, claims, evidence scores, and 20 downstream QA expectations. No model or agent may author, edit, or shift an expected gold answer under any circumstance.

- **Agreement before adjudication:** Two PL-knowledgeable human annotators independently annotate six stratified papers across development and test splits; exact-span F1, B-cubed/pairwise merge agreement, Cohen's `kappa` (roles/citations), and Krippendorff's ordinal `alpha` (evidence/answers; `alpha < .67` halts the round) are persisted before adjudication creates canonical gold ([round-001-plan.md](../experiments/ontology-extraction/round-001-plan.md) lines 61–65; [round-001-rubric.md](../experiments/ontology-extraction/round-001-rubric.md) lines 125–135, 142).
- **Disagreement handling (`check_gold_diff`):** When a model output disagrees with gold, gold wins ([lessons.md](../knowledge-system/lessons.md) `L10`). Every failing case receives one label: *system/skill finding*, *ambiguous question*, or *out-of-date case*. `check_gold_diff` fails CI on any expected-answer mutation lacking a human commit, a cited deciding rule sentence for the latter two categories, and second-reviewer sign-off.

## Extraction Review States and Hosted Spend Approval

All model-extracted assertions enter the derived staging store as `unreviewed` ([DATA_MODEL.md](DATA_MODEL.md)). During evaluation and promotion ([round-001-plan.md](../experiments/ontology-extraction/round-001-plan.md) lines 31–34, 82–84):

- **Blind human adjudication:** Human reviewers inspect records in randomized order, blind to arm (`U2`/`S2`/`P2`), recording review seconds per record and minutes per paper (target median `<= 20` min/paper). Promotion to canonical query state requires deterministic quote/locator pass plus human whole-evidence-set score `2` ([EVALUATION.md](EVALUATION.md)); unresolved ambiguity stays `unreviewed` or `Contested`. Cross-family LLM judges may supply secondary agreement data or triage queues only, never canonical approval or headline grades.
- **Hosted spend gate:** Paid extraction or evaluation calls require prior `human (adjudicator)` / `human (operator)` approval bound to a dated price snapshot, `check_budget_preflight` pass ([OPERATIONS.md](OPERATIONS.md)), and an enforced round cap (`<= US$25` in [round-001-rubric.md](../experiments/ontology-extraction/round-001-rubric.md) line 150). Local `llama.cpp` runs report hardware and wall-clock runtime rather than invented dollar prices.
