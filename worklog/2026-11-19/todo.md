# TODO — from design knowledge to a working, evaluated system

**Status:** planned, not started. Baseline: paper-graph at f1e46d9, 2026-11-19.
**Goal:** turn the current design knowledge into (a) a stable spine of governing documents, (b) a praxis entity model for lessons, (c) structurally encoded checks, (d) a docs-effectiveness eval, and (e) a four-paper pilot that can support a defensible paper claim.
**Definition of done:** every key has one owner and one check; the 14 lessons are normalized entities with provenance; the docs eval passes its pre-registered gate; one extraction model is frozen after a structured-output smoke test; the four development PDFs are parsed with valid locators; U2/S2/P2 are scored on development; the held-out run produces a verdict, including inconclusive.

## How to read this

- **W** = workstream, **T** = task. Tasks are ordered by dependency, not by calendar.
- Each task has an output and an exit criterion. A task is not done because an agent said so; it is done when the named command or artifact exists.
- The human owns gold annotations, final review, spend approval, and any choice that changes the paper claim. Agents may draft, audit, and run checks.
- One statement per fact: the constitution governs, the method prescribes, the owning project doc describes. Link, do not restate.

## Where the last two conversations land

| Conversation item | Workstream |
|---|---|
| Guidelines, constitution, spine docs | W1 (constitution, status, method, glossary, owner map, ADRs) |
| Project docs | W3 (architecture, data model, pipeline, evaluation, operations, experiments, roadmap, review) |
| Eval prompt to check effectiveness, using pstack and encode-lessons | W4 (rule-to-check coverage) and W5 (docs-to-decision eval) |
| Next experiments and feature roadmap, including HawDB utilization | W6-W9 (parse, pilot, held-out, identity/rebuild/HawDB) and W10 (roadmap) |
| Other recommendations: status as data, one statement per fact, pre-mortem, negative results, review queue, narrow paper claim | W0, W4, W10, and the guardrails |
| Lessons as first-class knowledge entities | W2 (Episode/Lesson/Check praxis schema, normalized lessons.jsonl) and T9.8 (projection into the graph) |
| nmem keys and pointers | T2.4 (five keys stay the index; lessons stay in the praxis file) |

## Workstreams at a glance

| W | Workstream | Effort | Depends on | Exit |
|---|---|---|---|---|
| W0 | Baseline and guardrails | 0.5 day | — | Branch, docs hash, templates, model inventory, smoke test |
| W1 | Spine and constitution | 1.5 days | W0 | Every key has owner + check; no duplicate rules |
| W2 | Praxis schema and lesson entities | 1 day | T1.1 | 14 valid lesson records; 5 eval scenarios |
| W3 | Project docs | 2 days | W1 | Each doc owns one subject; link check passes |
| W4 | Encode checks | 2 days | W1, W3 | 10 checks; one seeded defect each |
| W5 | Docs-effectiveness eval | 1.5 days | W3, W4 | Gate pass or documented failure |
| W6 | Phase 0: admission + parse | 2 days | W0 | 15/15 admission; 4/4 parse; locators valid |
| W7 | Phase 1: development pilot | 4 days | W2, W4, W6 | U2/S2/P2 scored; gates honest |
| W8 | Phase 2/3: held-out + QA | 4 days | W7 | Verdict + QA report or inconclusive |
| W9 | Identity, rebuild, HawDB utilization | 3 days | W6 | Identity/rebuild tests pass; HawDB memo |
| W10 | Roadmap and paper framing | 1 day | W8, W9 | Roadmap + paper outline + pre-mortem |

Total: roughly 4 weeks of focused work, with W3/W4/W9 partially parallel.

## Dependency graph

~~~mermaid
flowchart TD
  W0["W0 Baseline"] --> W1["W1 Spine"]
  W0 --> W6["W6 Parse"]
  W1 --> W2["W2 Praxis"]
  W1 --> W3["W3 Project docs"]
  W3 --> W4["W4 Checks"]
  W3 --> W5["W5 Docs eval"]
  W4 --> W5
  W2 --> W7["W7 Pilot"]
  W4 --> W7
  W6 --> W7
  W7 --> W8["W8 Held-out + QA"]
  W6 --> W9["W9 Identity + HawDB"]
  W8 --> W10["W10 Roadmap"]
  W9 --> W10
  W2 -.-> W9
  classDef docs fill:#e7f5ff,stroke:#1971c2;
  classDef experiment fill:#ebfbee,stroke:#2f9e44;
  classDef infra fill:#fff4e6,stroke:#e8590c;
  class W1,W2,W3,W4,W5 docs;
  class W6,W7,W8 experiment;
  class W9 infra;
  classDef critical stroke:#d9480f,stroke-width:3px;
  class W0,W1,W3,W4,W7,W8,W10 critical;
~~~

Solid arrows are hard dependencies. The dashed edge is a soft input: the praxis schema informs the W9 projection, but W9 can start without it. A thick border marks the critical path.

Plain-text view:

- W0 -> W1 -> W3 -> W4 -> W7 -> W8 -> W10
- W0 -> W6 -> W7
- W1 -> W2 -> W7
- W6 -> W9 -> W10

## Parallel waves

| Wave | Runs together | Why it can overlap |
|---|---|---|
| 0 | W0 | Baseline must exist before anything is hashed or branched |
| 1 | W1, W6 | Parsing is independent of the spine; it only needs W0 |
| 2 | W2, W3, W6 | Praxis and project docs both need W1; parse continues |
| 3 | W4, W9 | Checks need W3; identity/rebuild needs W6, not the docs |
| 4 | W5, W7 | Docs eval needs W4; pilot needs W2/W4/W6; they do not gate each other |
| 5 | W8, W9 tail | Held-out needs W7; HawDB experiments can finish in parallel |
| 6 | W10 | Roadmap/paper framing needs W8 and W9 findings |

