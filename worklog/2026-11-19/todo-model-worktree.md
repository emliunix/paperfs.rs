# Model routing and worktree plan

**Status:** derived 2026-11-19 from the Magpie catalog in this session, public benchmark pages fetched 2026-11-19, and the repo at f1e46d9. Companion to [todo.md](todo.md); it changes no task scope and does not edit todo.md.
**Updated 2026-11-19 ~03:45:** the subagent allowlist now authorizes Magpie, and GLM-5.3 is callable on three routes (zcode preferred, devin fallback, auto last). Sections 1.1, 1.2, 2.0-2.5 and R22-R24 below were rewritten for that; the task scope is unchanged.
**Evidence rule:** every score carries its evaluator, configuration, and date. Vendor self-reports are labelled. Where no public benchmark exists the row says *no public benchmark found* and ranks by catalog capacity plus family position only. No number below is invented; arithmetic on published prices is marked *est.*
**Counts:** todo.md had 67 checkboxes when this report was written (T0.1-T10.4); T0.8b was added afterward, so the current file has 68. The table in §2.3 covers the original 67 rows plus the T0.8b callability gate.

## 0. Three facts that change the plan

1. **The docs are untracked.** `git status` shows `?? paper-graph/knowledge-system/`. A worktree created from f1e46d9 has **no docs, no lessons, no models.md**. Freeze the directory in a commit first; the shared base becomes tag `pg-base`, not commit f1e46d9. f1e46d9 stays the *code* baseline.
2. **APFS is case-insensitive here** (`core.ignorecase=true`; `knowledge-system/METHOD.md` resolves to the existing `method.md`). todo.md already refers to `METHOD.md` four times. On macOS a write to `METHOD.md` silently edits `method.md`; on the Linux CI runner it creates a second file and duplicates every rule. Keep the lowercase name and let the docs lint reject case-only twins.
3. **`devin/swe-2` has no per-token price.** Cognition ships no SWE-2 API; the published cost is per Devin task (`$0.37` medium, `$0.78` high, `$1.18` max). The Magpie listing `0/0` is a listing artifact, not free capacity, and it cannot enter a cost gate. Its base-model lineage is undisclosed, so it can never certify reviewer independence.

## 1. Model matrix

### 1.1 Catalog facts (Magpie gateway, 2026-11-19)

Per-1M prices are the per-token values multiplied by 1e6. `auto/` is part of the model id; see risk R1.

| Magpie id | in $/M | out $/M | cache-read $/M | ctx | max out | modalities | efforts |
|---|---|---|---|---|---|---|---|
| `auto/gemini-4-carbon` | 2.00 | 10.00 | 0.10 | 1,048,576 | 65,536 | text, image | low, medium, high |
| `auto/gemini-4-argon` | 2.00 | 10.00 | 0.10 | 1,000,000 | 65,536 | text, image | low, medium, high |
| `deepseek/deepseek-v4-pro` | 0.66 | 1.98 | 0.022 | 1,048,576 | 393,216 | text | none, low, high, max |
| `deepseek/deepseek-flash` | 0.15 | 0.60 | 0.003 | 1,048,576 | 393,216 | text, image | none, low, high, max |
| `kimi/k3.1-pro` | 2.00 | 10.00 | 0.20 | 1,048,576 | 1,048,576 | text, image | low, high, max |
| `devin/swe-2` | 0 | 0 | 0 | 262,000 | 128,000 | text, image | medium, high, max |
| `devin/claude-opus-5-5` | 4.00 | 20.00 | 0.20 | 1,000,000 | 128,000 | text, image | low, medium, high, xhigh, max |
| `zcode/GLM-5.3` (magpie-anthropic) | n/p | n/p | n/p | 1,000,000 | 128,000 | text, image | low, high, max |
| `devin/glm-5-3` (magpie) | n/p | n/p | n/p | 1,048,576 | 128,000 | text, image | low, high, max |
| `auto/glm-5.3` (magpie, routing group) | n/p | n/p | n/p | 1,048,576 | 128,000 | text, image | low, high, max |

Non-Magpie routes:

| Route | Model | ctx | notes |
|---|---|---|---|
| DSH subagent (only authorized) | `deepseek-official/DeepSeek-V4.1-Flash` | 1M claim | efforts off, low, high, max; default max in this session |
| local llama.cpp | `gemma4-12b` Q4_K_XL, reasoning off | 40,960 preset | text, image, audio; cold start; shares GPU with OCR and embeddings |
| local llama.cpp | `gemma4-e4b`, `qwen3.5-4b`, `glm-ocr`, `bge-m3` 1024-d | 16,384 for qwen | no per-token price; GPU contention is the real budget |
| nmem provider | `glm-4.7` | 8,192 max_tokens, 60 s | the configuration that failed 16 of 17 KG extractions; fix in T0.7 |

`n/p` = no per-token price is published in the Magpie catalog or its listing; do not estimate one from a vendor page. Every GLM-5.3 route is advertised with 1M ctx and 128K max out; the request cap in the TUI (65,536) still binds first.

Cross-check on prices: Opus 5.5 is `$4/$20` list with `$0.20` cache read and 1.0M ctx (OpenRouter, released 2026-09-22), which matches the Magpie row exactly. DeepSeek V4.1 Flash is `$0.12/$0.80` at Morph and `$0.09/$0.18` at one aggregator against Magpie's `$0.15/$0.60` - the *billed* rate is Magpie's; never compare gateway quotes across providers.

Request cap: the TUI catalog applies `maxTokens=65536` per request. The max-out column is a ceiling, not a per-request setting; a 393K or 1M output value is unreachable unless the request raises it, and no extraction call should need more than 65,536 output tokens. Shard before you raise the cap.

### 1.2 Capability evidence

