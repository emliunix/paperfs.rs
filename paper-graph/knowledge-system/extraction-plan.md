# Extraction plans and rubrics, per point

This is the Round 2 plan, corrected by the Round 3 audit. Each point has a question, source threads, what to extract, the evidence standard, the distillation target, and the correction that changed its final wording.

The common lesson record shape was:

~~~
rule:          1-2 imperative sentences, generalizable beyond the incident
mechanism:     why it holds
evidence:      1-3 verbatim quotes with thread id and message index
code_check:    path, line, observation (optional)
boundary:      when it does not apply, or what would falsify it
operational:   1-3 checks for the paper knowledge system
confidence:    R0-R3 at Round 2; status tags in the final synthesis
verdict:       confirmed | qualified | rejected
~~~

## P01 — Layer ownership and persistence guarantees

- **Question.** What is canonical, what is rebuildable, and what is distilled? Where must each layer never be the sole home of data?
- **Sources.** 71ffc1b7; 5509b0c1 (analogy only; disqualified as system evidence in Round 3).
- **Extract.** Explicit layer assignments; what survives a rebuild; where extraction and human judgment live; whether any code links a distilled note to an itemID.
- **Evidence standard.** A user decision plus a code path that shows what a rebuild deletes. A claim about the distilled layer needs an implemented link, not a proposal.
- **Distil to.** lessons.md L01; engineering.md "Layer boundaries".
- **Round 3 correction.** Scope "disposable" to the metadata graph. The separate derived DB is a decision, not an implementation; the Mem layer is procedural, and the Mem save was blocked.

## P02 — Stable identity and paper addressability

- **Question.** Which IDs survive rebuilds? How is a paper made addressable for evidence?
- **Sources.** 71ffc1b7; e943b0c4 and 3099ed1f for the reading-index convention only.
- **Extract.** The identity column and its use in joins; whether a durable key is stored and exercised; whether chunk/page/offset coordinates exist; whether a re-import or merge test exists.
- **Evidence standard.** A schema or code path for the identity claim. A second thread about reading notes does not corroborate a Zotero identity claim.
- **Distil to.** lessons.md L02; ontology.md "Identity and coordinates"; engineering.md "Identity".
- **Round 3 correction.** Drop the OCaml/paper-notes thread as corroboration. itemID is a local SQLite row id; zotero_key is stored but never used in a lookup; no re-import test exists. The durable-address claim is conditional.

## P03 — Manual sync and non-destructive rebuild

- **Question.** Why manual/one-off sync? What must a rebuild never delete, and how should schema evolve?
- **Sources.** 71ffc1b7; d9cbb41d (SQLite tape-store migration incident).
- **Extract.** The sync trigger and transaction; what the rebuild deletes; whether extraction lives elsewhere; the migration mechanism and whether it drops objects; whether a count oracle runs.
- **Evidence standard.** The actual delete path, not a description. The migration remedy must be tested on the target engine before it is presented as settled.
- **Distil to.** lessons.md L03; engineering.md "Sync and rebuild".
- **Round 3 correction.** The additive ALTER remedy is proven only on SQLite/bub_sf. HawDB ALTER support is unverified, initialize() only creates the table, and there is no schema version. The current code commits a data wipe, which is not the same failure as a table rebuild.

## P04 — Prove the engine and admit sources before architecture

- **Question.** What must be smoke-tested before design commitments? When is an attachment a usable paper? How does metadata quality bound retrieval?
- **Sources.** 71ffc1b7.
- **Extract.** The smoke-test checklist and what it actually covered; the corpus-admission checks; the metadata coverage numbers; a concrete ranking failure caused by metadata.
- **Evidence standard.** A test or a recorded filesystem check. A status string is not a check.
- **Distil to.** lessons.md L04; engineering.md "Corpus admission" and "Metadata ceiling".
- **Round 3 correction.** tests/experiment_corpus.rs:15 only asserts the status string; it never stats a PDF or checks %PDF-. The 15/15 admission is a one-time manual record. The smoke test did not cover snapshot or branch behavior.