The practical rule: start W6 on day 1, not after the docs. Start W9 as soon as W6 has a parsed corpus. Do not start W3 until T1.1 is frozen, and do not start W7 until W2, W4, and W6 are all green.

## Workstream dependency matrix

| W | Depends on | Unblocks | Can run with |
|---|---|---|---|
| W0 | — | W1, W6 | — |
| W1 | W0 | W2, W3 | W6 |
| W2 | T1.1, T0.3 | W7 | W3, W6 |
| W3 | W1 | W4, W5 | W2, W6 |
| W4 | W1, W3 | W5, W7 | W9 |
| W5 | W3, W4 | — (docs gate) | W7 |
| W6 | W0 | W7, W9 | W1, W2, W3 |
| W7 | W2, W4, W6 | W8 | W5, W9 |
| W8 | W7 | W10 | W9 tail |
| W9 | W6 | W10 | W4, W5, W7 |
| W10 | W8, W9 | — | — |

## Critical path

**W0 -> W1 -> W3 -> W4 -> W7 -> W8 -> W10** = 0.5 + 1.5 + 2 + 2 + 4 + 4 + 1 = 15 days.

- Parse branch: W0 -> W6 -> W7 -> W8 -> W10 = 11.5 days.
- Praxis branch: W0 -> W1 -> W2 -> W7 -> W8 -> W10 = 12 days.
- Docs-eval branch: W0 -> W1 -> W3 -> W4 -> W5 = 7.5 days (not on the pilot path).
- Identity/HawDB branch: W0 -> W6 -> W9 = 5.5 days (not on the pilot path).

The critical path is **docs + checks**, not parsing. Protect W1, W3, and W4 from scope creep; adding a project doc delays the pilot, while adding a HawDB experiment does not.

## Key cross-workstream task edges

| Edge | Why |
|---|---|
| T1.1 -> T2.1, T2.2 | The constitution's invariants define the praxis fields and status tags |
| T1.1 -> T3.1-T3.8 | Project docs cite the constitution; they do not restate it |
| T1.5 -> T4.4 | The owner map decides where each check lives |
| T2.5 -> T5.1 | The five real lessons seed the docs-eval scenarios |
| T3.9 -> T4.4 | The docs lint and the check links must agree |
| T4.2 -> T5.3, T7.5 | The checks define what the eval and pilot can trust |
| T6.3 -> T7.3, T9.4 | The chunk/page/offset table is the evidence substrate for claims and snapshots |
| T7.7 -> T8.1 | The pilot gate decides whether the held-out run may start |
| T8.4 -> T10.2 | The verdict supplies the paper's results and negative results |

## Model plan

See models.md for the full inventory. Three constraints drive the work:

1. **Magpie is authorized for subagents (2026-11-19 ~03:45).** `list_subagent_models` now returns `magpie` and `magpie-anthropic`; `zcode/GLM-5.3`, `devin/glm-5-3` and `auto/glm-5.3` each answered a real call. Flash is still the bulk default, but it is no longer the only route. A reviewer is independent only if it uses a different *family*; GLM-5.3 (Zhipu) is the cheap second family, while Kimi and Opus remain the strong reviewers.
2. **GLM-5.3 is the default second family, and zcode is the preferred route.** Use `magpie-anthropic/zcode/GLM-5.3` first, `magpie/devin/glm-5-3` when ZCode quota returns 429, `magpie/auto/glm-5.3` last (routing group: assert the served model). All three are the same weights and are not independent of each other. GLM-5.3 indexes at 32, the same tier as Flash: it buys family diversity, not design quality.
3. **The local llama.cpp router is the privacy lane, not the second family.** It serves gemma4-12b (40K context, multimodal, reasoning off), gemma4-e4b, qwen3.5-4b, glm-ocr, and bge-m3. They are unloaded and cold-start on demand; use them when the human marks a paper sensitive, and measure structured-output reliability before use.
4. **Nmem's active route is GLM-4.7 at 8192 max_tokens and 60 s.** That is the configuration that failed 16 of 17 KG extractions at a 16K budget. Fix it before any extraction backfill.

| Role | Primary | Second family | Gate |
|---|---|---|---|
| Orchestrator, doc drafter | deepseek-flash | — | T0.5 |
| Design and constitution | auto/gemini-4-carbon | devin/claude-opus-5-5 | T0.5 |
| Independent reviewer (mechanical, fan-out) | zcode/GLM-5.3 | devin/glm-5-3, then auto/glm-5.3 | T0.8 |
| Independent reviewer (spine, long context) | kimi/k3.1-pro | claude-opus-5-5 | T0.8 |
| Docs-eval second family | zcode/GLM-5.3 | gemma4-12b (privacy lane) | T0.6 passes first |
| Extraction arm | one frozen candidate | — | T0.6; never mix across U2/S2/P2 |
| Extraction judge | human | different family from extractor | T7.4 |
| Bulk triage / classification | qwen3.5-4b or gemma4-e4b | — | T0.6 |
| Embeddings | bge-m3, 1024-d | — | fixed for the round |
| OCR fallback | glm-ocr | — | PDF parse smoke test |
| Gold annotation | human | — | never a model |

---


## W0 — Baseline and guardrails

**Depends on:** nothing. **Unblocks:** W1, W6. **Parallel note:** the only strictly serial prerequisite; do it once.