| Model | Independent | Vendor or secondary | Verdict |
|---|---|---|---|
| Gemini 4 Carbon | AA Intelligence Index v5 = **59, rank 1** (reported 2026-10-28; general availability 2026-11-09) | Google: #1 on CursorBench 4.0 (Cursor reports 60.4), WANDR, AutomationBench; >2x lower cost per task; agentic-honesty 92% vs 70% for Argon; 98% refusal to fill missing experimental data | Best available design model. Vendor numbers excluded from ranking. If the honesty claim holds, it is the right model for invariant 3 (no claim without a quote). |
| Claude Opus 5.5 | AA Index v5 = **58, rank 2**; Terminal-Bench 4.0 = **60.0%** at max, the best independent agentic-coding row | Moonshot's comparison table puts it at 66.4% (self-report) | Strongest *measured* coding and review model. 1.0M ctx. |
| Kimi K3.1 Pro | AA Index v5 = **55, rank 4**; AA-LCR = **86.3%** (2026-11-18) | Moonshot: Terminal-Bench 4.0 65.7% max at $5.25/task, 52.8% high at $4.16/task | Long-context auditor. Vendor TB4 65.7 sits 5.7 points above Opus's independent 60.0; treat it as a claim, not a measurement. |
| Gemini 4 Argon | AA Index **v4.3** = 50.4 high; TB4 57.1 high | Google: DeepSWE v1.1 77.9 vs Opus 74.2 | Sibling of Carbon. Use only for family diversity, never as a rank; v4.3 is not comparable to v5. |
| DeepSeek V4.1 Flash | AA Index v5 = **32** (tied with GLM-5.3); TB4 = **27.0%** max | DeepSeek tables: DeepSWE v1.1 74.2 vs V4-Pro 62.7; FrontierCode 2.0 28% | Bulk model. Correct for drafting and normalization, wrong for design or review. |
| GLM-5.3 | AA Index v5 = **32**, tied with Flash; no independent TB4 row found | Zhipu: no public row fetched | A different family at Flash-level quality. Right for cross-family review of mechanical artefacts and for the docs-eval second family; wrong for the constitution, the ontology, or the verdict. |
| DeepSeek V4 Pro | TB4 = **14.0%** max (AA) | No V4.1 Pro exists; the id still serves V4-Pro-0813 | Legacy. Do not route work here and do not use it as a reviewer for flash output (same family). |
| Devin SWE-2 | FrontierCode 1.1: 50.0% max ($1.18/task), 47.2 high, 43.1 medium; 62.5% on Devin at 5 trials (different config) | Cognition/Klu; no token API | Code agent only. Cost is per task, not per token; lineage unknown. |
| gemma4-12b | *no public benchmark found* | Google card: MMLU Pro 77.2, GPQA Diamond 78.8, LiveCodeBench v6 72.0, AIME 2026 77.5 (8-bit MLX card) | Local fallback only. 40K ctx and Q4 cap it at chunk level. |
| qwen3.5-4b | *no reliable benchmark found*; one Q4 GGUF card reports MMLU 35.1, ARC 58.7 (low confidence) | - | Trivial classification only; 16K ctx. |
| glm-ocr | *no public benchmark found* | - | OCR fallback for pages where native extraction fails. |
| bge-m3 | MTEB-era (2024); not re-run here | - | Embeddings fixed for the round; do not switch mid-experiment. |

### 1.3 What the evidence supports

- **Rank, then route.** v5 order is Carbon 59 > Opus 58 > GPT-6 Astra 57 > Kimi 55 >> Haiku 5.5 36 > DeepSeek V4.1 Flash 32 = GLM-5.3 32. The current default (flash everywhere) is a 32-index model writing the documents that decide the paper claim.
- **Three claims to strike or narrow:** "orchestrator and doc drafter = flash" is fine for drafts, not for CONSTITUTION, PRAXIS, EVALUATION, or the contradiction audit; "independent reviewer = flash" is self-review; "second family = gemma4-12b" only works when the writer is *not* Google (Gemma is Google) — GLM-5.3, added 2026-11-19, closes that gap on every wire and is now the default second family.
- **Index versions are not comparable.** Argon 50.4 is v4.3 (2026-09-07); Carbon 59 is v5 (2026-10-28). Same-family, different index, 8.6 points apart. Any table that mixes them is wrong.
- **Benchmark rows are configuration-bound.** The independent TB4 table lists Kimi **K3** at 13.0% and has no K3.1 row; Moonshot reports 65.7% for K3.1. A 50-point swing between consecutive versions in one family means a single TB4 number cannot select a model.
- **Cost is not the binding constraint at this scale.** A 720-call extraction pilot costs $1.30 on flash, $19.44 on Carbon or Kimi, $38.88 on Opus (est., §2.5). All are small against four days of human review. Freeze the extraction arm on schema validity, not on the $30 difference; use flash for bulk because it is 15x cheaper *and* sufficient for mechanical work, not because the budget breaks.
- **The real extraction constraint is schema breadth.** ExtractBench (35 PDFs, 12,867 fields) finds frontier models unreliable on realistic schemas and **0% valid output on a 369-field schema for every model tested**. No public PDF-to-JSON benchmark covers the 2026 models above, so shard to <=20 fields per call and let T0.6 measure first-pass validity. Treat local PL extraction quality as unknown.
- **`max` effort is not free.** Kimi buys 12.9 TB4 points (52.8 to 65.7) for +26% cost per task. Use high for design and low or none for JSON extraction; reasoning tokens eat the output budget, which is the exact nmem failure mode.

## 2. Task routing

### 2.0 Callability gate — **resolved 2026-11-19 ~03:45**

Path 1 landed. `list_subagent_models` now returns `magpie` and `magpie-anthropic` (the former `deepseek-official` route is gone), and three real two-token child calls returned `OK`: `magpie/devin/glm-5-3`, `magpie/auto/glm-5.3`, and `magpie-anthropic/zcode/GLM-5.3`. `magpie/zcode/GLM-5.3` is refused by the allowlist — zcode lives on the anthropic wire.

What that changes:

| Before | Now |
|---|---|
| flash was the only authorized route; every reviewer an F row was same-family | C, A, K, O, S, G rows are callable as child and workflow routes |
| "second family" was gemma4-12b or nothing | GLM-5.3 (Zhipu) is a real, cheap second family; zcode is the preferred route while quota holds |
| T0.8b was a proposal | T0.8b is now the *record* of a passing smoke test; keep it as the re-check whenever a route changes |

Residual rules: (a) `auto/*` ids are routing groups — assert the served model in the response, not the requested id (R1/R24); (b) zcode, devin/glm-5-3 and auto/glm-5.3 are the same model and are **not** independent of each other (R23); (c) a Magpie 429 is an upstream quota event, record it as a blocker, never as a model failure.

### 2.1 Route codes and rules

