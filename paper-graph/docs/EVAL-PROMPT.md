# W5 Docs-Effectiveness Evaluation — Frozen Prompt and Rubric

**Status:** Design frozen for review; **T5.4 not run**. The common prompt below is the only text supplied to every answering agent. The rubric is runner-only and must never be included in an answering agent's context.

**OPEN HUMAN DECISION:** The contested U2/S2/P2 evaluation axis, resolver, and extraction headline metric remain undecided. This W5 document does not select them. Its A/B/C documentation conditions are the W5 docs-effectiveness comparison, not a resolution of the extraction experiment's contested axis.

## 1. Variants and execution contract

Each scenario is answered once under each variant with identical scenario payload, model route, decoding parameters, output budget, and retry policy:

| Variant | Documentation input after the common prompt |
|---|---|
| **A — full docs** | The frozen full W5 documentation set, explicitly allowlisted in `config.json`: `knowledge-system/CONSTITUTION.md`, `method.md`, `lessons.md`, `ontology.md`, `PRAXIS.md`, `docs/owner-map.md`, `docs/rule-check-matrix.md`, and the eight owning project docs (`ARCHITECTURE.md`, `DATA_MODEL.md`, `PIPELINE.md`, `EVALUATION.md`, `OPERATIONS.md`, `EXPERIMENTS.md`, `ROADMAP.md`, `REVIEW.md`). Run artifacts and this runner-only prompt/rubric are excluded. |
| **B — encoded rules** | Only the frozen `AGENTS.md` rule file generated from the fourteen lessons, plus its generator/version identifier. No full-doc fallback is available. |
| **C — both** | The identical frozen bytes from A followed by the identical frozen bytes from B. Do not regenerate either input for C. |

The runner supplies only `scenario_id` and `situation` from each seed. It must not expose `required_decision`, `required_check`, `owning_doc`, `required_unknowns`, or `forbidden_action`. Start with `deepseek-flash`. Add `gemma4-12b` as a second family only if T0.6 passes; if only one family is callable, label the result **single-family**, never independent.

T5.4 uses one agent per scenario, obtains that agent's A/B/C answers, preserves every raw response and usage record, and scores in the main process. There is no agent-judge. A completed malformed answer is an executed case with zero quality scores; a missing answer, skip, timeout without a raw typed failure, or unscored case makes the run invalid rather than improving its denominator.

## 2. Frozen docs-hash requirement

No answering agent may run until `_runs/2026-11-19/docs-eval/config.json` freezes all of the following:

1. **Baseline proof:** the SHA-256 of `_runs/2026-11-19/docs.sha256` and verification of its 27 existing entries. That legacy manifest does **not** include the later project docs, this prompt, the generated rule file, or these seeds, so it is not sufficient by itself.
2. **Run input manifest:** for every common, variant, and seed input, a path relative to `paper-graph/` and the SHA-256 of its exact bytes. Store entries as `<hex>  <path>\n`, sorted by byte-wise path order, and record the manifest's own SHA-256 in `config.json`.
3. **Variant identity:** A's full-doc allowlist and hashes, B's generated `AGENTS.md` hash plus generator/source-lessons hashes, C's assertion that its bytes are exactly A+B, this prompt's hash, all fifteen seed hashes, and the deterministic scorer version/hash.
4. **Execution identity:** model/provider/family, decoding parameters, output/context budgets, retry policy, scenario order, and run id. Missing token usage is `unreported`, never `0`.

The freeze must precede predictions. A hash mismatch aborts the run. Editing docs, rules, prompt, seeds, or scorer after seeing predictions invalidates the freeze and requires a new run id; expected answers must never be tuned to outputs. **No hashes are fabricated in this design document and no T5.4 run is authorized here.**

## 3. BEGIN FROZEN COMMON PROMPT

You are a newcomer making one governed Paper Graph decision. Use only the scenario and the documentation supplied for this variant. Do not assume an unlisted implementation, measurement, human decision, locator, or usage value. Choose the action and unknown keys that best apply. Return one JSON object and no Markdown or extra text.

The scenario payload is:
`{"scenario_id":"...","situation":"..."}`

Return exactly these fields:
`{"owning_doc":"paper-graph/...","decision":"ACTION_CODE","check":"check_id","unknowns":["UNKNOWN_KEY"],"citations":["paper-graph/...#optional-anchor"],"forbidden_action":["FORBIDDEN_CODE"]}`

Rules:
- `decision`: exactly one action code from the vocabulary below, with no rationale or punctuation.
- `owning_doc`: the single canonical repository-relative owning document path.
- `check`: exactly one real check id from the rule-check matrix.
- `unknowns`: exactly the applicable unknown keys below; do not invent values for them.
- `citations`: include the owning document, optionally with an anchor; other supplied citations are allowed.
- `forbidden_action`: every applicable forbidden action code that your decision avoids, not an action you propose.
- Emit no additional fields.

### Action codes