~~~mermaid
flowchart LR
  T0_1["T0.1 Branch"] --> T0_2["T0.2 Docs hash"] --> T0_3["T0.3 Templates"] --> T0_4["T0.4 Precedence"]
  T0_4 --> T0_5["T0.5 Model inventory"]
  T0_5 --> T0_6["T0.6 Structured-output smoke test"]
  T0_4 --> T0_7["T0.7 nmem budget fix"]
  T0_5 --> T0_8["T0.8 Reviewer independence"]
~~~

- [x] **T0.1 Branch and worktree.** Create a branch or worktree for the spine work; record the base commit f1e46d9. Keep code changes off the docs branch until W4.
  - Output: branch name + base commit in this file.
  - Exit: clean worktree; git status shows only intended files.
  - **Recorded 2026-11-19 03:5x:** branch `pg/w0-baseline` from `migrate-paper-graph-rust`; code base `f1e46d9`; freeze commit `3cd97fc` (`docs(pg): freeze design corpus and 2026-11-19 plan`) covering `paper-graph/knowledge-system/` and `worklog/2026-11-19/`. Worktree clean except untracked `test_onedrive/`, which is not part of this work.
- [x] **T0.2 Docs hash manifest.** Hash the current knowledge-system docs and experiment records into _runs/2026-11-19/docs.sha256.
  - Why: W5 must evaluate a frozen docs version.
  - Exit: manifest exists; rerunning the hash is deterministic.
  - **Recorded:** 27 files hashed (knowledge-system + experiments), sorted by path, `shasum -a 256`; rerunning the hash reproduces the manifest byte-for-byte. The digest itself lives in the manifest and the W0 commit message, never inside a hashed doc (self-reference would invalidate it).
- [x] **T0.3 Templates.** Add _templates/lesson.md, _templates/check.md, _templates/eval-case.json, _templates/adr.md.
  - Exit: each template has required fields and one worked example.
  - **Recorded:** four templates; worked examples are L03 (lesson), check_sync_count_guard (check), ADR 0001 docs freeze (ADR), and `sync-destroys-derived` (eval case, JSON validated).
- [x] **T0.4 Precedence.** Record the document order: constitution > method > owning project doc > README status. A fact stated twice is a defect.
  - Exit: one paragraph in method.md; owner map skeleton in STATUS.md.
  - **Recorded:** `method.md` now has *Precedence and ownership* (order, one-owner rule, `_runs/` pinned to `paper-graph/_runs/`, lowercase-filename note); `STATUS.md` carries the owner-map skeleton and a gate ledger.
- [x] **T0.5 Model inventory and freeze.** Write models.md from live endpoints; record provider, model, context, quantization, status, and role. Choose one extraction candidate and one second-family reviewer candidate.
  - Exit: models.md exists; every route is marked authorized, local, or unverified.
  - **Recorded:** `models.md` now carries a *Route status* table with the three buckets. Second-family reviewer chosen: **zcode/GLM-5.3** (Zhipu), fallbacks devin/glm-5-3 then auto/glm-5.3. Extraction candidate stays open until T0.6.
- [ ] **T0.6 Structured-output smoke test.** Run deepseek-flash and gemma4-12b on the same three PL extraction cases: a theorem/definition/citation excerpt, a five-relation schema with evidence offsets, and an ambiguous mention-vs-use case.
  - Measure: first-pass schema validity, empty/truncated rate, finish reason, exact quote recovery, TTFT, tokens/s, cold start, cost.
  - Exit: model-smoke.json; one candidate frozen for the pilot, or the pilot is blocked.
  - **Partial 2026-11-19:** route reachability recorded in `_runs/2026-11-19/model-smoke.json` (seven routes answered). The three structured-output cases are **not** run, so no extraction candidate is frozen yet.
- [ ] **T0.7 nmem budget fix.** Read the provider config without exporting credentials; set max_tokens >= output_budget and timeout >= output_budget / measured tokens-per-second, or reduce the budget.
  - Current risk: GLM-4.7 at 8192 max_tokens and 60 s, the configuration that failed 16 of 17 KG extractions.
  - Exit: a preflight request with the intended budget succeeds, or the mismatch is recorded as a blocker.
  - **BLOCKER recorded 2026-11-19:** nmem serves GLM-4.7 at 8192 max_tokens / 60 s; no output budget has been chosen yet, so no preflight can be written. Fix belongs with the extraction pipeline (W7), not with a blind config bump. Tracked in STATUS.md.
- [x] **T0.8 Reviewer independence.** Pick a second model family for review: gemma4-12b locally, or a verified Claude/Codex route. If neither is callable, record the review as same-family and require human adjudication.
  - Exit: rule in method.md; no document calls a same-family review independent.
  - **Recorded:** second family = **GLM-5.3 (Zhipu)** via zcode, callable and verified; rule written into `method.md` (*Reviewer independence*). Same model behind zcode/devin/auto is explicitly one family.
- [ ] **T0.8b Magpie smoke test.** Send one 16-token request through each of magpie, magpie-anthropic, and magpie-responses; record status, served model, latency, and cold start.
  - Exit: three route records in _runs/2026-11-19/model-smoke.json; any 429/quota is recorded as an upstream blocker, not an unknown model.
  - **Recorded 2026-11-19:** `magpie` and `magpie-anthropic` are authorized for subagents; **`magpie-responses` is not** (`list_subagent_models` does not list it), so the grok wire can be reached only by direct HTTP. Seven route records written. zcode/GLM-5.3 answered after the re-login; the earlier 429 was quota exhaustion, now cleared.

## W1 — Spine and constitution

**Depends on:** W0. **Unblocks:** W2, W3, W4 (via W3), W5 (via W3/W4). **Parallel note:** T1.3-T1.7 are disjoint files and can run together after T1.1; T1.2 needs T1.1 and T1.5; T1.8 is the join.

