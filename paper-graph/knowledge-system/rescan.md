# Rescan: what survived, what changed

Round 2 produced 14 lesson records with R0-R3 confidence and a confirmed/qualified/rejected verdict. Round 3 attacked them with contradictions, code reconciliation, and a source-independence audit. This file records the regrade.

The short version: **nothing was rejected outright, but six claims were narrowed and four "confirmed" labels were demoted.** The corpus supports the architecture and the failure modes; it does not support the outcome claims.

## Regrade table

| Point | Round 2 | Round 3 finding | Final status |
|---|---|---|---|
| P01 Layer ownership | R2, qualified | "Disposable" licensed the destructive rebuild that P03/P14 forbid; the distilled layer is procedural and the Mem save was blocked | Decided · Observed · Specified |
| P02 Identity | R3, qualified | Second thread never mentions Zotero or itemID; zotero_key is stored but never joined; no re-import test | Decided · Observed · Durability untested |
| P03 Sync/rebuild | R3, confirmed | The schema-evolution half comes from bub_sf and is unproven on HawDB; the code commits a data wipe, not a table rebuild | Decided · Observed · Specified · Transferred |
| P04 Admission | R2, qualified | The corpus test only asserts a status string; admission is a manual record; no parser exists | Observed · Specified |
| P05 Ontology scope | R2, qualified | 9 node types and 17 relations written before any competency question; the closed kind enum can hide the taxonomy | Specified · Contested |
| P06 Relation strength | R2, confirmed | Spec-only: predicates exist in markdown, not code; INTRODUCES smuggles priority | Specified · Contested |
| P07 Evidence claims | R3, qualified | Locator half depends on an unbuilt parser and chunk/page/offset table; two kinds of unknown conflated | Specified · Transferred |
| P08 Evaluation first | R3, qualified | Resolution is not held constant across U2/S2/P2; claim scoring precedes the questions | Decided · Specified · Unexecuted |
| P09 Metric panel | R2, qualified | Research report still calls strict F1 the headline; the absolute "never" is too strong | Specified · Contested |
| P10 Eval integrity | R3, confirmed | Strong on two other projects; the paper corpus test is a tripwire, not gold protection; accounting can be a self-report | Observed · Transferred · Specified |
| P11 Evaluator validation | R3, confirmed | 28b5b22b is the first 27 messages of d0e2d0bd — one session, not two; doc-verify, not paper-graph | Transferred · Observed (one source) |
| P12 Extraction ops | R3, confirmed | Budget rule is unexercised in paper-graph; the code check is the embedding path, not an extraction call | Observed · Transferred · Unexercised |
| P13 Distillation | R3, confirmed | 5509b0c1 is disqualified as system evidence; archive pointer and rejected-alternative column are prescriptions | Decided · Observed · Prescribed |
| P14 Backend/migration | R3, qualified | The only count oracle is #[ignore]d and never runs a migration; backend and migration are from different projects, untested on HawDB | Transferred · Contested |

## The eight corrections that matter

### 1. Disposable metadata vs disposable knowledge

Round 2's P01 said "treat HawDB as disposable derived state". Round 3 pointed out that store.rs:108-109 runs DELETE FROM works and MATCH (n) DETACH DELETE n on every sync, and the only mitigation is a decision to use a separate derived database that does not exist. If the rule is stated without scope, it licenses the exact data loss P03 and P14 warn about.

**Correction.** Only the metadata graph is disposable. Extraction, human judgment, and run provenance are not. The separate derived store is a precondition, not a property.

### 2. A status string is not an admission gate

P04 and P10 cited the same line and disagreed. tests/experiment_corpus.rs:15 asserts corpus["status"] == "pdf_verified_gold_pending". It does not stat a PDF, check %PDF-, or assert the absence of gold. It is a tripwire on a self-declared label.

**Correction.** P04 is right: admission is a one-time manual record. P10's gold-protection claim is wrong for this repo; there are no gold files at all.

### 3. Identity is single-source and the durable key is unused