| Code | Route | Use |
|---|---|---|
| F | Magpie `deepseek/deepseek-flash` | drafting, normalization, transcription, scaffolding |
| P | `deepseek/deepseek-v4-pro` (V4-Pro-0813) | legacy fallback only |
| C | `auto/gemini-4-carbon` | design, adjudication, constitution, ontology, evaluation, adversarial scenarios |
| A | `auto/gemini-4-argon` | diversity only, never a rank |
| K | `kimi/k3.1-pro` | independent review, long-context audit |
| O | `devin/claude-opus-5-5` | coding, cross-family review, second opinion on design |
| Gz | `zcode/GLM-5.3` (magpie-anthropic) | cheap cross-family review, docs-eval second family, fan-out QC — **preferred GLM route** |
| Gd | `devin/glm-5-3` (magpie) | Gz fallback when the ZCode quota is exhausted |
| Ga | `auto/glm-5.3` (magpie routing group) | last-resort GLM fallback; assert the served model |
| S | `devin/swe-2` | code agent only; unverified for chat-style JSON |
| G12 | local `gemma4-12b` | privacy lane and offline fallback, chunk level only |
| G4B, Q4 | local `gemma4-e4b`, `qwen3.5-4b` | trivial triage, query rewriting |
| OCR | local `glm-ocr` | failed-page OCR only |
| H | human | gold, verdict, spend, claim, precedence approval |
| - | no model | deterministic shell, Rust, or string matching |

Rules, in priority order:

1. One writer family per artefact. The reviewer must be a different family; if it is not, the record says *same-family* and a human adjudicates (todo.md T0.8 already requires this rule in METHOD.md).
2. No model on gold, final verdict, spend approval, or the paper claim. A model may format a human decision, never make it.
3. Extraction arm is frozen across U2/S2/P2. One model, one snapshot, one prompt, one chunker; a mid-run swap voids the round.
4. Effort is part of the snapshot. Record model id *and* effort *and* temperature in the manifest; a silent default change is an arm change.
5. Bulk work goes to F only when the output is mechanically checkable (quote match, JSON schema, count). If the output needs judgment, it goes to C or K.

### 2.2 Workstream assignment

The gate in §2.0 is resolved, so the reviewer column is executable. The standing rule does not change: the reviewer must be a different family from the writer, and a same-family review is labelled, never called independent.

| W | Primary | Independent reviewer | Human gate |
|---|---|---|---|
| W0 baseline | F for boilerplate, C for precedence | Gz callability, C second opinion | T0.6 freezes the extraction arm; T0.8 approves the second family |
| W1 spine | C (F drafts prose) | K for the spine audit (1M ctx); Gz per-document | constitution and owner map sign-off |
| W2 praxis | C schema, F normalization | Gz quote spot-check; K on contested records | 14 records accepted; unresolved quotes adjudicated |
| W3 project docs | F, C for EVALUATION.md | O; Gz for the lint review | each doc's owner accepts; docs lint green |
| W4 checks | O, F scaffolding | C design check; Gz reads the ten checks for gaps | 10 seeded defects each turn red |
| W5 docs eval | C design, F run | **Gz as second family** (was K) | gate sign-off |
| W6 parse | no LLM; OCR on failed pages | - | parse failures recorded, not cleaned |
| W7 pilot | frozen arm (F unless T0.6 says G12) | Gz for QC only | blind score and gate by H |
| W8 held-out | same frozen arm | Gz as secondary judge only | verdict by H, including inconclusive |
| W9 identity/HawDB | F | C; Gz on the test assertions | identity and rebuild tests |
| W10 roadmap/paper | C, K attacks the plan | Gz as second attacker | claim approved by H |

### 2.3 Task table

Tier is estimated total cost per task execution: T0 $0, T1 <$0.10, T2 $0.10-1, T3 $1-10, T4 >$10. Estimates use the token assumptions in §2.5. Each code is the exact model id from §2.1 (C = `auto/gemini-4-carbon`, O = `devin/claude-opus-5-5`, F = `deepseek/deepseek-flash`); no row names a model class.