~~~mermaid
flowchart LR
  T1_1["T1.1 Constitution"] --> T1_2["T1.2 Status"]
  T1_1 --> T1_3["T1.3 Method"]
  T1_1 --> T1_4["T1.4 Glossary"]
  T1_1 --> T1_5["T1.5 Owner map"]
  T1_1 --> T1_6["T1.6 Guidelines"]
  T1_1 --> T1_7["T1.7 ADRs"]
  T1_5 --> T1_2
  T1_2 --> T1_8["T1.8 Spine review"]
  T1_3 --> T1_8
  T1_4 --> T1_8
  T1_6 --> T1_8
  T1_7 --> T1_8
~~~

- [ ] **T1.1 CONSTITUTION.md.** Seven invariants, one page, each with a falsifier and a check pointer:
  1. Zotero is canonical; the HawDB metadata graph is derived and rebuildable.
  2. Extraction, review state, and run provenance never live in the rebuild target.
  3. No claim without a verbatim quote and a resolvable locator; unknown stays unknown.
  4. Gold is the spec; a model never moves an expected answer.
  5. An eval that cannot prove execution is not evidence.
  6. Ontology grows only from a frozen competency question; no unearned taxonomy.
  7. A rule that can be a check must not remain prose.
  - Exit: each invariant has an owner doc, a falsifier, and a check ID or check pending.
- [ ] **T1.2 STATUS.md.** Built / specified / missing, gate ledger state, commit, docs hash. Generate the table from tests and the gate ledger; do not hand-write done.
  - Exit: status table names the owning doc for every subject.
- [ ] **T1.3 Extend method.md.** Add rounds, records, gates, review, delegation, and precedence to the existing mining method. Do not create METHOD.md; APFS resolves it to this file.
  - Exit: a new contributor can tell what to do first and what done means.
- [ ] **T1.4 GLOSSARY.md.** Work, Concept, Claim, EvidenceSpan, Episode, Lesson, Check, MENTIONS/USES/INTRODUCES, review states, status tags.
  - Exit: one definition per term; links to the owning doc.
- [ ] **T1.5 Owner map.** Subject -> owning doc -> enforcement check. At minimum: identity, rebuild, admission, ontology, relations, evidence, evaluation, integrity, operations, distillation.
  - Exit: no subject has two owners.
- [ ] **T1.6 GUIDELINES.md.** Decision procedures for: adding a relation; changing schema; changing an eval; admitting a corpus; promoting a memory. Each ends in a checklist and a stop rule.
  - Exit: each procedure has an input, a decision, a check, and a rollback.
- [ ] **T1.7 ADR/0001-0006.** Separate derived DB; identity strategy; ontology budget; one resolver; metric panel; migration policy. Each with context, decision, rejected alternative, consequences, check.
  - Exit: six ADRs, each with a rejected alternative and a check pointer.
- [ ] **T1.8 Spine review.** Independent reviewer runs spine-orient: reading order, one statement per fact, owning doc decides, environment facts verified.
  - Exit: reviewer findings resolved; no duplicated rule remains.

## W2 — Praxis schema and lesson entities

**Depends on:** T1.1 (constitution), T0.3 (template). **Unblocks:** W7, T5.1 via T2.5, W9 (soft). **Parallel note:** T2.4 and T2.5 can run together after T2.3; T2.1 is the schema gate for T2.2.

~~~mermaid
flowchart LR
  T2_1["T2.1 Praxis schema"] --> T2_2["T2.2 Normalize 14 lessons"] --> T2_3["T2.3 Validate provenance"]
  T2_3 --> T2_4["T2.4 nmem mapping"]
  T2_3 --> T2_5["T2.5 Eval seeds"]
~~~

- [ ] **T2.1 PRAXIS.md schema.** Add Episode, Lesson, Check as a separate namespace from the paper graph. Record relations: DERIVED_FROM, EVIDENCED_BY, APPLIES_TO, ENFORCED_BY, QUALIFIES, CONTRADICTS, SUPERSEDES.
  - Exit: schema doc with field definitions and one worked example.
- [ ] **T2.2 Normalize the 14 lessons.** Convert round-2-rescan.json into lessons.jsonl with fields: id, title, rule, mechanism, scope, boundary, status, derived_from[], evidence[], applies_to, enforcement, rejected_alternative, supersedes.
  - Exit: 14 records; JSONL parses; every record has at least one quote and one status tag.
- [ ] **T2.3 Validate provenance.** Check every quote against the raw thread; dedupe; mark unresolved as unverifiable rather than deleting.
  - Exit: 0 unresolved quotes for the 14; any exception listed in _runs/2026-11-19/praxis-validation.md.
- [ ] **T2.4 nmem mapping.** Keep the five compact keys as the recall index. Store lessons as learning/procedure only if they add recall value; otherwise keep them in lessons.jsonl and point from the index.
  - Exit: no duplicate memories; index links to the praxis file.
- [ ] **T2.5 Eval seeds.** Select five real lessons as docs-eval scenarios: destructive sync, zero-case eval, gold disagreement, question leakage, identity re-import.
  - Exit: five _templates/eval-case.json instances with expected decisions and forbidden actions.

## W3 — Project docs

**Depends on:** W1. **Unblocks:** W4, W5. **Parallel note:** T3.1-T3.8 are disjoint files and can run together; T3.9 is the join.

~~~mermaid
flowchart LR
  T3_1["T3.1 Architecture"] --> T3_9["T3.9 Docs lint"]
  T3_2["T3.2 Data model"] --> T3_9
  T3_3["T3.3 Pipeline"] --> T3_9
  T3_4["T3.4 Evaluation"] --> T3_9
  T3_5["T3.5 Operations"] --> T3_9
  T3_6["T3.6 Experiments"] --> T3_9
  T3_7["T3.7 Roadmap"] --> T3_9
  T3_8["T3.8 Review"] --> T3_9
