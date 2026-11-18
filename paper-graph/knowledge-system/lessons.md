# Lessons for the paper knowledge system

Fourteen rules distilled from 18 Nowledge Mem threads, the paper-graph repository at commit f1e46d9, and three audit rounds. This is **design knowledge**, not an experimental result: the ontology-extraction round has not run, no PDF has been parsed, and no gold annotations exist.

Each lesson is tagged by what kind of evidence supports it:

- **Observed** — seen in the current code or a measured incident.
- **Decided** — an explicit user decision, not necessarily implemented.
- **Specified** — written in a design document or rubric, not executed.
- **Transferred** — a lesson from another project, not yet tested here.
- **Contested** — a live counterargument or unresolved contradiction.

A lesson can carry several tags. The tag is not a confidence score; it says what would have to change to make the claim stronger.

## The fourteen rules at a glance

| # | Rule | Status |
|---|---|---|
| L01 | Separate canonical, rebuildable, and distilled layers; make the rebuild boundary explicit | Decided · Observed · Specified |
| L02 | Use Zotero IDs as local keys; persist a durable key and evidence coordinates | Decided · Observed · Untested |
| L03 | Manual sync; never let a metadata rebuild delete derived knowledge | Decided · Observed · Specified · Transferred |
| L04 | Admit sources by bytes and signature; metadata is a retrieval ceiling | Observed · Specified |
| L05 | Start ontology from competency questions, not a relation catalogue | Specified · Contested |
| L06 | Grade relation strength and attribution; resolve identity conservatively | Specified · Contested |
| L07 | Make evidence atomic and keep unknown visible | Specified · Transferred |
| L08 | Evaluate first, pre-register one axis, ask questions before building the scorer | Decided · Specified · Unexecuted |
| L09 | Score with a pre-registered panel; strict F1 is one diagnostic | Specified · Contested |
| L10 | An eval is evidence only if it proves execution, records the run, and leaves gold intact | Observed · Transferred · Specified |
| L11 | Validate the evaluator before trusting any verdict | Transferred · Observed (doc-verify) |
| L12 | Derive the extraction budget chain and record per-call provenance | Observed · Transferred · Unexercised |
| L13 | Distil on an explicit decision; record hard lessons, not process narrative | Decided · Observed · Prescribed |
| L14 | Choose a dependency-light backend; migrate by reviewed one-off scripts | Transferred · Contested |

## How the rules fit together

The system has five layers and one lifecycle. L01-L04 define the layers, identity, synchronization, and source admission. L05-L07 define what may enter the graph and how it is evidenced. L08-L11 define how the result is evaluated. L12-L14 define the operational envelope and how the work is remembered.

~~~
Zotero (canonical metadata, PDFs, annotations)
  -> parsed text + line index + chunk/page/offset coordinates
  -> HawDB derived index (SQL rows, graph, BM25, vectors)
  -> reviewed Concept/Claim/Evidence graph (separate derived store)
  -> distilled reading notes and lessons (pointers, not copies)
~~~

The lifecycle is: admit -> parse -> chunk -> extract -> validate -> review -> project -> retrieve -> distil. Each arrow is a place where a status can be lost; the lessons are mostly about not losing it.

## L01 — Separate canonical, rebuildable, and distilled layers — and make the rebuild boundary explicit

**Status:** Decided · Observed · Specified

**Rule.** Zotero is the only canonical home of bibliographic metadata. The HawDB metadata graph is rebuildable derived state. Extraction, human judgment, and run provenance must live in a separate derived store; distilled notes point back to evidence, they do not duplicate it.

**Why it holds.** Store rebuild deletes every work row and then every graph node in one transaction before reinserting from a read-only Zotero snapshot. Anything stored beside the metadata — concepts, claims, annotations, review state — disappears on the next sync. The project already decided that round 1 uses a separate derived HawDB, but no such database or schema exists yet.

**Evidence.**

- thread 71ffc1b7#44: 'Zotero stays the source of truth; hawdb becomes a rebuildable graph + search engine; Nowledge Mem keeps only distilled insights and pointers.'
- src/store.rs:108-109: DELETE FROM works, then MATCH (n) DETACH DELETE n, inside one transaction.
- experiments/ontology-extraction/decisions.md:5 records the separate-derived-database mitigation; data/ still contains only library.hawdb.
- No code links a Nowledge Mem note to a Zotero itemID; the Mem save in this thread was blocked and unconfirmed.

