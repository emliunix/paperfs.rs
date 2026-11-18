# Status — paper knowledge system

**Updated:** 2026-11-19 (W1.2 rewrite; owner map moved to docs/owner-map.md)
**Baseline:** paper-graph at `f1e46d9`; docs freeze commit `3cd97fc`; HEAD `ac3de4d`.
**Docs tag:** `pg-base` resolves to `c536301` (annotated tag object `9744fb1`), the reconciliation commit after the freeze, so the tag is not identical to the freeze commit.
**Rule:** a *no* row is not a pass. A row without a check is not done. A row that cites a command which no longer exists is stale, and stale means *no*.

Paths below resolve from `paper-graph/` unless they name a bare `knowledge-system/` file. The gate ledger mirrors `experiments/ontology-extraction/gates.md`; that file remains the experiment-side record, and T6.5 will re-run it. This file adds the evidence path and corrects stale states; it does not add measurements.

## Gate ledger

| Gate | State | Evidence / current state | Evidence path |
|---|---|---|---|
| Research survey | yes | Primary sources and official docs fetched 2026-11-18 | `analysis/llm-document-graph-research.md` |
| DeepSeek secondary critique | yes | Configured credential passed without disclosure; generic ontology text only | `analysis/deepseek-ontology-critique.md` |
| Initial DeepSeek attempt | yes | Failed with `MISSING_CREDENTIAL`, then succeeded after the parent supplied the key | Recorded in `analysis/deepseek-ontology-critique.md`; no separate run log |
| Corpus attachment rows | yes | Read-only HawDB queries over selected Zotero `itemID` values | `experiments/ontology-extraction/round-001-corpus.json` |
| Corpus local PDF admission | yes | 15/15 revised candidates have a local PDF with a `%PDF-` signature; manual, not executable | `experiments/ontology-extraction/round-001-corpus.json`, `experiments/ontology-extraction/grounding.md` |
| Dangling attachment detection | yes | `60`, `47`, `5`, `8` lacked the referenced local PDF and were excluded | `experiments/ontology-extraction/grounding.md` |
| Corpus structure | **no** (was yes) | The cited `uv run pytest` cannot run: there is no `pyproject.toml` and no Python test suite after the Rust port. The five "passed" tests have no runner on this tree | Absent: no `pyproject.toml`; `tests/*.rs` are Rust integration tests |
| Exploratory metadata baseline | yes | Four simple hybrid queries each returned a sensible rank-1 work; not a formal round result | `experiments/ontology-extraction/gates.md` §Exploratory baseline |
| Independent plan review | **no** | `round-001-review.md` absent; required before model extraction. The reviewed worklog plan is not a review of the round-001 experimental plan | Absent: `experiments/ontology-extraction/round-001-review.md` |
| PDF parse smoke test | no | No PDF has been parsed; notation, layout, and quote recovery unknown | Absent: `_runs/2026-11-19/parse-report.md` |
| Ontology/annotation manual | no | Research draft exists; reviewed v0 manual absent | Draft: `analysis/llm-document-graph-research.md`; no reviewed manual |
| Gold annotations and agreement | no | No gold files; no inter-annotator measurement | Absent |
| Downstream gold questions | no | Question design only; no reviewed question set | Absent: `experiments/ontology-extraction/questions.json` |
| Executable seeded defects | no | Required cases are listed in the rubric; fixtures and scorer absent | `experiments/ontology-extraction/round-001-rubric.md` (list only) |
| Model/provider freeze and price | no | No extraction model snapshot and no dated run price selected. Reachability smoke test is not a freeze | Absent; route reachability only in `_runs/2026-11-19/model-smoke.json` |
| Extraction arms `U2`/`S2`/`P2` | no | No hosted extraction run; no spend incurred for an arm | Absent |
| Deterministic scorer | no | Not implemented | Absent |
| Human blind scoring | no | Not performed | Absent |
| Round verdict | no | Verdict file absent | Absent: `experiments/ontology-extraction/round-001-verdict.md` |

The changes from the W0 skeleton are stated rather than implied: "Corpus structure" fell from yes to no because its command no longer exists, and "Independent plan review" fell from yes to no because the skeleton's *yes* pointed at this worklog, which is not the round-001 experimental plan. "Corpus local PDF admission" is *yes* (manual, 15/15) while gold remains the separate *no* row; the skeleton's *partial* merged two subjects.

## Workstream state