Safe actions:
- `ISOLATE_AND_GUARD_SYNC` — isolate non-metadata state and require a fail-closed sync count guard.
- `INVALIDATE_INCOMPLETE_RUN` — invalidate a zero or incomplete execution and preserve raw accounting.
- `PRESERVE_GOLD_FOR_HUMAN` — preserve human gold and refer disagreement to human adjudication.
- `REJECT_LEADING_QUESTION` — reject outcome-presupposing wording and require question lint.
- `REKEY_DURABLE_IDENTITY` — rejoin through durable Zotero keys/DOIs rather than local IDs, titles, or hashes.
- `REJECT_UNVERIFIED_SOURCE` — reject sources without verified local bytes, PDF signature, and SHA-256.
- `DEFER_ONTOLOGY_TO_QUESTIONS` — admit vocabulary only after frozen blocking competency questions.
- `ABSTAIN_PENDING_HUMAN_REVIEW` — keep semantic identity unresolved pending evidence and human adjudication.
- `MARK_UNKNOWN_AND_BLOCK_CLAIM` — keep unsupported evidence unknown/unreviewed and block promotion.
- `BLOCK_BUDGET_PREFLIGHT` — reject an infeasible call budget before execution.
- `SEPARATE_DERIVED_STATE` — keep review/provenance outside the disposable metadata rebuild target.
- `OPEN_HUMAN_DECISION` — leave a contested decision explicitly open to its named human owner.
- `BLOCK_FAILED_EVALUATOR` — map evaluator failures to BLOCKED/INVALID and withhold verdicts.
- `BLOCK_DESTRUCTIVE_MIGRATION` — reject startup destructive rebuilds and require a reviewed copy/oracle path.

Forbidden actions:
- `ALLOW_DERIVED_LOSS` — accept deletion of extraction or review state.
- `REEXTRACT_WITHOUT_GUARD` — repopulate lost state without a count guard.
- `REPORT_ZERO_CASE_PASS` — report a zero-case run as PASS/evidence.
- `SCORE_EMPTY_GRAPH_AGREEMENT` — score empty outputs as agreement.
- `EDIT_GOLD_TO_MODEL` — move expected gold toward model output.
- `LET_MODEL_AUTHOR_GOLD` — let a model/agent author replacement gold.
- `KEEP_OUTCOME_PRESUPPOSITION` — keep wording that supplies the tested outcome.
- `ASSUME_UNCHANGED_GOLD_IS_ENOUGH` — excuse question leakage because gold did not move.
- `USE_TITLE_AS_IDENTITY` — use title as paper identity.
- `USE_CONTENT_HASH_AS_IDENTITY` — use content hash as paper identity.
- `ADMIT_FROM_STATUS_STRING` — treat a status string as executable admission.
- `TREAT_ATTACHMENT_ROW_AS_BYTES` — treat attachment metadata as verified bytes.
- `ADD_CATALOGUE_WITHOUT_QUESTION` — add ontology vocabulary without a blocking question.
- `HIDE_TAXONOMY_IN_KIND_ENUM` — hide subsumption in a closed kind enum.
- `AUTO_MERGE_AMBIGUOUS_CONCEPTS` — merge ambiguous concepts without definition-level justification.
- `CLAIM_LOCATOR_PROVES_EQUIVALENCE` — treat locator validity as semantic-equivalence proof.
- `FABRICATE_QUOTE_OR_LOCATOR` — invent evidence text or coordinates.
- `PROMOTE_UNKNOWN_CLAIM` — promote an unresolved claim because it is plausible.
- `RETRY_IDENTICAL_BUDGET` — retry the same infeasible budget.
- `COERCE_MISSING_USAGE_TO_ZERO` — store unreported usage as zero.
- `COLOCATE_REVIEW_WITH_METADATA` — put review/provenance in the metadata rebuild target.
- `CALL_ALL_STATE_REBUILDABLE` — declare extraction/judgment disposable.
- `AGENT_CHOOSE_EVALUATION_AXIS` — let an agent choose the contested axis/resolver.
- `RUN_WITH_CONFOUNDED_RESOLVERS` — score while falsely claiming resolution is constant.
- `AGENT_CHOOSE_HEADLINE_METRIC` — let an agent choose the contested headline.
- `DECLARE_STRICT_F1_WINNER` — declare a strict-F1 winner while hiding panel risks.
- `MAP_TIMEOUT_TO_PASS` — map timeout/truncation/unasked atoms to PASS.
- `TRUST_UNVALIDATED_SCORER` — trust a verdict without evaluator validation.
- `RUN_STARTUP_DROP_REBUILD` — run startup DROP/rebuild migration logic.
- `USE_IGNORED_SNAPSHOT_AS_ORACLE` — treat an ignored fixed-count snapshot as a migration oracle.

### Unknown keys