**Boundary.** The third layer is procedural, not implemented. 'Disposable' applies to the metadata graph only; extraction and review state are not disposable. Until the separate DB exists, the rule is an unmet precondition, not a property of the system.

**Operational check.** Before sync, count non-metadata labels/rows in the target DB and fail closed if any exist. Record in every SyncReport the model, URL, token count, elapsed time, run id, and a hash of the source snapshot.

---

## L02 — Use Zotero IDs as local keys, but persist a durable key and explicit evidence coordinates

**Status:** Decided · Observed · Durability untested

**Rule.** Use Zotero numeric itemID as the local join key inside one library, persist zotero_key and DOI as the durable address, and give every evidence span attachment/page/character coordinates. Never use a title or a content hash as paper identity.

**Why it holds.** itemID is the SQL primary key and zotero_key is a non-key column, so search rows, UI URLs, and rebuild joins all depend on a local SQLite row id. A Zotero merge or restore can renumber itemIDs, which would orphan claims and evidence links. Storing the key without ever joining on it gives the appearance of a durable address without the guarantee.

**Evidence.**

- thread 71ffc1b7#49 (user): 'We can reuse the ID in the Zotero library for the identities, so that is fast and we don't need to consider the ID things from, say, Content Hash or whatever.'
- src/store.rs:11-12: item_id BIGINT PRIMARY KEY; zotero_key TEXT NOT NULL, never used in a WHERE clause.
- README.md:12 states itemID identities and no content-hash identity; no test exercises re-import or itemID reassignment.
- No chunk, page, or offset table exists in src/; the paper-index convention in thread 3099ed1f is a reading workflow, not an implemented evidence store.

**Boundary.** itemID is stable only while the Zotero library is unchanged. The line-index convention and character-offset EvidenceSpan are two different address spaces; neither is implemented in paper-graph. The OCaml/paper-notes thread does not corroborate this rule and is not counted as independent evidence.

**Operational check.** Add a key-to-itemID mapping table or make zotero_key the durable join. Add a re-import test that changes itemIDs and asserts evidence/claim survival. Reject evidence lacking attachment id, page, and verified character offsets.

---

## L03 — Make sync manual, and never let a metadata rebuild delete derived knowledge

**Status:** Decided · Observed · Specified · Transferred

**Rule.** Make synchronization a manual, explicit replacement of derived metadata. Never let it delete extraction or human judgment: scope the rebuild to owned metadata labels, or isolate extraction in a separate database, and prove it with a count oracle. No startup table rebuild that drops views or tables.

**Why it holds.** The current sync is a full destructive replace in one transaction; there is no upsert or partial path. The only mitigation is a decision to use a separate DB, not a code guard. In a different SQLite project, an automatic table-rebuild migration was rejected as dangerous and replaced by a reviewed one-off additive statement.

**Evidence.**

- thread 71ffc1b7#45 (user): 'we can make it just one-off synchronization for a proactive cause only, or some menu triggers only.'
- src/store.rs:108-109 and src/pipeline.rs sync_library: every sync runs the full delete-and-reinsert path.
- thread d9cbb41d#62 (user): 'this is dangerous, provide the one off migration sql to me and remove that logic'; #86: 'you have drops in schema? how dare you say it's a migration.'
- experiments/ontology-extraction/decisions.md:5: 'Domain extraction cannot be erased by a metadata sync' — recorded as a requirement, not tested.

**Boundary.** The additive ALTER remedy is proven only on SQLite/bub_sf. HawDB ALTER support is unverified, initialize() only creates the table when absent, INSERT_WORK hardcodes 13 columns, and there is no schema version. A 'table rebuild' and a 'data wipe' are different failures; the current code commits the second.

**Operational check.** Populate a derived DB, run a metadata sync, and assert extraction/claim/evidence counts unchanged; fail closed if derived data is present. Keep a schema version and reconcile columns instead of only creating the table. Forbid DROP TABLE/VIEW in startup paths.

---

## L04 — Admit sources by bytes and signature; treat metadata as a retrieval ceiling

**Status:** Observed · Specified

**Rule.** Admit a paper only after resolving a local file and checking its signature. Treat metadata coverage as a ceiling on retrieval claims. A status string in a corpus file is a tripwire, not an executable admission gate.