P02 claimed R3 because it listed a second thread. That thread is an OCaml pattern-matching conversation; searches for Zotero and itemID return zero matches. In the code, item_id is the primary key, zotero_key is a non-key column, and no query joins on the key. No test exercises a merge, restore, or re-import.

**Correction.** The identity rule is a user decision plus an observed schema, not replicated evidence. The durable-address half is untested.

### 4. No IS_A is not the same as no unearned taxonomy

P05's headline said "ship no generic IS_A edge". The DeepSeek critique points out that the closed, single-valued Concept.kind enum relocates subsumption. The design already has 9 node types and 17 relations, written before any competency question was frozen, and the recommended five-relation start was not followed.

**Correction.** The rule is now "start from competency questions; add a type only when a question needs it; no IS_A is not enough". The current vocabulary is over budget for an unrun pilot.

### 5. The ontology-specificity axis is not constant

P08 said one manipulated variable: ontology constraint, with resolution held constant. The plan gives U2 exact normalized-name merge, S2 exact known-alias rules, and P2 curated PL aliases as its only additional input. If those resolvers are equivalent, the plan should say so and use one; if not, the axis is vocabulary plus curated aliases.

**Correction.** Either share one resolver with the alias table as the only difference, or state the axis honestly and add an arm that separates vocabulary from resolution. Do not claim a constant resolution policy.

### 6. The metric panel is the default, not an absolute

P09 said "never a single aggregate F1". The research report still says strict evidenced relation F1 is the headline twice (lines 23 and 643), and P09's own boundary allows F1 as headline when gold is complete, one relation dominates, and abstention cannot be rewarded.

**Correction.** Panel by default; strict F1 as one diagnostic; F1 as headline only under the three named conditions. Amend the report so the primary design document and the rubric agree.

### 7. Distillation evidence is thinner than it looked

P13 cited 5509b0c1 as support, but P01 had already disqualified that thread as an agent-memory analogy with no Zotero or HawDB content. The other thread supports deleting temporary scaffolding, not a summary-plus-archive-pointer mechanism. The decision log has a Why and Evidence column, but no archive pointer and no rejected-alternative column.

**Correction.** The confirmed claim is narrower: a dated decision log with why and evidence, plus a practice of excluding temporary test artifacts. The archive pointer and rejected-alternative field are prescriptions to add.

### 8. The count oracle does not run

P14 leaned on tests/existing_database.rs as a publication gate. That test is #[ignore]d, opens a database, asserts seven hardcoded 2026-11-17 counts, and never calls rebuild or applies a migration. The backend and migration clauses come from three different projects (OpenClaw, bub_sf, paper-graph) and none of them published after a rebuilding migration passed a count oracle.

**Correction.** Mark backend and migration as transferred and untested on HawDB. Add a CI-run copy -> migrate -> count harness before calling it a gate.

## What was rejected or downgraded

- **Rejected:** "The corpus test enforces PDF admission." It asserts a status string only.
- **Rejected:** "28b5b22b and d0e2d0bd are two independent sources." They are one session captured twice.
- **Rejected:** "zotero_key is the durable address." It is stored but never joined on.
- **Downgraded:** P06 from confirmed to specified. The predicates exist only in markdown.
- **Downgraded:** P12's budget rule from confirmed to unexercised in paper-graph. The incidents are real; the pipeline has no extraction call.
- **Downgraded:** P14's count oracle from a gate to a snapshot assertion.
- **Softened:** P09's "never a single F1" to a default with named exceptions.
- **Scoped:** P01's "HawDB is disposable" to the metadata graph only.

## Residual uncertainty

- The 17-relation vocabulary may still be right; it has simply not been tested against questions.
- The separate derived DB may be the right isolation; it does not exist yet.
- The metric panel may beat strict F1; no arm has run.
- The additive migration may work on HawDB; ALTER support is unverified.
- itemID may be stable enough in practice; no re-import test exists.
- The doc-verify validation pattern may transfer; it has not been applied to this pipeline.

These are the next experiments, not conclusions.