| Task | Model | Effort | Why | Tier | Fallback |
|---|---|---|---|---|---|
| T0.1 branch/worktree | - | - | git only; no text to generate | T0 | - |
| T0.2 docs hash | - | - | `shasum -a 256` is deterministic | T0 | F if scripted |
| T0.3 templates | F | low | boilerplate with required fields; human checks one example | T1 | Q4, reject on missing fields |
| T0.4 precedence | F draft, C adjudicate | low, high | one paragraph, but it governs every later doc | T1 | O |
| T0.5 model inventory | - plus F formatting | low | facts come from endpoints; extend models.md with the §1.1 Magpie rows | T1 | - |
| T0.6 structured-output smoke test | F, G12, one C sample | none, off, low | measure both lanes on the same 3 cases; the C sample prices the design model on JSON | T1 | C if F fails schema |
| T0.7 nmem budget | - | - | config edit; keep credentials out of prompts | T0 | H |
| T0.8 reviewer independence | C or K decision plus callability smoke test | high | family choice and a real call decide whether review is independent | T1 | H sign-off |
| T1.1 constitution | C | high | 7 invariants, each with falsifier and check | T1 | O |
| T1.2 status | F | low | generated from tests and the gate ledger; no judgment | T1 | - |
| T1.3 method | F draft, C review | low, high | procedure mostly exists; precedence is new | T1 | O |
| T1.4 glossary | F | low | term normalization against ontology.md | T1 | - |
| T1.5 owner map | C | medium | structural adjudication, one owner per subject | T1 | K review |
| T1.6 guidelines | C | medium | each procedure ends in a check and a stop rule | T1 | O |
| T1.7 ADRs 0001-0006 | C | high | six decisions each need a rejected alternative | T2 | O |
| T1.8 spine review | K | high | different family from the writer; 1M ctx fits the spine | T2 | O |
| T2.1 praxis schema | C | high | new namespace plus 7 relations; ontology budget | T1 | O |
| T2.2 normalize 14 lessons | F x4 batches | low | mechanical; quotes checked by script, not by the model | T1 | P |
| T2.3 validate provenance | - | - | exact substring against the raw thread | T0 | H on exceptions |
| T2.4 nmem mapping | F | low | index pointers only; no duplicate memories | T1 | - |
| T2.5 eval seeds | C | medium | five scenarios need expected decisions and forbidden actions | T1 | K |
| T3.1 architecture | F draft, O review | low, high | content exists in README and engineering.md | T2 | C |
| T3.2 data model | F | low | schema section generated from code | T1 | O |
| T3.3 pipeline | F | low | lifecycle already written in README | T1 | C |
| T3.4 evaluation | C | high | pre-registration, arms, gates; claim-bearing | T2 | O |
| T3.5 operations | F | low | runbook from engineering.md | T1 | C |
| T3.6 experiments | F | low | ledger transcription; no new claims | T1 | - |
| T3.7 roadmap | C | medium | each item tied to a gate or competency question | T1 | K |
| T3.8 review | F | low | short doc; does not restate method | T1 | C |
| T3.9 docs lint | F writes, script runs | low | the lint is deterministic; the model only writes it | T1 | - |
| T4.1 rule-to-check matrix | F x14, C dedupe | low, high | fan-out per lesson; dedupe needs judgment | T2 | K |
| T4.2 ten checks | O | high | code plus a seeded defect each; best measured coder | T4 | S if callable, else F scaffold |
| T4.3 CI wiring | F | low | YAML plus the `pg/**` branch trigger | T1 | - |
| T4.4 docs point to checks | F | low | mechanical link insertion | T1 | - |
| T5.1 fifteen scenarios | C | high | five adversarial cases decide the gate's power | T2 | O |
| T5.2 prompt and rubric | C | high | rubric must be scoreable without a judge | T2 | K |
| T5.3 variants | F | low | generate the encoded AGENTS.md variant | T1 | - |
| T5.4 run matrix | F x45 via workflow | low | deterministic scoring in the main process; no judge | T3 | K second family |
| T5.5 gate | - | - | thresholds pre-registered before the run | T0 | H sign |
| T6.1 admission | - | - | signature and SHA-256; a model adds nothing | T0 | F writes the test |
| T6.2 parse four PDFs | - native; OCR per failed page | - | page mapping is deterministic; OCR only where text is absent | T1 | glm-ocr |
| T6.3 chunk table | - | - | schema, offsets, hashes | T0 | - |
| T6.4 quote recovery | - | - | exact substring plus offset validity | T0 | H sets the bar first |
| T6.5 gate ledger | F transcribe, H approve | low | a *no* row is not a pass | T1 | - |
| T7.1 twenty questions | C | high | four unanswerable cases and forbidden inferences | T2 | O |
| T7.2 one resolver | F | low | alias table is the only difference between arms | T1 | O review |
| T7.3 claim/evidence tables | F | low | schema plus review states plus run id | T1 | O |
| T7.4 scorer and seeds | F code, H rubric | low | deterministic; no LLM judge in the headline | T2 | - |
| T7.5 run U2/S2/P2 | frozen F, temp 0 | low | one arm across three arms; snapshot in manifest | T3 | G12 if T0.6 says so |
| T7.6 blind score | H | - | gold is human; a model may pre-format only | T0 | K pre-format |
| T7.7 gate | H | - | the decision changes the paper claim | T0 | - |
| T8.1 held-out run | frozen arm | low | 11 papers once; preserve failures | T3 | - |
| T8.2 blind review | H | - | randomize order; blind to arm | T0 | K pre-screen only |
| T8.3 downstream QA | H plus deterministic scorer | - | citation precision and recall are string-computable | T2 | K secondary judge |
| T8.4 verdict | H | - | inconclusive is allowed | T0 | C wording only |
| T9.1 re-import drill | F | low | code plus assertions on a cloned library | T2 | O review |
| T9.2 derived-DB sync | F | low | fail-closed count guard | T2 | - |
| T9.3 durable key | F | low | mapping plus test | T1 | O |
| T9.4 COW branches | F | low | snapshot id recorded in the manifest | T2 | - |
| T9.5 exact vs ANN | F | low | decide from measurement only | T2 | - |
| T9.6 graph algorithms | F | low | record query plans, not impressions | T2 | - |
| T9.7 context budgets | F | low | deliberately over-budget and measure the failure | T1 | - |
| T9.8 staging to canonical | F | low | one transaction, idempotent by run id | T2 | O review |
| T10.1 feature roadmap | C | medium | now/next/later tied to gates | T1 | K |
| T10.2 paper outline | C | high | contributions, negative results, threats to validity | T2 | O, claim by H |
| T10.3 pre-mortem | K | high | a different family must attack the plan | T2 | C |
| T10.4 status update | F | low | record specified vs observed | T1 | - |

**GLM-5.3 assignment (added 2026-11-19 ~03:45).** Use Gz for: the T0.8 callability record; every reviewer cell marked Gz in §2.2; the docs-eval second family (T5.3/T5.4); spot-checking the 14 normalized lesson records (T2.2); and the second read over the ten checks (T4.2). Do not use Gz for: the constitution, ontology, evaluation protocol or paper outline (those stay C), the spine audit (K), the authoring of the ten checks (O), the extraction arm (frozen at T0.6), or any human-owned gate. GLM-5.3 indexes at 32 — the same tier as Flash — so it adds *family diversity*, not design strength. When a Gz route returns 429, fall back to Gd then Ga and record the route change in the run manifest (§2.1 rule 4).

### 2.4 Independence matrix

| Writer | Independent reviewers | Not independent |
|---|---|---|
| F or P (DeepSeek) | C, A, K, O, Gz/Gd/Ga, G12 (Gemma is Google) | P, any `deepseek-*` |
| C or A (Google) | K, O, F, P, Gz/Gd | A (sibling), G12, G4B (Gemma) |
| K (Moonshot) | C, A, O, F, P, Gz | none here |
| O (Anthropic) | C, A, K, F, P, Gz | none here |
| Gz, Gd, Ga (Zhipu GLM-5.3) | C, A, K, O, F, P, G12 | the other GLM routes — same model behind a different gateway |
| G12, G4B (Google) | K, O, F, P, Gz | C, A |
| S (Devin) | unknown lineage: cannot certify | everything |

Consequences: a Carbon-written constitution is reviewed by K or O, never by Argon. A flash-drafted method is reviewed by C. Gemma is a *second family* only for non-Google writers. **GLM-5.3 is one model behind three routes:** zcode reviewing devin/glm-5-3 output is self-review with extra steps, so cite the route family, never the gateway, when claiming independence.

### 2.5 Cost arithmetic (est.)

Assumptions: 10K input and 2K output per docs-eval call; 6K and 1.5K per extraction chunk; 60 chunks per paper; no cache hits. Prices from §1.1.