**Why it holds.** Corpus planning assumed attachment rows; a filesystem check found 11 of 15 readable PDFs and four dangling attachment records (itemIDs 60, 47, 5, 8). A literal %PDF- check became a separate admission condition. Metadata is thin (117 of 324 works have abstracts; one work is titled only 'PDF' and wins semantic search), so simple topic queries saturate.

**Evidence.**

- thread 71ffc1b7#160: 'So “has an attachment row” is not sufficient for corpus admission.'
- thread 71ffc1b7#133: semantic search ranks itemID 2315, titled 'PDF', first for 'fixed points and recursion schemes'; metadata cleanup or title-aware ranking is required.
- tests/experiment_corpus.rs:15 asserts only the status string pdf_verified_gold_pending; it never stats a PDF, checks %PDF-, or protects gold.
- README.md:10-11: the pipeline indexes metadata and abstracts only; no PDF parser exists.

**Boundary.** Admission is established only to file and signature level, not text recoverability. The corpus JSON stores no attachment id, path, or hash, so admission cannot be re-executed from the artifact. The 15/15 claim is a one-time manual record.

**Operational check.** Executable admission script: resolve attachment, stat file, check %PDF-, record SHA-256; fail on any missing file. Add a parser smoke test on the four development PDFs before annotation. Attach a metadata coverage report to every retrieval result set.

---

## L05 — Start ontology from competency questions, not from a relation catalogue

**Status:** Specified · Contested

**Rule.** Start from pre-registered competency questions. Add a node, relation, or kind only when a question cannot be answered without it. Banning a named IS_A edge is not enough: a closed, single-valued kind enum can relocate the same taxonomy.

**Why it holds.** The v0 design says there is no generic IS_A and makes Concept.kind a presentation field. But the design already carries 9 node types and 17 relations before any downstream question has been frozen, and the DeepSeek critique points out that the closed kind enum still encodes subsumption. The design grew past the recommended five-relation start.

**Evidence.**

- thread 71ffc1b7#137: 'I'd build a small, query-driven property-graph ontology, not a large OWL ontology up front.'
- thread 71ffc1b7#154: keep curated domain concepts separate from paper-derived assertions.
- analysis/llm-document-graph-research.md:162,183 and the 17-row relation table at 199-215.
- analysis/deepseek-ontology-critique.md:36 argues the IS_A ban relocated subsumption into the closed kind enum.
- gates.md:20: 'Downstream gold questions | no'; no questions.json exists.

**Boundary.** Design-only: src/ builds only Work, Person, Collection, Tag, Venue, Attachment; no Concept or Claim node exists. The rule is a target for the pilot, not a description of the current schema. A disputed categorization must stay needs_review rather than earn a new class or edge.

**Operational check.** Freeze the competency questions before freezing the vocabulary. Cut v0 to the relations the questions need (start with five). Grep emitted schema/manual for IS_A, subClassOf, BROADER_THAN; require an explicit cue span for every GENERALIZES. Make kind faceted, multi-valued, and attributed until tested.

---

## L06 — Grade relation strength and attribution; resolve identity conservatively

**Status:** Specified · Contested

**Rule.** Grade every Work→Concept link by the evidence that licenses it: MENTIONS for a bare naming, USES only for the work's own method, and INTRODUCES only for a work-scoped claim of contribution — not priority. Aliases are reversible links; ambiguous references stay unresolved; a merge needs definition-level justification.

**Why it holds.** A passing mention must not become a use or an invention. Automatic MENTIONS are kept separate from reviewed ABOUT/USES/INTRODUCES, and cue-constrained validation blocks role inflation. Aliases carry their own evidence rather than merging nodes, so free monads and freer monads survive as distinct concepts. Pairwise merge precision is gated at .98 and one false merge is reported as a cluster-level incident.

**Evidence.**

- thread 71ffc1b7#138: 'The distinction between MENTIONS, ABOUT, USES, and INTRODUCES is crucial. An extractor should not turn every occurrence of “fixed point” into a claim that the paper introduces fixed-point theory.'
- experiments/ontology-extraction/round-001-rubric.md:30-35 scores roles separately and reports one false merge as one cluster-level incident; line 145 sets precision ≥ .95 and merge precision ≥ .98.
- analysis/deepseek-ontology-critique.md:24 warns that Work INTRODUCES smuggles priority and will be read as 'first'; cue-gating alone does not fix the arity.