~~~

Each doc owns one subject and links to the constitution/method instead of restating rules.

- [ ] **T3.1 ARCHITECTURE.md.** Layers, trust boundaries, current vs target, the two pipelines (papers and praxis).
- [ ] **T3.2 DATA_MODEL.md.** SQL/graph/evidence schema, IDs, review states; generate the schema section from code.
- [ ] **T3.3 PIPELINE.md.** Admit -> parse -> chunk -> extract -> validate -> review -> project; failure states; provenance fields.
- [ ] **T3.4 EVALUATION.md.** Arms, frozen inputs, metric panel, gates, integrity checks. One owner; link from methodology.md if needed.
- [ ] **T3.5 OPERATIONS.md.** Sync, backup, migration, single-writer, recovery, failure modes.
- [ ] **T3.6 EXPERIMENTS.md.** Current round, hypotheses, gate ledger, raw result paths.
- [ ] **T3.7 ROADMAP.md.** Now/next/later, each tied to a competency question or gate.
- [ ] **T3.8 REVIEW.md.** Definition of done, reviewer independence, what counts as evidence.
- [ ] **T3.9 Docs lint.** Link check; owner-map check; grep for duplicated rules; stale command check (the pytest row in gates.md is the known example).
  - Exit: 0 broken links, 0 duplicated rules, 0 stale commands.

## W4 — Encode checks

**Depends on:** W1, W3. **Unblocks:** W5, W7. **Parallel note:** the ten checks in T4.2 can be implemented in parallel; T4.3 and T4.4 are the joins.

~~~mermaid
flowchart LR
  T4_1["T4.1 Rule-to-check matrix"] --> T4_2["T4.2 Ten checks (parallel)"]
  T4_2 --> T4_3["T4.3 CI wiring"] --> T4_4["T4.4 Docs point to checks"]
~~~

For each key, prefer a check over prose. principle-encode-lessons-in-structure: if the lesson needs judgment and cannot be encoded, make the instruction prominent and add a failure example.

- [ ] **T4.1 Rule-to-check matrix.** 14 lessons -> check type (lint, fixture, preflight, gate, review) -> owner -> path.
  - Exit: every lesson has a check or an explicit judgment, reviewed by X.
- [ ] **T4.2 Implement the ten checks:**
  1. Pre-sync non-metadata count guard; fail closed.
  2. Populated derived-DB sync test: extraction counts unchanged.
  3. Executable corpus admission: stat, %PDF- signature, SHA-256.
  4. Corpus identity test: itemID <-> zotero_key/DOI mapping.
  5. Evidence quote/locator validator (stub until parser; exact substring first).
  6. Eval runner: executed == corpus, skip exits non-zero, manifest schema.
  7. Gold expected-diff: fail if an expected answer moves without a cited decision.
  8. Question lint: outcome-presupposing wording; schema-field conflation.
  9. Budget preflight: context/max_tokens/timeout/batch inequalities.
  10. Migration lint: no DROP TABLE/VIEW in startup; count oracle on a copy.
  - Exit: 10 checks run locally; each check has a seeded defect that turns it red.
- [ ] **T4.3 CI wiring.** Run checks on PR; keep the slow/property suite nightly.
  - Exit: a seeded defect fails CI; a clean tree passes.
- [ ] **T4.4 Docs point to checks.** Each lesson/owner doc links to its check; no restatement.
  - Exit: owner map has a check column with no empty cells except judgment.

## W5 — Docs-effectiveness eval

**Depends on:** W3, W4. **Unblocks:** docs gate only; not on the pilot path. **Parallel note:** T5.1 and T5.2 can run together; T5.3 waits for both; T5.4 and T5.5 are sequential.

~~~mermaid
flowchart LR
  T5_1["T5.1 15 scenarios"] --> T5_3["T5.3 Variants + frozen hash"]
  T5_2["T5.2 Prompt + rubric"] --> T5_3
  T5_3 --> T5_4["T5.4 Run matrix"] --> T5_5["T5.5 Gate"]
~~~

- [ ] **T5.1 15 scenarios.** Five from real lessons, five novel but plausible, five adversarial. Each has: situation, required decision, required check, required unknowns, forbidden action.
- [ ] **T5.2 Prompt and rubric.** JSON output: owning_doc, decision, check, unknowns, citations, forbidden_action. Score: decision 0/2; citation ownership; check resolvability; unknown handling; forbidden-action absence; token cost.
- [ ] **T5.3 Variants.** A = full docs; B = encoded AGENTS.md rule file generated from the lessons; C = both. Start with deepseek-flash; add gemma4-12b as a second family if T0.6 passes. If only one family is callable, label the result single-family, not independent. Freeze the docs hash before any run.
- [ ] **T5.4 Run.** Use workflow with one agent per scenario, each returning three variant answers; score deterministically in the main process, not by an agent-judge.
- [ ] **T5.5 Gate.** >=90% correct decisions; 0 forbidden actions; >=80% checks resolvable; unknowns correctly flagged on the three ambiguous cases. If it fails, fix the owner/check, not the prompt.
  - Output: _runs/2026-11-19/docs-eval/{config.json,raw/,report.md}.

## W6 — Phase 0: admission and parse

**Depends on:** W0. **Unblocks:** W7, W9. **Parallel note:** admission (T6.1) and parse (T6.2) can overlap after the corpus list; T6.3 waits for the parser; T6.4 and T6.5 can overlap.