| Run | Calls | Tokens in/out | F | C or K | O | P |
|---|---|---|---|---|---|---|
| docs eval, 15 scenarios x 3 variants | 45 | 450K / 90K | $0.12 | $1.80 | $3.60 | $0.48 |
| develop pilot, 4 papers x 3 arms x 60 chunks | 720 | 4.32M / 1.08M | $1.30 | $19.44 | $38.88 | $4.99 |
| held-out, 11 papers x 60 chunks | 660 | 3.96M / 0.99M | $1.19 | $17.82 | $35.64 | $4.57 |
| 14 lessons, 4 batches | 4 | 32K / 12K | $0.01 | $0.18 | $0.37 | $0.05 |

Reading: output tokens dominate, so *effort* changes cost more than the model tier does. Cache reads at 5% of prompt price make the A/C variants of the docs eval cheaper than the table if A runs before C. At 15 papers, every option is affordable; choose on validity and review burden.

GLM-5.3 is deliberately absent from the cost columns: no per-token price is published for the zcode, devin or auto routes. Budget it as **unpriced** until Magpie bills a real run, and do not substitute a vendor list price — the gateway is what invoices.

### 2.6 Local and privacy lane

- Local is the only lane that keeps PDF text off LAN and WAN. Use it when the human marks a paper sensitive; record the decision in the run manifest.
- Gate before any local extraction: >=95% first-pass schema validity and >=98% exact quote recovery on the three T0.6 cases. Below that, local is a parsing aid, not an extraction arm.
- 40,960 context and Q4 mean chunks <=8K tokens. No full-paper prompts, no 45-agent fan-out on one GPU.
- glm-ocr only for pages where native text is missing; record page ids and never silently replace extracted text.

## 3. Worktree plan

### 3.1 Preconditions

Run once, in the main worktree, before any `git worktree add`. C0a freezes the untracked docs; C0b is the W0 result that becomes the shared base.

~~~bash
cd /Users/emliunix/Documents/paperfs.rs
git switch -c pg/w0-baseline migrate-paper-graph-rust
git add paper-graph/knowledge-system
git commit -m "docs(knowledge-system): freeze design corpus at f1e46d9"

# T0.2-T0.5 on this branch: templates, hash manifest, precedence, models.md, sha2
git add paper-graph/knowledge-system paper-graph/_runs paper-graph/Cargo.toml Cargo.lock
git commit -m "chore(w0): templates, docs hash, precedence, model inventory, sha2"
git tag -a pg-base -m "paper-graph knowledge baseline 2026-11-19"
~~~

Notes: `pg/w0-baseline` is based on the checked-out branch, which equals `main` at f1e46d9; do not touch `main`. W0 adds `sha2` and `hex` to `paper-graph/Cargo.toml` so W4 and W6 never fight over the manifest; W6 alone adds the parser dependency later. `_runs/` is pinned to `paper-graph/_runs/` because every other path in todo.md (`experiments/`, `tests/`) is crate-relative; T0.4 records this in method.md.

### 3.2 Trees

`WTROOT=/Users/emliunix/Documents/paperfs.rs-wt`, outside the repo so `git status` stays clean.

| # | Path | Branch | Base | Created | Writes | Tasks |
|---|---|---|---|---|---|---|
| 1 | `$WTROOT/integration` | `pg/integration` | `pg-base` | wave 1 | `_runs/**`, STATUS and ROADMAP final edits | T5, T7, T8, T10 |
| 2 | `$WTROOT/w1` | `pg/w1-spine` | `pg-base` | wave 1 | spine docs | T1.1-T1.8 |
| 3 | `$WTROOT/w6` | `pg/w6-admission` | `pg-base` | wave 1 | parser code, `gates.md`, `decisions.md` | T6.1-T6.5 |
| 4 | `$WTROOT/w2` | `pg/w2-praxis` | integration tip after W1 | wave 2 | PRAXIS, lessons.jsonl, eval seeds | T2.1-T2.5 |
| 5 | `$WTROOT/w3` | `pg/w3-project` | same | wave 2 | `paper-graph/docs/**`, docs lint | T3.1-T3.9 |
| 6 | `$WTROOT/w4` | `pg/w4-checks` | integration tip after W3 | wave 3 | `tests/check_*`, CI, owner-map check column | T4.1-T4.4 |
| 7 | `$WTROOT/w9` | `pg/w9-identity` | integration tip after W6 | wave 3 | store/zotero, HawDB memo | T9.1-T9.8 |

Why not pre-create all seven at `pg-base`: every wave-2 and wave-3 tree would start without its dependency's files and immediately rebase, which manufactures the conflicts this plan exists to avoid. Empty worktrees cost nothing, but stale bases cost merges. Create a tree when its dependency is green on `pg/integration`.

W5, W7 and W8 have no tree of their own: they consume frozen artefacts and write only `_runs/`. Giving them a tree would let them edit docs mid-run, which the freeze rule in §4.4 forbids.

Four trees are active at once at most (wave 3: W4 and W9 plus finishing W6 and integration runs), matching the guardrail of four concurrent writers.

### 3.3 File ownership

Paths relative to the repo root. Disjoint within every wave; entries marked *after* are sequential handoffs, not concurrent writes.

| Tree | Owns | Must not touch |
|---|---|---|
| W0 | `paper-graph/knowledge-system/_templates/**`, `models.md`, `method.md` (precedence paragraph), `paper-graph/_runs/2026-11-19/docs.sha256`, `paper-graph/Cargo.toml` (sha2, hex), `Cargo.lock` | any other knowledge-system file |
| W1 | `knowledge-system/{CONSTITUTION,STATUS,GLOSSARY,GUIDELINES}.md`, `knowledge-system/method.md`, `knowledge-system/README.md` (index only), `knowledge-system/docs/owner-map.md`, `knowledge-system/ADR/**` | `paper-graph/docs/**`, `_templates/**`, any code |
| W2 | `knowledge-system/PRAXIS.md`, `knowledge-system/lessons.jsonl`, `knowledge-system/eval-seeds/**`, `paper-graph/_runs/2026-11-19/praxis-validation.md` | `knowledge-system/method.md`, `paper-graph/docs/**`, templates (read-only) |
| W3 | `paper-graph/docs/**`, `paper-graph/scripts/docs_lint.sh` | `knowledge-system/**`, `tests/**`, CI |
| W4 | `paper-graph/tests/check_*.rs`, `paper-graph/tests/common/**`, `paper-graph/scripts/verify.sh`, `paper-graph/scripts/seeded_defects.sh`, `.github/workflows/rust.yml`, `knowledge-system/docs/owner-map.md` (check column, *after* W1) | `src/**`, `Cargo.toml`, `gates.md` |
| W6 | `paper-graph/src/{parse,chunk,pipeline,lib,main,cli}.rs`, `paper-graph/Cargo.toml` and `Cargo.lock` (parser), `paper-graph/tests/{admission,parse,chunk}.rs`, `paper-graph/experiments/ontology-extraction/{gates,decisions}.md`, `paper-graph/_runs/2026-11-19/parse/**` | `store.rs`, `zotero.rs`, `tests/check_*` |
| W9 | `paper-graph/src/{store,zotero}.rs`, `paper-graph/tests/{identity,rebuild_drill}.rs`, `paper-graph/experiments/hawdb-utilization.md`, `paper-graph/_runs/2026-11-19/hawdb/**` | `parse.rs`, `chunk.rs`, `pipeline.rs`, `Cargo.toml` |
| Integration | `_runs/**` except dated parse/praxis subdirs, `knowledge-system/STATUS.md` (T10.4), `paper-graph/docs/ROADMAP.md` (T10.1 *after* W3), `knowledge-system/README.md` (T10.4) | everything owned by an open tree |