**Boundary.** Specification only: MENTIONS/USES/INTRODUCES exist in markdown, not code; no extraction arm has run. The INTRODUCES predicate may need renaming or a non-priority qualifier before it is safe. A qualified alias of two partly overlapping notions still needs a human decision.

**Operational check.** Define INTRODUCES as 'this work claims the contribution' and add a non-priority qualifier. Test the 'Dunfield and Krishnaswami introduce X' case. Demote any USES/INTRODUCES supported only by a token match. Report pairwise merge precision and each false merge's downstream edge count.

---

## L07 — Make evidence atomic and keep unknown visible

**Status:** Specified · Transferred

**Rule.** Admit a claim only as one atomic, independently evaluable proposition bound to a verbatim quote and a resolvable locator. Evidence is a leaf; claim–evidence is many-to-many; overlapping spans dedupe to one support. Missing or unresolved input stays unknown and blocks or demotes — it is never silently filled.

**Why it holds.** Claims are reified and projected as Work ASSERTS Claim, Claim ABOUT Concept, Claim HAS_EVIDENCE EvidenceSpan, with Chunk as a locator. Deterministic checks require the quote to be a substring of the cited raw page and dedupe overlapping spans so one sentence cannot count as independent corroboration. The doc-verify precedent keeps pessimistic and optimistic bounds so PASS holds only if it survives every way the unknowns could resolve.

**Evidence.**

- thread cb2db461#82 (user): 'the evidence should be atomic, and evidence and claims are not 1-1 correspondence.'
- thread cb2db461#86 (user): 'only evidence is leaf, the whole thing is directed graph flattened by reverse dependency order.'
- thread 28b5b22b#26: 'Unknown propagates and never resolves on its own. A verdict is PASS only when it would hold under every way the unknown atoms could be answered.'
- analysis/llm-document-graph-research.md:544,555 defines the EvidenceSpan table and Claim HAS_EVIDENCE projection; rubric lines 82-88 add quote/locator checks and dedup.

**Boundary.** Conditional on a parser and a chunk/page/offset table that do not exist. Today evidence can only be quote-checked against Zotero metadata or abstracts; locators must be marked unavailable rather than assumed. Split vocabulary: an unknown input demotes the affected claim and marks it unreviewed; an incomplete evaluation blocks the run and produces no score.

**Operational check.** Validate every claim's quote as an exact substring of the cited raw page/chunk and its locator to attachment/chunk/page/offset; a fabricated locator is a hard failure. Dedupe overlapping spans. Pre-register abstention for unanswerable questions.

---

## L08 — Evaluate first, pre-register one axis, and ask the questions before building the scorer

**Status:** Decided · Specified · Unexecuted

**Rule.** Build the smallest end-to-end artifact over the real corpus, then pre-register one comparison with frozen inputs and a non-regression baseline. Ask and lint the competency questions before building claim scoring.

**Why it holds.** The first deliverable was search over the real Zotero library (324 works; BM25, exact vector, weighted-RRF hybrid, loopback UI), so the first experiment measures search rather than extraction depth. Round 1 manipulates ontology constraint only (U2/S2/P2), holding parsed text, chunk ids, model snapshot, decoding, order, call budget, JSON envelope, evidence rules, retry, and quote/page validation constant. Because four simple topic probes already return a sensible rank-1 work, they become non-regression controls; the primary downstream metric moves to evidence-supported compositional answers.

**Evidence.**

- thread 71ffc1b7#47: 'I'd make search evaluation the first product, with synchronization reduced to a manual, repeatable command.'
- thread 71ffc1b7#48 (user): 'Sure, it's a good idea to have a proof of work first.'
- experiments/ontology-extraction/round-001-plan.md:18-21,56-59 freezes inputs and requires the manual/prompts/schemas/chunker/model/gates to be frozen before test predictions.
- README.md:150 still calls 12-20 preselected questions 'a useful next evaluation'; no questions.json exists, while plan sections 4-6 and rubric 4-5 already define claim/evidence scoring.

**Boundary.** The plan is pre-registered but unexecuted: gates.md marks plan review, PDF parsing, gold, seeds, extraction, scorer and verdict all 'no'. It also inverts the rule by investing in claim scoring before the retrieval baseline questions are asked. The four simple probes have no headroom and cannot justify the extraction layer.