## P05 — Small domain ontology without unearned taxonomy

- **Question.** What belongs in the curated ontology vs the extracted assertions? Why no generic IS_A? When is an edge justified?
- **Sources.** 71ffc1b7; the research report and DeepSeek critique.
- **Extract.** The stated design principle; the actual node/relation count; whether competency questions exist; whether a closed kind enum relocates subsumption; what the first minimal vocabulary should be.
- **Evidence standard.** A design document is Specified, not Observed. A counterargument must be recorded, not averaged away.
- **Distil to.** lessons.md L05; ontology.md.
- **Round 3 correction.** "No IS_A" is not enough. The design has 9 node types and 17 relations before any question is frozen; the closed kind enum can hide the same taxonomy. Cut to the questions, or make kind faceted and attributed.

## P06 — Relation strength, attribution, and conservative resolution

- **Question.** How do MENTIONS/USES/INTRODUCES differ? How are aliases and merges handled so related-but-distinct concepts survive?
- **Sources.** 71ffc1b7; rubric; DeepSeek critique.
- **Extract.** The definition and licensing evidence for each predicate; the attribution rule; alias vs merge; the false-merge blast radius; whether the predicate itself smuggles priority.
- **Evidence standard.** A specification plus a seeded negative case. No code exists, so the verdict cannot be "confirmed".
- **Distil to.** lessons.md L06; ontology.md "Relations" and "Negative distinctions".
- **Round 3 correction.** INTRODUCES may be the wrong predicate: it smuggles priority and conflicts with EXTENDS/GENERALIZES. Keep the cue gate but rename or qualify it, and test the attributed-introduction case.

## P07 — Evidence-bearing claims and unknown propagation

- **Question.** What is atomic evidence? What graph shape links claims and evidence? How do unknown/missing data propagate?
- **Sources.** cb2db461; 28b5b22b/d0e2d0bd; the research report.
- **Extract.** The evidence leaf and many-to-many shape; the locator fields; quote validation and dedup; the difference between an unknown input and an incomplete evaluation.
- **Evidence standard.** A schema plus a worked example. The locator half is conditional on a parser and a chunk/page/offset table.
- **Distil to.** lessons.md L07; ontology.md "Evidence model"; methodology.md "Abstention".
- **Round 3 correction.** Make the rule explicitly conditional: until the parser and coordinate table exist, evidence can only be quote-checked against metadata or abstracts. Split "unknown demotes a claim" from "incomplete evaluation blocks the run".

## P08 — Evaluation-first and pre-registration

- **Question.** Why is search/QA evaluation the first product? What is held constant, what is frozen, and what is a non-regression control?
- **Sources.** 71ffc1b7; the plan and gate ledger.
- **Extract.** The first end-to-end artifact; the manipulated variable; the frozen list; whether the questions exist before the scorer; whether the baseline has headroom.
- **Evidence standard.** A user decision plus a frozen protocol. An unrun protocol is Specified, not a result.
- **Distil to.** lessons.md L08; methodology.md.
- **Round 3 correction.** The resolution policy is not held constant across U2/S2/P2: U2 uses exact normalized-name merge, S2 adds known aliases, P2 adds curated PL aliases. Either use one resolver or state the axis as vocabulary plus aliases. The plan also builds claim scoring before the competency questions are asked.

## P09 — Metric panel and high-risk gates

- **Question.** Why not a single F1? Which metrics are co-headline? What gates stop a run?
- **Sources.** 71ffc1b7; rubric; research report; critique.
- **Extract.** The failure modes a single F1 hides; the panel members; the high-risk gates; whether the primary design document agrees with the rubric.
- **Evidence standard.** A pre-registered rubric, not a measured comparison. The thresholds are project choices.
- **Distil to.** lessons.md L09; methodology.md "Metric panel".
- **Round 3 correction.** The report still says strict F1 is the headline twice; the rubric demoted it. Amend the report. Soften the absolute "never": allow F1 as headline under named conditions.

