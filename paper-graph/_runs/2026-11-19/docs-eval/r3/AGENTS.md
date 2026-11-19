# AGENTS.md — Paper Graph governed decision rules

Generator: paper-graph-docs-eval-rule-generator-v1.
Sources: paper-graph/knowledge-system/lessons.jsonl and paper-graph/docs/rule-check-matrix.md.
Scope: these are the only encoded rules available in variant B; no full-document fallback is implied.
Apply the narrowest matching lesson. Preserve every stated boundary and unknown; never infer an implementation, measurement, human adjudication, locator, or usage value. Prefer a fail-closed action and name the listed deterministic check when one applies. A human-judgment entry never licenses an agent to make that judgment.

## L01 — Separate canonical, rebuildable, and distilled layers — and make the rebuild boundary explicit

Rule: Zotero is the only canonical home of bibliographic metadata. The HawDB metadata graph is rebuildable derived state. Extraction, human judgment, and run provenance must live in a separate derived store; distilled notes point back to evidence, they do not duplicate it.

Boundary: The third layer is procedural, not implemented. 'Disposable' applies to the metadata graph only; extraction and review state are not disposable. Until the separate DB exists, the rule is an unmet precondition, not a property of the system.

Enforcement: Before sync, count non-metadata labels/rows in the target DB and fail closed if any exist. Record in every SyncReport the model, URL, token count, elapsed time, run id, and a hash of the source snapshot.

Rejected alternative: Keeping extraction, annotations, or review state beside rebuildable metadata.

Matrix rule: Canonical, rebuildable, and distilled layers stay separate
Owner: W3.5 operations.
Canonical owning document(s): paper-graph/docs/OPERATIONS.md.
Check type: gate.
Deterministic check id(s): check_sync_count_guard.
Human judgment explicitly required: no.

## L02 — Use Zotero IDs as local keys, but persist a durable key and explicit evidence coordinates

Rule: Use Zotero numeric itemID as the local join key inside one library, persist zotero_key and DOI as the durable address, and give every evidence span attachment/page/character coordinates. Never use a title or a content hash as paper identity.

Boundary: itemID is stable only while the Zotero library is unchanged. The line-index convention and character-offset EvidenceSpan are two different address spaces; neither is implemented in paper-graph. The OCaml/paper-notes thread does not corroborate this rule and is not counted as independent evidence.

Enforcement: Add a key-to-itemID mapping table or make zotero_key the durable join. Add a re-import test that changes itemIDs and asserts evidence/claim survival. Reject evidence lacking attachment id, page, and verified character offsets.

Rejected alternative: Using titles, content hashes, or unqualified local row IDs as durable paper identity.

Matrix rule: Local item IDs have durable keys and evidence coordinates
Owner: W3.2 data model.
Canonical owning document(s): paper-graph/docs/DATA_MODEL.md.
Check type: fixture.
Deterministic check id(s): check_corpus_identity, check_evidence_locator.
Human judgment explicitly required: no.

## L03 — Make sync manual, and never let a metadata rebuild delete derived knowledge

Rule: Make synchronization a manual, explicit replacement of derived metadata. Never let it delete extraction or human judgment: scope the rebuild to owned metadata labels, or isolate extraction in a separate database, and prove it with a count oracle. No startup table rebuild that drops views or tables.

Boundary: The additive ALTER remedy is proven only on SQLite/bub_sf. HawDB ALTER support is unverified, initialize() only creates the table when absent, INSERT_WORK hardcodes 13 columns, and there is no schema version. A 'table rebuild' and a 'data wipe' are different failures; the current code commits the second.

Enforcement: Populate a derived DB, run a metadata sync, and assert extraction/claim/evidence counts unchanged; fail closed if derived data is present. Keep a schema version and reconcile columns instead of only creating the table. Forbid DROP TABLE/VIEW in startup paths.

Rejected alternative: Automatic synchronization or startup migrations that rebuild tables and drop derived data.

Matrix rule: Manual metadata sync never deletes derived knowledge
Owner: W3.5 operations.
Canonical owning document(s): paper-graph/docs/OPERATIONS.md.
Check type: gate.
Deterministic check id(s): check_migration_oracle, check_sync_count_guard.
Human judgment explicitly required: no.

## L04 — Admit sources by bytes and signature; treat metadata as a retrieval ceiling

Rule: Admit a paper only after resolving a local file and checking its signature. Treat metadata coverage as a ceiling on retrieval claims. A status string in a corpus file is a tripwire, not an executable admission gate.