~~~mermaid
flowchart LR
  S6["Corpus list"] --> T6_1["T6.1 Admission"]
  S6 --> T6_2["T6.2 Parse 4 PDFs"]
  T6_1 --> T6_3["T6.3 Chunk table"]
  T6_2 --> T6_3
  T6_3 --> T6_4["T6.4 Quote recovery"] --> T6_5["T6.5 Gate ledger"]
~~~

- [ ] **T6.1 Executable admission.** Run on all 15 corpus papers: attachment, file, signature, SHA-256. Fail on any missing file.
- [ ] **T6.2 Parse the four development PDFs.** Native parser first; page mapping, two-column order, math, references, exact quote recovery. Record failures; do not clean them away.
- [ ] **T6.3 Chunk table.** attachment id, parser version, page range, section path, ordinal, content hash, raw text, start/end offsets.
- [ ] **T6.4 Quote recovery.** Sample quotes per domain; measure exact substring and locator validity; pre-register the threshold before measuring.
- [ ] **T6.5 Gate ledger.** Update gates.md: PDF parse, admission, locator validity. A no row is not a pass.
  - Exit: 15/15 admission, 4/4 parse, locator validity at or above the pre-registered bar; failures recorded.

## W7 — Phase 1: development pilot

**Depends on:** W2, W4, W6. **Unblocks:** W8. **Parallel note:** T7.1-T7.3 can start together; T7.4 joins the questions and the claim schema; T7.5 is the single run; T7.6 and T7.7 are sequential.

~~~mermaid
flowchart LR
  S7["Frozen inputs"] --> T7_1["T7.1 20 questions"]
  S7 --> T7_2["T7.2 One resolver"]
  S7 --> T7_3["T7.3 Claim/evidence tables"]
  T7_1 --> T7_4["T7.4 Scorer + seeds"]
  T7_3 --> T7_4
  T7_1 --> T7_5["T7.5 Run U2/S2/P2"]
  T7_2 --> T7_5
  T7_3 --> T7_5
  T7_4 --> T7_5
  T7_5 --> T7_6["T7.6 Blind score"] --> T7_7["T7.7 Gate"]
~~~

- [ ] **T7.1 Freeze 20 questions.** 10 deterministic graph/fact, 6 cross-paper synthesis, 4 unanswerable. Each names acceptable answers, required evidence, forbidden inferences, abstention rule.
- [ ] **T7.2 One resolver.** Share the resolver across U2/S2/P2; the alias table is the only difference. Cut the ontology to the five relations the questions need.
- [ ] **T7.3 Claim/evidence tables.** Implement EvidenceSpan, Claim, HAS_EVIDENCE, review states, run id, schema version.
- [ ] **T7.4 Deterministic scorer + seeded defects.** No LLM judge in the headline; 18 seeded cases from the rubric. Use a human or a different-family judge for secondary review.
- [ ] **T7.5 Run U2/S2/P2 on the four development papers.** Freeze provider, model, quantization, reasoning effort, prompts, and chunker; use one model across all three arms. One prompt revision and one chunk revision maximum, recorded.
- [ ] **T7.6 Blind score.** Human scores evidence/claims/answers; LLM judge only secondary.
- [ ] **T7.7 Gate decision.** Proceed only if structural, provenance, high-risk, seeded-defect, and cost gates pass; otherwise stop and report.
  - Exit: _runs/<date>/pilot/{manifest.json,raw/,metrics.json,report.md}.

## W8 — Phase 2/3: held-out and QA

**Depends on:** W7. **Unblocks:** W10. **Parallel note:** review is sequential by design; do not parallelize blind review across arms.

~~~mermaid
flowchart LR
  T8_1["T8.1 Held-out run"] --> T8_2["T8.2 Blind review"] --> T8_3["T8.3 Downstream QA"] --> T8_4["T8.4 Verdict"]
~~~

- [ ] **T8.1 Run the 11 held-out papers once** with frozen artifacts and budgets; preserve raw failures.
- [ ] **T8.2 Blind review.** Randomize record order; reviewers blind to arm.
- [ ] **T8.3 Downstream QA.** Reviewed graph vs metadata hybrid on the 20 questions; answer score, citation precision/recall, abstention.
- [ ] **T8.4 Verdict.** Pre-registered gates; inconclusive is allowed. Report exact counts, intervals, cost, review burden.
  - Exit: round-001-verdict.md or an explicit inconclusive with the failing gate.

## W9 — Identity, rebuild, HawDB utilization

**Depends on:** W6 (and W7 data for projection). **Unblocks:** W10. **Parallel note:** identity (T9.1-T9.3) and HawDB experiments (T9.4-T9.8) are independent branches; T9.4-T9.8 can run together.

~~~mermaid
flowchart LR
  G["W6 corpus + W7 claims"] --> T9_1["T9.1 Re-import drill"] --> T9_2["T9.2 Derived-DB sync"] --> T9_3["T9.3 Durable key"]
  G --> T9_4["T9.4 COW branches"]
  G --> T9_5["T9.5 Exact vs ANN"]
  G --> T9_6["T9.6 Graph algorithms"]
  G --> T9_7["T9.7 Context budgets"]
  G --> T9_8["T9.8 Staging to canonical"]
~~~

- [ ] **T9.1 Re-import drill.** Clone the library; force a merge/restore that changes itemIDs; sync; assert claims/evidence survive.
- [ ] **T9.2 Derived-DB sync drill.** Populate a derived DB; run metadata sync; assert counts unchanged or fail closed.
- [ ] **T9.3 Durable key.** Add an itemID <-> zotero_key mapping or promote key to the join; test it.
- [ ] **T9.4 COW branches.** One branch per arm/annotator; snapshot-pinned replay; record snapshot id in the manifest.
- [ ] **T9.5 Exact vs ANN.** 1K/10K/100K chunks; recall@k, latency, memory. Decide only from measurement.
- [ ] **T9.6 Graph algorithms.** shortestPath, collect, WITH, variable-length paths on citation/coauthor/compositional queries; record query plans.
- [ ] **T9.7 Context budgets.** Row/payload budgets as agent context budgets; deliberately over-budget and measure the failure.
- [ ] **T9.8 Staging -> canonical.** One transaction; only reviewed assertions enter the query graph; idempotent by run id.
  - Exit: identity/rebuild tests pass; hawdb-utilization.md with measured results.