**Operational check.** Write and lint the competency questions first; freeze hashes for manual, prompts, schemas, chunker, model snapshot and gates before opening test outputs. Record every post-freeze change as a protocol deviation. Keep a gate ledger where an unrun row is 'no', never a pass.

---

## L09 — Score with a pre-registered panel; keep strict F1 as one diagnostic

**Status:** Specified · Contested

**Rule.** Use a pre-registered panel by default: per-relation quality, high-risk violation rate, coverage, evidence validity, downstream QA, cost, and review burden. Keep strict evidenced relation F1 as one diagnostic; allow it as headline only when gold is complete, one relation type dominates, and abstention cannot be rewarded.

**Why it holds.** Strict fully-evidenced relation F1 can hide selective abstention, rare direction errors on GENERALIZES/EXTENDS, and the blast radius of one bad concept merge. The rubric therefore keeps existence, semantic, and strict F1 as factorized diagnostics, adds per-predicate macro-F1, direction confusion matrices, a risk–coverage curve with matched-coverage F1, and a per-merge cluster incident count. A single aggregate driven by common MENTIONS or CITES is not a win.

**Evidence.**

- thread 71ffc1b7#178: 'strict fully-evidenced relation F1 should not stand alone: it can hide abstention, rare high-risk relation errors, direction mistakes, and clustered damage from one bad concept merge.'
- experiments/ontology-extraction/round-001-rubric.md:3-6,140-151 defines the panel and the engineering gates.
- analysis/llm-document-graph-research.md:23 and :643 still say strict evidenced relation F1 is the headline — the report was never amended after the rubric demoted it.

**Boundary.** The panel is an argument, not a measurement: no arm, scorer, or gold exists. The thresholds (.95 precision, .05 margin, US$25, 20 min/paper) are project choices, not community benchmarks. The absolute 'never a single F1' is too strong and is retracted by the rule's own conditions.

**Operational check.** Publish numerator, denominator, and stop threshold per metric before any arm runs. Reject any report whose headline is one aggregate F1. Amend the research report so the primary design document and the rubric do not contradict each other. Re-run every 'complete: yes' gate row from a clean checkout.

---

## L10 — An eval is evidence only if it proves execution, records the run, and leaves gold intact

**Status:** Observed · Transferred · Specified

**Rule.** Treat an eval as evidence only when the run proves it executed every case, records the configuration and raw per-case output that produced the score, and leaves gold untouched. When a model disagrees with gold, gold wins unless a cited skill or decision sentence changes the case. Lint questions and the answer schema as hard as the expected answers.

**Why it holds.** A harness silently skipped any project that declared no model, so every historical eval ran zero cases while appearing to work. A run record that stores only scores cannot be traced to the model or request fields that produced it. A review found that rewording questions to presuppose the answer made trap cases unable to catch the failure they were written for, even though no expected answer moved. A batch partitioner respected a byte cap but exceeded the judge's token budget, so the first sub-batch failed before any semantic call and two operators got opposite call counts.

**Evidence.**

- thread 78dd3c13#352: 'vieval silently skips any project that declares no model, and this one declared none. So every past eval:full run executed 0 cases while appearing to work.'
- thread d64e9502#330: 'A run's results don't record the model or the extra fields used, so a score like “64/126 on deepseek-flash” can't be traced to its configuration.'
- thread e1f865e1#139: reworded questions presupposed the answer; 'That is the same bending your rule forbids, done through the question instead of the expected answer.'
- thread fc1cc7be#849 (user, relaying review): 'partitionBatches packs to maxEvidenceBytes (36000) and the JEV stateTokenBudget is 32000, so the first sub-batch throws the judge state budget before complete.'; #850: opencode reported 4 semantic calls, grok reproduced 0 with exit 3.

**Boundary.** Two independent eval projects plus a third project's status tripwire; not the paper ontology experiment itself. The gold-wins half has no instance in paper-graph: there are no gold files, and tests/experiment_corpus.rs:15 only asserts a status string — it does not protect gold or verify PDF admission. Only one model family was available, so skill findings stay single-family.