Boundary: Admission is established only to file and signature level, not text recoverability. The corpus JSON stores no attachment id, path, or hash, so admission cannot be re-executed from the artifact. The 15/15 claim is a one-time manual record.

Enforcement: Executable admission script: resolve attachment, stat file, check %PDF-, record SHA-256; fail on any missing file. Add a parser smoke test on the four development PDFs before annotation. Attach a metadata coverage report to every retrieval result set.

Rejected alternative: Treating an attachment row or corpus status string as proof of a readable PDF.

Matrix rule: Sources enter by verified bytes and signature
Owner: W3.3 pipeline.
Canonical owning document(s): paper-graph/docs/PIPELINE.md.
Check type: fixture.
Deterministic check id(s): check_corpus_admission.
Human judgment explicitly required: no.

## L05 — Start ontology from competency questions, not from a relation catalogue

Rule: Start from pre-registered competency questions. Add a node, relation, or kind only when a question cannot be answered without it. Banning a named IS_A edge is not enough: a closed, single-valued kind enum can relocate the same taxonomy.

Boundary: Design-only: src/ builds only Work, Person, Collection, Tag, Venue, Attachment; no Concept or Claim node exists. The rule is a target for the pilot, not a description of the current schema. A disputed categorization must stay needs_review rather than earn a new class or edge.

Enforcement: Freeze the competency questions before freezing the vocabulary. Cut v0 to the relations the questions need (start with five). Grep emitted schema/manual for IS_A, subClassOf, BROADER_THAN; require an explicit cue span for every GENERALIZES. Make kind faceted, multi-valued, and attributed until tested.

Rejected alternative: Starting with a broad relation catalogue or treating an IS_A ban as sufficient to prevent taxonomy.

Matrix rule: Ontology starts from competency questions, not a catalogue
Owner: W7.1 ontology owner.
Canonical owning document(s): paper-graph/knowledge-system/ontology.md.
Check type: lint + review.
Deterministic check id(s): check_question_lint.
Human judgment explicitly required: no.

## L06 — Grade relation strength and attribution; resolve identity conservatively

Rule: Grade every Work→Concept link by the evidence that licenses it: MENTIONS for a bare naming, USES only for the work's own method, and INTRODUCES only for a work-scoped claim of contribution — not priority. Aliases are reversible links; ambiguous references stay unresolved; a merge needs definition-level justification.

Boundary: Specification only: MENTIONS/USES/INTRODUCES exist in markdown, not code; no extraction arm has run. The INTRODUCES predicate may need renaming or a non-priority qualifier before it is safe. A qualified alias of two partly overlapping notions still needs a human decision.

Enforcement: Define INTRODUCES as 'this work claims the contribution' and add a non-priority qualifier. Test the 'Dunfield and Krishnaswami introduce X' case. Demote any USES/INTRODUCES supported only by a token match. Report pairwise merge precision and each false merge's downstream edge count.

Rejected alternative: Inferring use or invention from mentions, or merging ambiguous concept identities automatically.

Matrix rule: Relation strength, attribution, and identity resolution stay conservative
Owner: human (annotator).
Canonical owning document(s): paper-graph/knowledge-system/ontology.md, paper-graph/docs/REVIEW.md.
Check type: review.
Deterministic check id(s): none; human judgment required.
Human judgment explicitly required: yes.

## L07 — Make evidence atomic and keep unknown visible

Rule: Admit a claim only as one atomic, independently evaluable proposition bound to a verbatim quote and a resolvable locator. Evidence is a leaf; claim–evidence is many-to-many; overlapping spans dedupe to one support. Missing or unresolved input stays unknown and blocks or demotes — it is never silently filled.

Boundary: Conditional on a parser and a chunk/page/offset table that do not exist. Today evidence can only be quote-checked against Zotero metadata or abstracts; locators must be marked unavailable rather than assumed. Split vocabulary: an unknown input demotes the affected claim and marks it unreviewed; an incomplete evaluation blocks the run and produces no score.

Enforcement: Validate every claim's quote as an exact substring of the cited raw page/chunk and its locator to attachment/chunk/page/offset; a fabricated locator is a hard failure. Dedupe overlapping spans. Pre-register abstention for unanswerable questions.

Rejected alternative: Bundling multiple propositions into one claim or silently filling unresolved evidence.

Matrix rule: Claims are atomic, evidenced, and keep unknown visible
Owner: W3.2 data model.
Canonical owning document(s): paper-graph/docs/DATA_MODEL.md.
Check type: fixture.
Deterministic check id(s): check_evidence_locator.
Human judgment explicitly required: no.