## W10 — Roadmap and paper framing

**Depends on:** W8, W9. **Unblocks:** none. **Parallel note:** T10.1-T10.3 can run together; T10.4 is the join.

~~~mermaid
flowchart LR
  T10_1["T10.1 Feature roadmap"] --> T10_4["T10.4 Status update"]
  T10_2["T10.2 Paper outline"] --> T10_4
  T10_3["T10.3 Pre-mortem"] --> T10_4
~~~

- [ ] **T10.1 Feature roadmap.** Now/next/later, each tied to a question or gate. No UI before evidence.
- [ ] **T10.2 Paper outline.** Contributions, method, negative results, threats to validity. Claim: architecture + protocol + operational findings, not ontology/QA gains.
- [ ] **T10.3 Pre-mortem and assumption ledger.** Most likely failures and tripwires: parse quality, annotation cost, ontology overfit, eval leakage, identity loss, review burden.
- [ ] **T10.4 Status update.** Update STATUS.md and README; record what is still specified vs observed.

---

## Subagent plan

Rules: one writer per file; a subagent output is a draft or a finding, never done. Reviewers are independent only if they use a different model family; otherwise label the review same-family and add human adjudication. Use subagent_fork only when the task needs this conversation's context; otherwise write a self-contained prompt.

| Task | Agent | Kind | Context to provide | Output | Review |
|---|---|---|---|---|---|
| T1.1 constitution | constitution-drafter | subagent | 14 lessons, status tags, check list; template | CONSTITUTION.md draft | spine-reviewer |
| T1.2 status | status-drafter | subagent | gate ledger, README, test list | STATUS.md draft | spine-reviewer |
| T1.3 method + glossary | method-drafter | subagent | precedence, rounds, status tags | method.md, GLOSSARY.md | spine-reviewer |
| T2.1 praxis schema | praxis-drafter | subagent_fork | last two conversations, ontology.md, lessons JSON | PRAXIS.md + schema | ontology-reviewer |
| T2.2 normalize lessons | lesson-normalizer | subagent | round-2-rescan.json, template | lessons.jsonl | main validates quotes |
| W3 architecture + data model | arch-drafter | subagent | constitution, code, README | two docs | doc-reviewer |
| W3 pipeline + operations | pipeline-drafter | subagent | code, gate ledger, failure modes | two docs | doc-reviewer |
| W3 evaluation + experiments | eval-drafter | subagent | methodology.md, plan, rubric | two docs | doc-reviewer |
| W4 checks 3/5/10 | check-implementer | subagent | check spec, repo, tests | code + tests | main runs CI |
| W5 eval scenarios | scenario-drafter | subagent | lessons, eval template | 15 cases | adversarial-evaluate |
| W5 eval run | eval-runner | workflow | frozen docs hash, prompt, schema | per-scenario JSON | main scores |
| W6 parse | main | — | parser policy | chunks + report | seeded-defect-gate |

Do not use subagents for gold annotation, final verdicts, spend approval, or the paper claim. Those are human decisions.

## Workflow plan

Use workflow only where the work is a real fan-out with independent units. Do not use it to avoid reading or to parallelize a single decision.

### 1. spine-audit

- **Phases:** audit.
- **Fan-out:** 3 reviewers: precedence, duplication, testability.
- **Input:** CONSTITUTION.md, STATUS.md, method.md, GLOSSARY.md, owner map.
- **Schema:** findings:[{kind, ids, issue, fix, severity}].
- **Output:** _runs/<date>/spine-audit.json.
- **Gate:** no high-severity finding unresolved.

### 2. praxis-normalize

- **Phases:** extract -> validate.
- **Fan-out:** 4 batches of 3-4 lessons.
- **Input:** round-2-rescan.json records, _templates/lesson.md.
- **Schema:** lessons:[{id, rule, mechanism, scope, boundary, status, evidence, enforcement, rejected_alternative}].
- **Output:** lessons.jsonl; main validates every quote against the raw thread.
- **Gate:** 14 records, 0 unresolved quotes.

### 3. rule-check-matrix

- **Phases:** propose -> dedupe.
- **Fan-out:** 14 agents, one per lesson.
- **Input:** one lesson + repo map.
- **Schema:** check_type, path, command, pass_condition, seeded_defect.
- **Output:** rule-check-matrix.json.
- **Gate:** every lesson has a check or an explicit judgment owner.

### 4. docs-eval

- **Phases:** freeze -> evaluate -> score.
- **Fan-out:** 15 scenarios x 3 variants; start with deepseek-flash and add gemma4-12b as the second family if T0.6 passes; expand only if variance is high.
- **Input:** docs hash, scenario JSON, output schema.
- **Schema:** scenario_id, variant, owning_doc, decision, check, unknowns, citations, forbidden_action.
- **Output:** _runs/<date>/docs-eval/raw/; main scores deterministically.
- **Gate:** >=90% decisions, 0 forbidden actions, >=80% checks resolvable, 3/3 unknown cases.

### 5. project-doc-review

- **Phases:** review.
- **Fan-out:** 3 reviewers per doc set: ownership, operability, evidence.
- **Input:** docs, constitution, owner map.
- **Schema:** findings:[{doc, rule, issue, fix, severity}].
- **Gate:** no restated rule; every doc has an owner and a check link.