| W | State | What exists |
|---|---|---|
| W0 | done for its deliverables; two tasks open | Freeze commit `3cd97fc` (docs corpus + 2026-11-19 plan), docs manifest `_runs/2026-11-19/docs.sha256` (27 files, one manifest line per file), four templates in `_templates/`, `models.md` route-status inventory, `model-smoke.json` (seven routes answered a two-token prompt). T0.6 structured-output smoke and T0.7 nmem budget remain open and are carried into W7 |
| W1 | in progress | Drafted: `CONSTITUTION.md` (seven invariants, `cc3279d`), `method.md` governance sections, `GLOSSARY.md`, `GUIDELINES.md`, `docs/owner-map.md`, ADR `0001`-`0006` (`ac3de4d`). This file is T1.2. Pending: T1.8 spine audit, and resolution of the 14 multi-document conflicts listed in the owner map |
| W2 | not started | No `PRAXIS.md`; no `lessons.jsonl`; the 14 lessons remain prose records in `lessons.md` / `round-2-rescan.json` |
| W3 | not started | `docs/` contains only `owner-map.md`; no ARCHITECTURE, DATA_MODEL, PIPELINE, EVALUATION, OPERATIONS, EXPERIMENTS, ROADMAP, or REVIEW |
| W4 | not started | Zero `check_*` binaries or tests; the ten checks exist as planned IDs in the owner map |
| W5 | not started | No docs-eval config, raw runs, or report |
| W6 | not started | No parser, no parse report, no chunk/page/offset table; admission is still the manual 15/15 check |
| W7 | not started | No pilot run, manifest, metrics, or score |
| W8 | not started | No held-out run, blind review, downstream QA, or verdict |
| W9 | not started | No identity/rebuild drill and no HawDB memo |
| W10 | not started | No roadmap update and no paper outline |

The docs manifest is a freeze snapshot, not a current-tree check. `shasum -a 256 -c` currently matches 26/27 files; `knowledge-system/method.md` differs because T1.3 extended it after the freeze. W5 must re-freeze a manifest before it runs.

## Specified vs observed

Observed facts are the only rows allowed to read as results. Everything else is design or plan.

### Observed (evidence exists)

| Fact | Evidence |
|---|---|
| 15/15 corpus candidates have a local PDF with a `%PDF-` signature (manual check, 2026-11-18) | `experiments/ontology-extraction/round-001-corpus.json`, `experiments/ontology-extraction/grounding.md` |
| 18 threads mined over 17 independent sessions; 14 lessons; three mining rounds | `knowledge-system/README.md`, `corpus.md`, `threads.json` |
| Four exploratory hybrid queries returned rank-1 works (`3154`, `2314`, `1490`, `774`) | `experiments/ontology-extraction/gates.md` |
| Seven model routes answered a two-token prompt on 2026-11-19 | `_runs/2026-11-19/model-smoke.json` |
| Four templates exist with worked examples | `knowledge-system/_templates/` (`lesson.md`, `check.md`, `eval-case.json`, `adr.md`) |
| 27 docs hashed at the freeze; 26 still match, `method.md` drifted after T1.3 | `_runs/2026-11-19/docs.sha256`; verified 2026-11-19 |
| The crate's current model is metadata-only (Work, Person, Collection, Tag, Venue, Attachment); no Concept/Claim node, no chunk table, no PDF parser, no scorer | Reported from the code at `f1e46d9` in `knowledge-system/README.md`; not rebuilt in this task |

### Design only (no experiment has run)

| Subject | What is specified | Would become observed when |
|---|---|---|
| Three-layer split: Zotero canonical, HawDB rebuildable index, separate derived store for extraction and review | `CONSTITUTION.md` invariants 1-3, ADR `0003` | A populated derived store survives a metadata rebuild with unchanged counts |
| Evidence model: atomic claims, atomic spans, verbatim quote plus resolvable locator, `unknown` stays visible | `CONSTITUTION.md` invariants 2 and 5, ADR `0004`, `ontology.md` | A parser produces a chunk/page/offset table and quote recovery is measured |
| Praxis schema: Episode, Lesson, Check as a separate namespace | `PRAXIS.md` is not written yet; relations named in `todo.md` T2.1 | 14 normalized records in `lessons.jsonl` pass schema and provenance checks |
| Evaluation protocol: frozen axis, one resolver, metric panel, pre-registered gates | `methodology.md`, ADR `0005` | An eval manifest exists and the runner proves `executed == corpus` |
| Ontology budget and the five-relation pilot cut | `ontology.md`, `CONSTITUTION.md` invariant 5 | A frozen competency question is answered by an arm, not by a hand-written draft |
| Ten structural checks, each with a seeded defect | `docs/owner-map.md` check column, `todo.md` T4.2 | Each check runs locally and turns red on its seeded defect |
| Owner map: one owning doc and one check or explicit human judgment per subject | `docs/owner-map.md` | T4.4 links every owner row to a resolvable check |
| Source admission rule: local bytes, `%PDF-` signature, SHA-256 | `CONSTITUTION.md` invariant 4, `engineering.md` | An executable admission check runs on all 15 papers |
| Extraction budget inequality and per-call provenance | `engineering.md`, `lessons.md` L12 | T0.7 preflight succeeds and a run logs per-call usage |
| Paper claim restricted to architecture, protocol, and operational findings | `CONSTITUTION.md` invariant 7, `README.md` | The held-out run produces a verdict, including inconclusive |

## Owner map

The subject-to-owner table lives in [docs/owner-map.md](docs/owner-map.md). This file does not duplicate it; a subject with two owner rows is a defect, and the owner map's *Current Multi-Document Conflicts* section already lists the subjects that still appear in more than one document.

The W0 skeleton's inline owner table is removed rather than carried forward. It assigned *Layer ownership* to `CONSTITUTION.md`/`check_sync_count_guard`, which contradicted invariant 1 and the owner map; keeping two tables would have preserved exactly the duplicate ownership the map exists to prevent.

## What this file does not claim

- No extraction arm has run; no PDF has been parsed.
- No gold annotation exists.
- Model routes were smoke-tested (two tokens), not benchmarked.