Conflict-relieving decisions: (a) W4 uses integration-test binaries only, so no `Cargo.toml` edit and no `Cargo.lock` churn; (b) W6 owns `cli.rs` and `lib.rs` while it is open, and W9 inherits them only after W6 merges; (c) `gates.md` has exactly one writer (W6) and one reader (W3); (d) W1's spine docs never link to files that W3 has not created yet - links are added by T4.4 after both merge.

### 3.4 Creation commands

Wave 1, from the tag:

~~~bash
export WTROOT=/Users/emliunix/Documents/paperfs.rs-wt
mkdir -p "$WTROOT"
cd /Users/emliunix/Documents/paperfs.rs
git worktree add -b pg/integration "$WTROOT/integration" pg-base
git worktree add -b pg/w1-spine    "$WTROOT/w1"        pg-base
git worktree add -b pg/w6-admission "$WTROOT/w6"       pg-base
~~~

Wave 2, after W1 is rebased and merged:

~~~bash
git -C "$WTROOT/integration" merge --ff-only pg/w1-spine
git worktree add -b pg/w2-praxis  "$WTROOT/w2" pg/integration
git worktree add -b pg/w3-project "$WTROOT/w3" pg/integration
~~~

Wave 3, after W3 and W6 respectively:

~~~bash
git -C "$WTROOT/integration" merge --ff-only pg/w3-project
git worktree add -b pg/w4-checks "$WTROOT/w4" pg/integration
git -C "$WTROOT/integration" merge --ff-only pg/w6-admission
git worktree add -b pg/w9-identity "$WTROOT/w9" pg/integration
~~~

Build cache: use one target directory for all trees. Concurrent cargo invocations serialize on its lock but reuse artefacts, which is cheaper than five copies of a 12 GB `target/` on a disk with 111 GB free.

~~~bash
export CARGO_TARGET_DIR=/Users/emliunix/Documents/paperfs.rs/target
~~~

### 3.5 Sequential handoffs

| File | First writer | Later writer | Rule |
|---|---|---|---|
| `method.md` | W0 (precedence paragraph) | W1 (T1.3) | W0 commits before W1 branches; W1 owns it afterwards |
| `STATUS.md` | W1 (T1.2) | integration (T10.4) | W10 updates numbers, never the structure |
| `paper-graph/docs/ROADMAP.md` | W3 (T3.7) | integration (T10.1) | T3.7 writes the skeleton and gate links; T10.1 fills later/now |
| `owner-map.md` | W1 (T1.5) | W4 (T4.4) | W4 edits only the check column |
| `gates.md` | W6 (T6.5) | - | W3 reads; a lint failure is fixed by W6, not by the linter |
| `cli.rs`, `lib.rs` | W6 | W9 (*after* merge) | never open both trees at once |
| `Cargo.toml`, `Cargo.lock` | W0 (sha2, hex) | W6 (parser) | one holder per wave; W4 and W9 never edit |

## 4. Merge, verification, cleanup

### 4.1 Merge order

Merge when dependencies are green, not by number: W1 -> W6 -> W2 -> W3 -> W4 -> W9. W1 unblocks W2 and W3; W6 is the longest pole and unblocks W9; W4 needs W3; nothing needs W4 except W5, which runs in integration.

Each feature branch rebases onto the integration tip, passes its own checks, then fast-forwards. Integration is never rebased and never gets a direct commit except the merge itself.

~~~bash
git -C "$WTROOT/w1" rebase pg/integration
(cd "$WTROOT/w1" && bash paper-graph/scripts/docs_lint.sh)   # once W3 has merged the lint
git -C "$WTROOT/integration" merge --ff-only pg/w1-spine
~~~

For W1, before the lint exists, use the local checks in §4.3.

### 4.2 Conflict rules

1. Documentation conflict: the file's owner wins. The losing branch re-applies its delta as a new commit after the rebase; it is never force-pushed over the merge.
2. Never `-X ours` or `-X theirs` on docs. Both discard a rule silently, which is the defect the owner map exists to prevent.
3. `Cargo.lock` is never hand-merged: take the integration copy, regenerate, then stage. During a rebase `--ours` is the integration side, which is the copy to keep.
   ~~~bash
   git checkout --ours Cargo.lock && cargo metadata --format-version 1 >/dev/null && git add Cargo.lock
   ~~~
4. A conflict that reveals two owners for one file is a plan defect: stop, add the ownership row, then resolve. Do not improvise ownership mid-rebase.
5. Case-only path conflicts (`METHOD.md` vs `method.md`) abort the rebase; rename the file to the existing lowercase path and rerun.
6. A tree that adds a file another tree also plans to add loses: the later tree links to the file instead. Never create two copies of one doc.
7. If a rebase needs more than two rounds of manual resolution, abandon the branch and re-run the task in integration. Rework is cheaper than a tangled history.

### 4.3 Per-tree checks