**Operational check.** Fail the run when executed != corpus size; any skip exits non-zero; a zero-case run is a harness failure, not a score. Commit one run record per execution with commit, model id, extra body/config, repeat index, per-case expected vs actual, and raw output. Diff expected lines on every case edit; require a cited deciding sentence and a second reviewer. Lint questions for outcome-presupposing wording.

---

## L11 — Validate the evaluator before trusting any verdict

**Status:** Transferred · Observed (doc-verify) · Not independent

**Rule.** Before trusting a verdict, validate the evaluator itself against invariants and an independent reference that shares no code with it. Map timeouts, budget exhaustion, parse or backend failures, and unasked atoms to BLOCKED or unknown, never to PASS.

**Why it holds.** Three layers, cheapest first: invariants over generated programs (certain ⊆ possible; answering another atom never grows the gap; answering every demanded atom collapses the bounds; order independence; PASS implies every error constraint has a certain goal); a naive reference for the deterministic subset; and a three-valued reference for the rest, differentially tested. In doc-verify this found a real false-PASS in count literals and a blocking bug where kernel budget exhaustion returned PASS instead of BLOCKED.

**Evidence.**

- thread 28b5b22b#24: 'The way to get confidence that your own evaluator has not broken the logic is to pin the semantics with an independent second implementation and test the two against each other.'
- thread d0e2d0bd#884: 'Blocking: when the kernel runs out of its time budget, the overall verdict comes out PASS instead of BLOCKED.'
- thread d0e2d0bd#1359: on 24 real documents, swapping the evaluator changes no verdict and no finding.
- doc-verify/src/engine/index.ts:196-201 maps incomplete coverage to BLOCKED; diagnostics.ts:151 says a budget failure must never be treated as satisfied.

**Boundary.** doc-verify semantics, not evaluators in general. 28b5b22b is the first 27 messages of d0e2d0bd, so this is one session captured twice, not two independent sources. The reference covers the deterministic subset; 24 documents do not exercise prefix demand or the refusals; the property suite is nightly (~1,006 s). Invariants catch evaluator bugs but not a wrong specification.

**Operational check.** Run the invariant/property suite on generated programs and require zero refuting before citing any verdict. Differential-test against a naive reference and a three-valued derivation. Seed every failure path — tiny budget, absent key, truncated/unparseable output, unasked atom — and assert BLOCKED or NEEDS_REVIEW, never PASS.

---

## L12 — Derive the extraction budget chain and record per-call provenance

**Status:** Observed · Transferred · Unexercised in paper-graph

**Rule.** Derive every extraction call's budget from one inequality: context window ≥ prompt + output budget; max_tokens ≥ output budget; timeout ≥ output budget divided by measured tokens per second; batch limit ≥ largest record. Classify empty/truncated output by finish reason and the content/reasoning split before calling it a model error. Log per call: run id, model snapshot, input/output/cached tokens, attempt, finish reason, latency, error class. Missing usage is 'unreported', not 0.

**Why it holds.** A 16,384-token structured budget exceeded the provider's 8,192 max_tokens and 60 s timeout; the reasoning model spent its output in reasoning_content, returning finish_reason length, content_len 0, 2,836 reasoning tokens, and 1,024 completion tokens on 121 prompt tokens. At the measured ~51 tokens/s, a 16K generation alone needs ~321 s, so the timeout could never be met. Server restarts produced 500s that were misread as extraction failures. The paper pipeline's embedding client coerces missing usage to 0 and its SyncReport has no run id, timestamp, attempt, or finish reason.

**Evidence.**

- thread f8ea8d0d#21: 'Failures all request output_budget=16384 — but your zai provider has max_tokens: 8192 and timeout: 60.0.'
- thread c6422ed2#20: 'finish_reason: length, content_len=0, reasoning_len=2836 (but capped at 1024 tokens). The model put everything into reasoning_content and never emitted content.'; #54: at 51 t/s, 16384 tokens = 321 s > 60 s; #69: the 500s were self-inflicted llama-server restarts.
- src/embeddings.rs:109-112 sums usage with unwrap_or(0); src/pipeline.rs:20-29 records only database, model, URL, tokens, elapsed, counts.
- experiments/ontology-extraction/round-001-rubric.md:118-121 specifies the fuller manifest; no code emits it.

**Boundary.** The concrete numbers are deployment-specific; the invariant is the budget inequality. paper-graph has no extraction call and no budget validator, so the budget rule is a pre-flight checklist, not an exercised result. The per-call manifest is rubric text.

