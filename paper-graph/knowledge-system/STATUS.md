# Status — paper knowledge system

**Updated:** 2026-11-19 (W0 skeleton; W1.2 owns this file after the spine lands)
**Baseline:** paper-graph at `f1e46d9`; docs frozen at `pg-base`.
**Rule:** a *no* row is not a pass. A row without a check is not done.

## Gate ledger (skeleton)

| Gate | Artifact | State | Evidence |
|---|---|---|---|
| Plan review | `worklog/2026-11-19/todo.md` | yes (this plan) | — |
| Docs freeze | `_runs/2026-11-19/docs.sha256` | yes | 27 files hashed and reproducible; digest recorded in the W0 commit and todo.md (not inside a hashed doc) |
| PDF parse | `_runs/2026-11-19/parse-report.md` | no | no parser has run |
| Corpus admission | `experiments/ontology-extraction/round-001-corpus.json` | partial | 15/15 PDF-verified 2026-11-18, gold pending |
| Model freeze | `_runs/2026-11-19/model-smoke.json` | partial | seven routes answered a two-token call; T0.6 structured-output cases not run, so no arm is frozen |
| nmem budget (T0.7) | nmem provider config | no — blocker | GLM-4.7 8192 max_tokens / 60 s; no output budget chosen yet |
| Gold annotations | — | no | human-owned |
| U2/S2/P2 scored | — | no | depends on parse + gold |
| Verdict | `round-001-verdict.md` | no | depends on held-out run |

## Owner map (skeleton — W1.1/W1.5 fill it)

One row per subject; one owner; one check. Empty check cells are allowed only when the owner is marked `judgment`.

| Subject | Owning doc | Owner | Check | Status |
|---|---|---|---|---|
| Layer ownership (canonical vs rebuildable vs distilled) | CONSTITUTION.md | W1.1 | check_metadata_sync_guard | planned |
| Source admission | docs/PIPELINE.md | W3.3 | check_corpus_admission | planned |
| Evidence and provenance | docs/DATA_MODEL.md | W3.2 | check_evidence_locator | planned |
| Evaluation protocol | docs/EVALUATION.md | W3.4 | check_eval_manifest | planned |
| Extraction budget | docs/OPERATIONS.md | W3.5 | check_budget_preflight | planned |
| Workbook / praxis schema | PRAXIS.md | W2.1 | check_lesson_schema | planned |
| Paper claim | docs/ROADMAP.md | W10.2 | judgment (human) | planned |

## What this file does not claim

- No extraction arm has run; no PDF has been parsed.
- No gold annotation exists.
- Model routes were smoke-tested (two tokens), not benchmarked.