## L08 — Evaluate first, pre-register one axis, and ask the questions before building the scorer

Rule: Build the smallest end-to-end artifact over the real corpus, then pre-register one comparison with frozen inputs and a non-regression baseline. Ask and lint the competency questions before building claim scoring.

Boundary: The plan is pre-registered but unexecuted: gates.md marks plan review, PDF parsing, gold, seeds, extraction, scorer and verdict all 'no'. It also inverts the rule by investing in claim scoring before the retrieval baseline questions are asked. The four simple probes have no headroom and cannot justify the extraction layer.

Enforcement: Write and lint the competency questions first; freeze hashes for manual, prompts, schemas, chunker, model snapshot and gates before opening test outputs. Record every post-freeze change as a protocol deviation. Keep a gate ledger where an unrun row is 'no', never a pass.

Rejected alternative: Building a claim scorer before establishing retrieval questions and a frozen baseline.

Matrix rule: Evaluation freezes questions, inputs, and one axis before scoring
Owner: W3.4 evaluation.
Canonical owning document(s): paper-graph/docs/EVALUATION.md.
Check type: lint + gate.
Deterministic check id(s): check_eval_manifest, check_question_lint.
Human judgment explicitly required: no.

## L09 — Score with a pre-registered panel; keep strict F1 as one diagnostic

Rule: Use a pre-registered panel by default: per-relation quality, high-risk violation rate, coverage, evidence validity, downstream QA, cost, and review burden. Keep strict evidenced relation F1 as one diagnostic; allow it as headline only when gold is complete, one relation type dominates, and abstention cannot be rewarded.

Boundary: The panel is an argument, not a measurement: no arm, scorer, or gold exists. The thresholds (.95 precision, .05 margin, US$25, 20 min/paper) are project choices, not community benchmarks. The absolute 'never a single F1' is too strong and is retracted by the rule's own conditions.

Enforcement: Publish numerator, denominator, and stop threshold per metric before any arm runs. Reject any report whose headline is one aggregate F1. Amend the research report so the primary design document and the rubric do not contradict each other. Re-run every 'complete: yes' gate row from a clean checkout.

Rejected alternative: Using one aggregate strict relation F1 as the sole headline result.

Matrix rule: Metric panel and any strict-F1 exception are pre-registered
Owner: human (evaluation owner).
Canonical owning document(s): paper-graph/docs/EVALUATION.md.
Check type: review.
Deterministic check id(s): none; human judgment required.
Human judgment explicitly required: yes.

## L10 — An eval is evidence only if it proves execution, records the run, and leaves gold intact

Rule: Treat an eval as evidence only when the run proves it executed every case, records the configuration and raw per-case output that produced the score, and leaves gold untouched. When a model disagrees with gold, gold wins unless a cited skill or decision sentence changes the case. Lint questions and the answer schema as hard as the expected answers.

Boundary: Two independent eval projects plus a third project's status tripwire; not the paper ontology experiment itself. The gold-wins half has no instance in paper-graph: there are no gold files, and tests/experiment_corpus.rs:15 only asserts a status string — it does not protect gold or verify PDF admission. Only one model family was available, so skill findings stay single-family.

Enforcement: Fail the run when executed != corpus size; any skip exits non-zero; a zero-case run is a harness failure, not a score. Commit one run record per execution with commit, model id, extra body/config, repeat index, per-case expected vs actual, and raw output. Diff expected lines on every case edit; require a cited deciding sentence and a second reviewer. Lint questions for outcome-presupposing wording.

Rejected alternative: Accepting scores from zero-case, unrecorded, or gold-mutating evaluation runs.

Matrix rule: Evals prove execution, record raw runs, and leave gold intact
Owner: W3.4 evaluation; human (annotator) gold.
Canonical owning document(s): none listed.
Check type: gate + lint.
Deterministic check id(s): check_eval_manifest, check_gold_diff, check_question_lint.
Human judgment explicitly required: no.

## L11 — Validate the evaluator before trusting any verdict

Rule: Before trusting a verdict, validate the evaluator itself against invariants and an independent reference that shares no code with it. Map timeouts, budget exhaustion, parse or backend failures, and unasked atoms to BLOCKED or unknown, never to PASS.

Boundary: doc-verify semantics, not evaluators in general. 28b5b22b is the first 27 messages of d0e2d0bd, so this is one session captured twice, not two independent sources. The reference covers the deterministic subset; 24 documents do not exercise prefix demand or the refusals; the property suite is nightly (~1,006 s). Invariants catch evaluator bugs but not a wrong specification.