| Tree | Command | Pass condition |
|---|---|---|
| W1 | `grep -c 'Falsifier' knowledge-system/CONSTITUTION.md` and `ls knowledge-system/ADR/000{1..6}-*.md` | 7 falsifier lines (one per invariant), 6 ADRs, every invariant names a check id or *pending* |
| W2 | `jq -s 'length' knowledge-system/lessons.jsonl` and a status-presence check | 14 records, every status present, 0 unresolved quotes |
| W3 | `bash paper-graph/scripts/docs_lint.sh` | 0 broken links, 0 duplicate rule ids, 0 stale commands, 0 case-only twins |
| W4 | `cargo test -p paper-graph --tests` then `bash paper-graph/scripts/seeded_defects.sh` | all green, then every seeded defect exits non-zero |
| W6 | `cargo test -p paper-graph --test admission --test parse --test chunk` | 15/15 admission, 4/4 parse, chunk hashes stable across two runs |
| W9 | `cargo test -p paper-graph --test identity --test rebuild_drill` | re-import preserves claims; sync fails closed on count change |
| Integration | `cargo test --workspace`, docs lint, hash check, `git diff --check` | green and clean |

Docs lint contract (T3.9): relative links resolve; every owner-map subject maps to an existing file and a check id or *judgment*; no two docs carry the same rule id; no stale command (`pytest`, `npm test`, `make test` outside a code fence); no tracked paths differing only by case (`git ls-files | tr 'A-Z' 'a-z' | sort | uniq -d` empty); no `METHOD.md` reference.

Baseline evidence: `cargo test -p paper-graph` is green at f1e46d9 on 2026-11-19 (all non-ignored tests; `existing_database` is `#[ignore]` and needs `PAPER_GRAPH_REAL_DB`). A red tree after a rebase is a real regression, not a flaky baseline.

CI note: `.github/workflows/rust.yml` triggers on push to `main` and PRs to `main` only, so no worktree branch is tested by CI until T4.3 adds `pg/**` to the push branches. Local checks are the only gate until then.

### 4.4 Integration verification

~~~bash
cd "$WTROOT/integration"
git status --porcelain                       # must be empty
git log --oneline pg-base..HEAD
git diff --stat pg-base..HEAD
cargo test --workspace                       # CI parity, includes paperfs_rs
bash paper-graph/scripts/docs_lint.sh
(cd paper-graph && shasum -a 256 -c _runs/2026-11-19/docs.sha256)
git diff --check
~~~

Freeze rule: the docs hash must still match before T5.4, T7.5 and T8.1. Any doc edit after the hash invalidates the run that follows; re-hash deliberately and record why.

### 4.5 Cleanup

Only after `pg-v1` is tagged and merged. Verify each worktree is clean first; `remove` refuses dirty trees, which is the safety net.

~~~bash
cd /Users/emliunix/Documents/paperfs.rs
git tag -a pg-v1 -m "paper-graph knowledge system v1" pg/integration
git switch migrate-paper-graph-rust
git merge --ff-only pg/integration

for t in w1 w2 w3 w4 w6 w9 integration; do
  git worktree remove "$WTROOT/$t"
done
for b in pg/w0-baseline pg/w1-spine pg/w2-praxis pg/w3-project pg/w4-checks pg/w6-admission pg/w9-identity; do
  git branch -d "$b"
done
git worktree prune
git worktree list
~~~

Keep `pg/integration` and the tags until the paper is submitted; delete feature branches immediately. Do not delete `pg-base`: it is the only reference to the frozen docs state.

### 4.6 Rollback

~~~bash
git worktree remove --force "$WTROOT/w4"   # only after checking no evidence is uncommitted
git branch -D pg/w4-checks
git -C "$WTROOT/integration" reset --hard HEAD^           # one bad merge only; otherwise restore from the reflog
~~~

## 5. Risks and unknowns

| # | Risk | Evidence | Impact | Control |
|---|---|---|---|---|
| R1 | `auto/` may be a routing prefix, not a pinned model | Magpie ids `auto/gemini-4-*`; no public doc found | frozen arm is not frozen | assert response `model` field matches; record it in every manifest (T0.6) |
| R2 | `devin/swe-2` may not accept chat/JSON calls | Cognition: no SWE-2 API, per-task pricing | W4 fallback unusable | one smoke call before routing; otherwise O only |
| R3 | SWE-2 `0/0` price read as free | listing only | budget and cost gate wrong | never use; cost per task from FrontierCode |
| R4 | Benchmark version drift | Argon 50.4 (v4.3) vs Carbon 59 (v5); Kimi K3 13 vs K3.1 65.7 | wrong ranking | always print index version and date; never mix |
| R5 | Vendor self-reports treated as independent | Google Carbon claims; Moonshot TB4 65.7 | review validity | label vendor rows; use AA only for rank |
| R6 | Carbon is new (Nov 9) and lightly covered | no AA TB4 row found for Carbon | design model quality unverified on code | T0.6 adds one Carbon sample |
| R7 | Extraction validity collapses with schema breadth | ExtractBench: 0% valid at 369 fields | pilot produces nothing | shard to <=20 fields; validate per call |
| R8 | No public PDF-to-JSON benchmark for these models | search found none | extraction quality unknown | T0.6 is the only admissible evidence |
| R9 | Local PL quality unmeasured | no benchmark; 40K ctx | local lane overclaimed | quality gate in §2.6 before use |
| R10 | GPU contention and cold start | router unloaded, max_instances 4 | long wall-clock, timeouts | measure TTFT and tokens/s in T0.6 |
| R11 | nmem 8K/60 s route | observed config; 16/17 failures | extraction backfill fails | T0.7 before any backfill |
| R12 | Same-family review claimed independent | flash reviews flash by default | weak spine | §2.4 matrix; `method.md` rule |
| R13 | Case-only filename collision | APFS ignorecase; todo.md says METHOD.md | silent edit or duplicate rule | lint check; edit `method.md` |
| R14 | Untracked docs lost by worktree | `git status` shows `??` | empty trees | C0a freeze commit, tag |
| R15 | `_runs/` location ambiguous | todo.md has no root anchor | evidence scattered | pin to `paper-graph/_runs/` in T0.4 |
| R16 | `Cargo.lock` churn from two trees | W4 and W6 both need hashing/parser | merge conflicts | W0 adds sha2/hex; W6 sole manifest owner |
| R17 | Shared target lock serializes builds | 12 GB `target/`, 111 GB free | slow parallel waves | one `CARGO_TARGET_DIR`; accept serialization |
| R18 | CI does not see `pg/**` branches | workflow triggers on `main` only | untested merges | T4.3 adds branch trigger; local `verify.sh` |
| R19 | Docs hash drifts during eval | edits after hash | invalid score | freeze rule in §4.4; re-hash deliberately |
| R20 | Human review capacity unmeasured | 18 seeded cases, 11 held-out papers | schedule slip | time-box in W7; report burden in T8.4 |
| R21 | Advertised max-out exceeds the per-request default | TUI catalog `maxTokens=65536` | truncated JSON, false failures | keep extraction output <=16K; shard batches |
| R22 | Magpie authorization was missing | resolved 2026-11-19: allowlist patch + three passing smoke calls | the routing table ran flash-only | keep T0.8b as the re-check after any future route change |
| R23 | ZCode quota exhaustion (HTTP 429) | observed 2026-11-19 before the re-login | Gz fails mid-run | fall back Gd, then Ga; record the route in the manifest; a 429 is a quota event, never a model failure |
| R24 | `auto/*` ids may be routing groups, not pinned models | R1; `auto/glm-5.3` is a routing group | served model differs from the requested id | assert the response `model`; use pinned zcode/devin ids for anything that must be frozen |
| R25 | One GLM model behind three gateways | zcode, devin/glm-5-3, auto/glm-5.3 are the same weights | a false independence claim | §2.4: cite the family, not the route |