**Operational check.** Pre-flight validator: assert the four inequalities and fail the configuration before the job. Log the per-call manifest; store missing usage as unreported. On empty output, read finish_reason and content/reasoning split before retrying. After a restart, re-issue the same request id and compare outcomes.

---

## L13 — Distil on an explicit decision; record hard lessons, not process narrative

**Status:** Decided · Observed · Prescribed

**Rule.** Keep working context separate from durable memory; write durably on an explicit decision. When a topic closes or outgrows its context, replace the detailed span with a summary and a pointer to the archived detail. A durable record carries the decision, why, evidence, and the rejected alternative — not temporary test scaffolding or process narrative.

**Why it holds.** Working context is cheap and reversible; durable storage is curated and hard to retract, so promotion is a deliberate act, not an inference from interestingness. The working layer behaves as an index: summaries and pointers, with detail archived once a topic closes. The design record is a decision/lesson log, not a diary.

**Evidence.**

- thread 5509b0c1#15 (user): 'Not that keep in mem (calling nowledge memory add) I mean you just remember.'
- thread 5509b0c1#31 (user): 'memory works more like an index ... after a topic finish or has grown larger, we can replace that detailed part into summarization records pointing to the archived details.'
- thread 7172fd66#50 (user): 'some are just temporary test like the flip, that's not what we need to record. the design is to record the hard lessons we learnt.'
- experiments/ontology-extraction/decisions.md:3 uses Timestamp | Decision | Why | Evidence | Result; gates.md:3 defines 'complete: no' as not a pass.

**Boundary.** 5509b0c1 is an idea-capture conversation about agent memory, not evidence about the paper system, and it was explicitly disqualified in L01; do not count it as independent corroboration. No outcome is measured. The archive-pointer and rejected-alternative columns are prescriptions: the decision log has no archive pointer and no rejected-alternative column, though row 7 embeds one in Why.

**Operational check.** Require an explicit promotion decision before any durable write. On topic close, write a summary plus pointer and stop carrying the detail. Require decision + why + evidence + rejected alternative + open issues; reject temporary scaffolding and unexecuted results.

---

## L14 — Choose a dependency-light backend and migrate by reviewed one-off scripts

**Status:** Transferred · Contested

**Rule.** Choose the backend with the fewest native moving parts that is already pinned by the runtime, and prove its version and capabilities before adopting it. Execute every schema change as a reviewed one-off script against a copy — additive wherever possible, never a table rebuild that application code runs at startup — and publish only after a count oracle passes.

**Why it holds.** At a few hundred works, exact scan plus an in-process full-text index is sufficient, so the decisive variable is how many native layers can drift, not search quality. A LanceDB attempt died on a numpy/libstdc++ ABI conflict and the workable prototype was SQLite FTS5/BM25. A PostgreSQL extension failed after a base-image ICU change because it was built against ICU 72's versioned symbols. An automatic table-rebuild migration was rejected in favor of a single additive ALTER with a default.

**Evidence.**

- thread 05a7de1d#45: 'hit a dependency issue with numpy/libstdc++.so.6. This is a known issue with PyTorch environments.' The fallback indexed 286 sections with SQLite FTS5/BM25.
- thread d9cbb41d#62 (user): 'this is dangerous, provide the one off migration sql to me and remove that logic'; #86: 'you have drops in schema? how dare you say it's a migration.'
- thread dcca45f9#30: pg_search failed to load after the base image moved to ICU 76 (undefined symbol uloc_canonicalize_72).
- tests/existing_database.rs:9-21 is an #[ignore]d snapshot assertion of seven counts; it never calls rebuild or applies a migration.

**Boundary.** Backend and migration clauses are transferred from other projects and untested on HawDB. paper-graph still rebuilds destructively; HawDB ALTER support is unverified; the count oracle is skipped by cargo test and asserts fixed 2026-11-17 counts, not a migration. No system in the evidence published after a rebuilding migration passed a count oracle.

**Operational check.** Record engine version, compiled features, and toolchain floor before adoption. Apply migrations as a reviewed one-off script on a copy inside a transaction, additive first; forbid startup/rebuild code that drops tables or views. Gate publish on a CI-run count/content oracle, not an ignored snapshot test.

---