Enforcement: Run the invariant/property suite on generated programs and require zero refuting before citing any verdict. Differential-test against a naive reference and a three-valued derivation. Seed every failure path — tiny budget, absent key, truncated/unparseable output, unasked atom — and assert BLOCKED or NEEDS_REVIEW, never PASS.

Rejected alternative: Trusting a single evaluator implementation or converting infrastructure failures into PASS.

Matrix rule: Evaluator failures never become PASS; independent semantics are reviewed
Owner: W3.4 evaluation; human (evaluator owner).
Canonical owning document(s): paper-graph/docs/EVALUATION.md.
Check type: gate + review.
Deterministic check id(s): check_eval_manifest.
Human judgment explicitly required: no.

## L12 — Derive the extraction budget chain and record per-call provenance

Rule: Derive every extraction call's budget from one inequality: context window ≥ prompt + output budget; max_tokens ≥ output budget; timeout ≥ output budget divided by measured tokens per second; batch limit ≥ largest record. Classify empty/truncated output by finish reason and the content/reasoning split before calling it a model error. Log per call: run id, model snapshot, input/output/cached tokens, attempt, finish reason, latency, error class. Missing usage is 'unreported', not 0.

Boundary: The concrete numbers are deployment-specific; the invariant is the budget inequality. paper-graph has no extraction call and no budget validator, so the budget rule is a pre-flight checklist, not an exercised result. The per-call manifest is rubric text.

Enforcement: Pre-flight validator: assert the four inequalities and fail the configuration before the job. Log the per-call manifest; store missing usage as unreported. On empty output, read finish_reason and content/reasoning split before retrying. After a restart, re-issue the same request id and compare outcomes.

Rejected alternative: Setting budgets independently and treating missing token usage as zero.

Matrix rule: Budgets derive from measured limits and calls retain provenance
Owner: W3.5 operations.
Canonical owning document(s): paper-graph/docs/OPERATIONS.md.
Check type: preflight.
Deterministic check id(s): check_budget_preflight.
Human judgment explicitly required: no.

## L13 — Distil on an explicit decision; record hard lessons, not process narrative

Rule: Keep working context separate from durable memory; write durably on an explicit decision. When a topic closes or outgrows its context, replace the detailed span with a summary and a pointer to the archived detail. A durable record carries the decision, why, evidence, and the rejected alternative — not temporary test scaffolding or process narrative.

Boundary: 5509b0c1 is an idea-capture conversation about agent memory, not evidence about the paper system, and it was explicitly disqualified in L01; do not count it as independent corroboration. No outcome is measured. The archive-pointer and rejected-alternative columns are prescriptions: the decision log has no archive pointer and no rejected-alternative column, though row 7 embeds one in Why.

Enforcement: Require an explicit promotion decision before any durable write. On topic close, write a summary plus pointer and stop carrying the detail. Require decision + why + evidence + rejected alternative + open issues; reject temporary scaffolding and unexecuted results.

Rejected alternative: Automatically persisting interesting context, temporary tests, or process narrative as durable lessons.

Matrix rule: Durable distillation requires an explicit promotion decision
Owner: human (curator).
Canonical owning document(s): paper-graph/knowledge-system/lessons.md.
Check type: review.
Deterministic check id(s): none; human judgment required.
Human judgment explicitly required: yes.

## L14 — Choose a dependency-light backend and migrate by reviewed one-off scripts

Rule: Choose the backend with the fewest native moving parts that is already pinned by the runtime, and prove its version and capabilities before adopting it. Execute every schema change as a reviewed one-off script against a copy — additive wherever possible, never a table rebuild that application code runs at startup — and publish only after a count oracle passes.

Boundary: Backend and migration clauses are transferred from other projects and untested on HawDB. paper-graph still rebuilds destructively; HawDB ALTER support is unverified; the count oracle is skipped by cargo test and asserts fixed 2026-11-17 counts, not a migration. No system in the evidence published after a rebuilding migration passed a count oracle.

Enforcement: Record engine version, compiled features, and toolchain floor before adoption. Apply migrations as a reviewed one-off script on a copy inside a transaction, additive first; forbid startup/rebuild code that drops tables or views. Gate publish on a CI-run count/content oracle, not an ignored snapshot test.

Rejected alternative: Adding native backend layers without capability probes or running destructive migrations at application startup.

Matrix rule: Backend/schema changes use reviewed additive migrations on copies
Owner: W3.5 operations.
Canonical owning document(s): paper-graph/docs/OPERATIONS.md.
Check type: lint + fixture.
Deterministic check id(s): check_migration_oracle.
Human judgment explicitly required: no.