## P10 — Evaluation integrity and gold authority

- **Question.** How can an eval look green while doing nothing? What must a run manifest contain? Who wins when model and gold disagree? How can questions/schema leak the answer?
- **Sources.** 78dd3c13; d64e9502; e1f865e1; fc1cc7be; the paper corpus test.
- **Extract.** The skip path; the executed-case count; the run record fields; the gold-edit rule; the question/schema leak; the stop rule.
- **Evidence standard.** A reproduced incident plus a code or log check. A status string is not a gold guarantee.
- **Distil to.** lessons.md L10; methodology.md "Integrity".
- **Round 3 correction.** The paper corpus test is a tripwire on a self-declared label, not an admission gate or gold protection. "Gold wins" has no instance in this repo. Add the requirement that the execution accounting itself be validated against an independent oracle.

## P11 — Validate the evaluator before trusting verdicts

- **Question.** What three oracles validate an evaluator? What invariants must hold? How must timeouts/parse failures map to verdicts?
- **Sources.** 28b5b22b and d0e2d0bd (one session); the doc-verify code.
- **Extract.** The invariant list; the independent reference; the seeded defects; the BLOCKED mapping; the coverage of the 24-document corpus.
- **Evidence standard.** A code path plus a differential test. Two captures of one session do not make two sources.
- **Distil to.** lessons.md L11; methodology.md "Evaluator validation".
- **Round 3 correction.** 28b5b22b is the first 27 messages of d0e2d0bd. Treat it as one source. The 24 documents do not exercise prefix demand or the refusals; the property suite is nightly.

## P12 — Extraction operations and run provenance

- **Question.** Which budget/timeout mismatches kill extraction? How are empty/truncated outputs and infra restarts separated from model errors? What must be recorded per run?
- **Sources.** f8ea8d0d; c6422ed2; the paper pipeline.
- **Extract.** The budget inequality; the observed failure signatures; the timeout arithmetic; the restart artifact; the manifest fields; what the paper pipeline actually records.
- **Evidence standard.** A log or a measurement. The paper pipeline's code check is the embedding path, not an extraction call.
- **Distil to.** lessons.md L12; engineering.md "Run provenance".
- **Round 3 correction.** Mark the budget rule "unexercised in paper-graph". The incidents are real; the code evidence is the embedding client's lossy usage accounting and the SyncReport's missing run identity. Keep the inequality as a pre-flight checklist.

## P13 — Durable distillation practice

- **Question.** What is working context vs durable memory? When does an episode close? What must a design/lesson record contain and exclude?
- **Sources.** 5509b0c1 (idea only); 7172fd66; the decision log.
- **Extract.** The write trigger; the index/summary behavior; the decision-log columns; what the user rejected as process narrative; whether the archive pointer and rejected-alternative fields actually exist.
- **Evidence standard.** A user statement about practice, not a measured outcome. Do not cite a thread that was disqualified elsewhere.
- **Distil to.** lessons.md L13; method.md (this file).
- **Round 3 correction.** 5509b0c1 is disqualified as paper-system evidence. The durable claim is narrower: a dated decision log with why and evidence, plus a practice of deleting temporary scaffolding. The archive pointer and rejected-alternative column are prescriptions.

## P14 — Backend selection and migration safety

- **Question.** What makes a search backend reproducible at this scale? What native/ABI/concurrency hazards appear? How should migrations be executed?
- **Sources.** 05a7de1d; dcca45f9; c59ca24c; d9cbb41d; the ignored count test.
- **Extract.** The dependency failure; the fallback; the ABI mismatch; the async boundary; the migration mechanism; whether the count oracle actually runs.
- **Evidence standard.** A real incident plus a working fallback. A transferred remedy is Transferred until tested on the target engine.
- **Distil to.** lessons.md L14; engineering.md "Backend selection".
- **Round 3 correction.** The backend and migration clauses come from different projects and are untested on HawDB. tests/existing_database.rs is #[ignore]d, asserts fixed counts, and never runs a migration. It is a snapshot, not a publication gate.