- `HAWDB_ALTER_SUPPORT` — HawDB additive-ALTER capability is unverified.
- `DERIVED_SCHEMA_VERSION` — the derived store's schema version is unestablished.
- `ZERO_EXECUTION_CAUSE` — the concrete cause of zero executions is unestablished.
- `RAW_RUN_COMPLETENESS` — authentic raw per-case output availability is unestablished.
- `HUMAN_GOLD_DISPOSITION` — no human disposition of the disagreement is assumed.
- `GOLD_REVIEW_STATUS` — required human review status is unestablished.
- `QUESTION_PRIOR_USE` — prior use of the revised question is unestablished.
- `GOLD_CHANGE_STATUS` — whether the effective oracle changed is unestablished.
- `DOI_AVAILABILITY` — DOI coverage for affected works is unestablished.
- `REIMPORT_KEY_MAPPING` — the verified old/new durable-key mapping is unestablished.
- `PDF_TEXT_RECOVERABILITY` — byte admission does not establish parseable text.
- `METADATA_COVERAGE` — metadata limits on retrieval claims are unestablished.
- `BLOCKING_COMPETENCY_QUESTION` — no frozen blocking competency question is assumed.
- `ONTOLOGY_VOCABULARY_DECISION` — the human ontology owner's vocabulary decision is open.
- `DEFINITION_LEVEL_EQUIVALENCE` — definition-level semantic equivalence is unestablished.
- `HUMAN_MERGE_ADJUDICATION` — human semantic merge adjudication is unestablished.
- `SOURCE_QUOTE_AVAILABILITY` — an exact raw source quote is unestablished.
- `EVIDENCE_LOCATOR_RESOLUTION` — resolvable evidence coordinates are unestablished.
- `MEASURED_TOKENS_PER_SECOND` — measured throughput needed for timeout sizing is unavailable.
- `TOKEN_USAGE_REPORTING` — actual token usage is unreported.
- `SEPARATE_STORE_IMPLEMENTATION` — implementation of the separate derived store is unestablished.
- `SYNC_TARGET_CONTENTS` — non-metadata contents of the sync target are unestablished.
- `EVALUATION_AXIS_DECISION` — the evaluation-axis decision is OPEN HUMAN DECISION.
- `RESOLVER_DECISION` — the resolver/arm decision is OPEN HUMAN DECISION.
- `HEADLINE_METRIC_DECISION` — the headline-metric decision is OPEN HUMAN DECISION.
- `METRIC_EXCEPTION_CONDITIONS` — single-F1 exception conditions are unestablished.
- `EXECUTION_ACCOUNTING` — complete execution/failure accounting is unestablished.
- `SCORER_VALIDATION_STATUS` — independent evaluator validation is unestablished.
- `MIGRATION_COUNT_ORACLE` — an executed non-ignored migration oracle is unestablished.

## 4. END FROZEN COMMON PROMPT

## 5. Runner-only deterministic rubric

The scorer reads expected codes directly from the frozen seeds: the prefix before `: ` in `required_decision` is the expected action; prefixes in `required_unknowns` and `forbidden_action` are expected key sets. Sort and deduplicate arrays before comparison; duplicates in a raw answer make it structurally invalid. No embeddings, fuzzy matching, LLM judgment, or post-hoc synonyms are permitted.

### Structural validity

Require an object with exactly the six requested fields and these types: strings for `owning_doc`, `decision`, `check`; arrays of unique strings for `unknowns`, `citations`, `forbidden_action`. `decision` and `check` must be single tokens from the frozen vocabularies. Invalid JSON/schema scores zero on every quality field but remains an executed failure with token cost preserved.

### Per-case scoring

| Field | Points | Deterministic rule |
|---|---:|---|
| **decision** | **0 or 2** | 2 iff `decision` exactly equals the seed's expected safe action code; otherwise 0. There is no partial credit. |
| **citation ownership** | **0 or 1** | 1 iff `owning_doc` exactly equals the seed's `owning_doc` and at least one citation has that exact path, allowing a suffix beginning `#`. Extra non-owner citations do not substitute for the owner. |
| **check resolvability** | **0 or 1** | 1 iff `check` exactly equals the seed's `required_check` and is one of the nine unique W4 check ids used by the matrix; otherwise 0. Naming a human judgment or a non-matrix id scores 0. |
| **unknown handling** | **0 or 1** | 1 iff the `unknowns` set exactly equals the seed's expected unknown-key set; missing, fabricated, or extra keys score 0. The three pre-registered ambiguous cases gate on this field. |
| **forbidden-action absence** | **0 or 1** | 1 iff the chosen decision is not any forbidden action code and the `forbidden_action` set exactly equals the seed's forbidden set. A forbidden decision is also a run-level forbidden-action incident. |

Diagnostic total: **0–6** per answer. This total does not replace the T5.5 gates and is not an extraction headline metric.

### Aggregation and token cost

- Report each model × variant cell separately: decision accuracy = earned decision points / (2 × 15); resolvable checks = check points / 15; citation ownership and unknown accuracy are diagnostics; forbidden incidents are exact counts.
- The three ambiguous gate cases are `identity-reimport`, `ambiguous-concept-merge`, and `unresolvable-evidence`. Each must earn the unknown-handling point.
- Record provider-reported input/output/cached tokens and computed total tokens per answer, with sums and means per cell. Missing usage is `unreported` and excluded from numeric token means with its count disclosed; never coerce it to zero. Token cost is observational and has no W5 pass threshold.
- Preserve raw JSON, parse errors, token records, and score rows. A skipped or unscored answer invalidates execution accounting and prevents a PASS verdict regardless of percentages.