### 6. held-out-review (later)

- **Phases:** review -> adjudicate.
- **Fan-out:** one reviewer per test paper, blind to arm and order.
- **Input:** extracted records, rubric, evidence spans.
- **Schema:** paper_id, claim_scores, evidence_scores, answer_scores, notes.
- **Gate:** agreement recorded before adjudication; no LLM judge as the sole grade.

## PTC tool plan

- **Driver:** every multi-step run is a run_code program that calls tools.workflow, writes JSON to _runs/, and prints only the aggregate. No manual copy-paste between agents.
- **Files:** tools.read, tools.grep, tools.glob; tools.write/tools.edit for docs. Use bash for git, tests, hashes, and long-running jobs, not for reading text files.
- **Progress:** tools.todo_write at the start and end of each workstream; one todo per task ID.
- **Long jobs:** bash with run_in_background for parse/eval; track job_list/job_output; do not busy-poll.
- **Agents:** subagent for independent drafts; subagent_fork for context-heavy schema work; send_message to steer; list_agents to check status.
- **Review:** reviewer agents get the artifact and the constitution, not the author's rationale. Main applies fixes.
- **nmem:** memory_search before writing; memory_add only for keys/pointers; memory_relation_add to link a key to its praxis record; do not save the docs.
- **Presentation:** tools.present at W1, W5, W7, W8.
- **Blocking choices:** ask_user_question only for parser choice, DB path, model/provider, spend approval, or a change to the paper claim. Approval prompts are disabled; do not request sandbox escalation.

## Guardrails

1. **Freeze before eval.** Hash docs and code; no edits during a run. A score is tied to a hash.
2. **One writer per file.** Parallel writers get disjoint files or worktrees.
3. **Reviewer independence.** No agent reviews its own draft.
4. **Evidence or it did not happen.** Every done carries a command, path, or test output.
5. **Status honesty.** specified is not observed; transferred is not tested here; inconclusive is a result.
6. **Budget.** Max 4 concurrent writers; max 15 workflow agents per phase; stop a phase after two consecutive failures and triage.
7. **No silent cleanup.** Parse failures, missing files, and empty outputs are recorded, not deleted.
8. **Keep the paper claim narrow.** Architecture + protocol + operational findings. Do not claim ontology or QA gains before the held-out run.

## First 48 hours

1. **T0.1-T0.8** — branch, docs hash, templates, precedence, model inventory, structured-output smoke test, nmem budget fix, reviewer independence. (Day 1)
2. **W1 drafts** — launch constitution-drafter, status-drafter, method-drafter with disjoint files; run spine-audit after they land. (Day 1)
3. **W2** — fork praxis-drafter; normalize the 14 lessons; select five eval scenarios. (Day 2)
4. **W5 design** — write the 15-case prompt and rubric while W1/W2 are in review; do not run until the docs hash is frozen. (Day 2)

## Next step

Start with **T0.1-T0.8b**, then launch the three spine drafters. Do not start W3 project docs until the constitution and owner map pass the spine audit: writing project docs before the spine exists is how the same rule ends up in five places.

## Model and worktree addendum

See [todo-model-worktree.md](todo-model-worktree.md) for the full model matrix, task routing, worktree commands, conflict rules, and cleanup. These rules override the defaults above:

1. **Route by task, not by default.** deepseek-flash is a 32-index bulk model: drafts, normalization, transcription only. auto/gemini-4-carbon (AA Index v5 59) writes CONSTITUTION.md, PRAXIS.md, EVALUATION.md, and the paper outline; kimi/k3.1-pro runs the spine audit, pre-mortem, and long-context checks; devin/claude-opus-5-5 writes the ten checks; **zcode/GLM-5.3 (Zhipu, AA v5 32) is the cheap second family for review and QC — preferred over gemma4-12b now that Magpie is authorized.**
2. **Independence is relative and gated.** The gate is open: `list_subagent_models` returns `magpie` and `magpie-anthropic`, and T0.8b's three smoke calls passed (2026-11-19 ~03:45). Reviewers still must be a different family: gemma4-12b is Google and cannot review Gemini; deepseek-v4-pro cannot review flash; **zcode/GLM-5.3, devin/glm-5-3 and auto/glm-5.3 are the same model, so they cannot review each other — cite the family, not the route.** Prefer zcode while its quota holds, fall back to devin/glm-5-3 on 429.
3. **Freeze the docs before branching.** knowledge-system/ is untracked: commit it, tag pg-base, and use that tag as the worktree base (f1e46d9 remains the code baseline). Do not create METHOD.md; APFS resolves it to the existing method.md. Extend method.md with the governance sections instead.
4. **One writer per file, one tree per wave.** Worktrees: pg/integration, pg/w1-spine, pg/w2-praxis, pg/w3-project, pg/w4-checks, pg/w6-admission, pg/w9-identity. Merge order W1 -> W6 -> W2 -> W3 -> W4 -> W9. devin/swe-2 has no token API and its 0/0 Magpie price is a listing artifact; do not budget with it.
5. **ZCode GLM-5.3 is authorized for subagents only under magpie-anthropic.** Measured 2026-11-19: `magpie/zcode/GLM-5.3` returns *not allowed for this Session*, while `magpie-anthropic/zcode/GLM-5.3` answered OK after the relogin. The catalog lists the id on both wires; the child-route allowlist admits only the anthropic one. Use `magpie-anthropic/zcode/GLM-5.3`, and keep `devin/glm-5-3` as the non-Flash fallback. If a future allowlist adds the openai-completions route, rerun the smoke test before switching.