## Appendix A: sources

Tier A = evaluator or vendor primary; B = independent aggregator or reporting; C = secondary repost.

| Source | Tier | Date | Used for |
|---|---|---|---|
| [Artificial Analysis Intelligence Index v4.3](https://artificialanalysis.ai/articles/artificial-analysis-intelligence-index-v4-3) | A | 2026-09-07 | index composition: Terminal-Bench 4.0, AA-LCR v1.1, AA-Omniscience, GDP.pdf |
| [The Next Web: Gemini 4 Carbon tops AI rankings](https://thenextweb.com/news/gemini-4-carbon-google-cybersecurity-model) | B | 2026-10-28 | Carbon 59, Opus 58, GPT-6 Astra 57 |
| [China Daily: Kimi K3.1](https://www.chinadaily.com.cn/a/202610/27/WS6adff3c1e4b06d4aa0564e78.html) | B | 2026-10-27 | Kimi 55 rank 4; vendor TB4 65.7 vs Opus 66.4 |
| [Baidu Baike: DeepSeek-V4.1-Flash-1017](https://baike.baidu.com/item/DeepSeek-V4.1-Flash-1017/69439738) | C | 2026-10-28 | Flash 32, GLM-5.3 32, Haiku 36, FrontierCode 2.0 28% |
| [SXF: Terminal-Bench 4.0 results](https://sxf.si/evaluations/terminal-bench-4.0-aa/) | B | verified 2026-10-08 | Opus 60.0, Flash 27.0, V4 Pro 14.0, Kimi K3 13.0 |
| [Morph: Kimi K3.1](https://www.morphllm.com/kimi-k3-1) | B | 2026-11-09 | Kimi $2/$10; TB4 52.8 high and 65.7 max with per-task cost |
| [BenchLM: AA-LCR](https://benchlm.ai/benchmarks/lcr) | B | 2026-11-18 | Kimi K3 88.7, K3.1 Pro 86.3 |
| [Morph: DeepSeek V4.1](https://www.morphllm.com/deepseek-v4-1) | B | 2026-11-09 | no V4.1 Pro; V4-Pro-0813 still served; DeepSWE 74.2 vs 62.7 |
| [OrcaRouter: Cognition SWE-2](https://www.orcarouter.ai/blog/cognition-swe-2-release) | B | 2026-09-09 | no SWE-2 API; FrontierCode 50.0; 64% cheaper |
| [Klu: FrontierCode 1.1](https://klu.ai/benchmarks/frontiercode) | B | 2026-10-06 | SWE-2 43.1/$0.37, 47.2/$0.78, 50.0/$1.18 |
| [OpenRouter: Claude Opus 5.5](https://openrouter.ai/anthropic/claude-opus-5.5) | A | 2026-09-22 | $4/$20, cache $0.20, 1.0M ctx |
| [Google: Gemini 4 Carbon available](https://blog.google/intl/pt-br/novidades/tecnologia/gemini-4-carbon-disponivel/) | A | 2026-11-09 | CursorBench/WANDR/AutomationBench claims; honesty 92% vs 70%; 98% refusal |
| [BAAI hub: Carbon first-day integrations](https://hub.baai.ac.cn/view/59543) | C | 2026-11-10 | Cursor 60.4%; notes all numbers are Google self-reported |
| [Google Gemma 4 12B MLX card](https://huggingface.co/AtomicChat/gemma-4-12B-it-MLX-8bit) | A | 2026-11-04 | MMLU Pro 77.2, GPQA 78.8, LCB v6 72.0, AIME 77.5 |
| [ExtractBench](https://sinoxiv.napstic.cn/article/25582660) | B | 2026-02-12 | 35 PDFs, 12,867 fields, 0% valid at 369 fields |
| [OpenRouter: Gemini 4 Argon batch](https://openrouter.ai/ko/google/gemini-4-argon:batch) | A | 2026-10-12 | Argon 50.4 index v4.3, TB4 57.1 |
| [The News: Gemini 4 Argon](https://www.thenews.com.pk/latest/1418272-google-launches-gemini-4-argon-as-its-new-frontier-model) | B | 2026-09-30 | DeepSWE 77.9 vs Opus 74.2 |
| [BenchLM: GDP.pdf](https://benchlm.ai/benchmarks/gdppdf) | B | 2026-11-09 | document understanding is a separate, harder axis; no Carbon row |
| [BenchLM: AA-Omniscience hallucination](https://benchlm.ai/benchmarks/omnisciencehallucinationrate) | B | 2026-11-07 | Argon 4.3% best hallucination rate |
| [BenchGecko: DeepSeek V4 Flash](https://benchgecko.ai/model/deepseek-v4-flash) | C | 2026-11-13 | list price $0.09/$0.18 per 1M; cross-check only, not billed |

## Appendix B: what this plan does not claim

- No benchmark here was run on this repo's corpus. Every capability number is borrowed and configuration-bound.
- No cost figure includes reasoning-token variance, retries, or cache misses; treat all $ as planning estimates.
- No claim that the Magpie gateway serves the exact snapshot the vendor scored; assert the returned model id.
- No claim that worktrees prevent rule duplication; they prevent file conflicts. The owner map and docs lint prevent duplication, which is the more likely defect in this project.
