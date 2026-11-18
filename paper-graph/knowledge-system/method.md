# Method: three rounds from threads to design knowledge

The goal was not to summarize conversations. It was to turn them into rules that a paper knowledge system can be built and evaluated against, with enough provenance that a later reader can tell a measured finding from a proposal.

Three rounds were run. Each round changed both the set of interesting points and the scan method.

## Round 1 — collect and broaden

**Scan method.** Nowledge Mem thread search over eight theme families, then four broader scouts. Each scout returned thread ids, dates, message counts, a relevance rationale, and key message indexes. A separate probe confirmed that workflow subagents can call the Nowledge Mem tools directly.

**Output.** 19 candidate threads; 18 retained. Five evidence auditors then read the core thread and the most relevant adjacent threads and returned 50 candidate lessons with verbatim quotes and message indexes. Six quotes were spot-checked against the original messages; all six resolved.

**What went wrong.** The first pass was generous. It treated assistant proposals as findings, counted a 27-message prefix and its 1,360-message continuation as two sources, and let one thread's caveat disqualify a thread that another lesson still cited.

## Round 2 — make the points falsifiable

**Point refinement.** The 50 candidates were clustered into 14 points, each stated as a question:

1. Layer ownership
2. Identity and paper addressability
3. Manual sync and non-destructive rebuild
4. Engine smoke test and source admission
5. Small ontology without unearned taxonomy
6. Relation strength, attribution, and resolution
7. Evidence-bearing claims and unknown propagation
8. Evaluation-first and pre-registration
9. Metric panel and high-risk gates
10. Evaluation integrity and gold authority
11. Evaluator validation
12. Extraction operations and run provenance
13. Durable distillation
14. Backend selection and migration safety

**Scan method.** For each point: targeted message fetch around the relevant exchange, verbatim quote recovery, a code check where the repository could confirm or contradict the claim, and a fixed evidence rubric:

- **R0 unsupported** — generic advice, no artifact or decision.
- **R1 anecdotal** — one verbatim quote or one incident.
- **R2 verified** — a user decision, or a quote plus code/file/measurement.
- **R3 replicated** — two or more independent threads, or code/tests plus a negative case.

**Output.** 14 lesson records with rule, mechanism, evidence, code check, boundary, operationalization, confidence, and verdict (confirmed / qualified / rejected). Raw records are in round-2-rescan.json.

**What went wrong.** The rubric graded compound rules with one label. A rule could be half-implemented and still read "confirmed". Two lessons could cite the same artifact and split verdicts. R3 counted a second thread that never mentioned the system.

## Round 3 — attack the synthesis

**Scan method.** Three adversarial auditors, each given the 14 rules but not the earlier reasoning:

1. **Contradiction and overclaim audit.** Find conflicting pairs, rules that claim more than their evidence, unstated dependencies, and caveats that invalidate a headline.
2. **Code reconciliation.** Verify every code-facing claim against the current tree at f1e46d9: path, line, and observation. Status must be verified, contradicted, stale, or unverifiable.
3. **Replication and gap audit.** For each lesson, decide whether the evidence is one thread/project, multiple independent threads/projects, or code plus a second source. List the six gaps the corpus cannot close.

**Output.** 17 findings; 35 code claims (32 verified, 1 contradicted, 1 stale, 1 unverifiable); 14 replication judgments; 6 top gaps. Raw records are in round-3-audit.json.

**What changed.** The audit forced a regrade and several rewordings:

- **Disposability was scoped.** The metadata graph may be rebuilt; extraction, human judgment, and run provenance may not. The separate derived DB is a decision, not an implementation.
- **The corpus test was demoted.** tests/experiment_corpus.rs:15 asserts a status string. It is a tripwire, not an admission gate and not gold protection.
- **Two sources became one.** 28b5b22b is the first 27 messages of d0e2d0bd. The doc-verify lesson is strong, but it is one session.
- **Two "confirmed" labels became "specified".** The ontology and relation rules exist only in markdown; no Concept or Claim node exists in src/.
- **The identity rule lost its second source.** The OCaml/paper-notes thread never mentions Zotero or itemID; zotero_key is stored but never joined on.
- **The evaluation axis was corrected.** U2/S2/P2 do not hold resolution constant: U2 uses exact normalized-name merge, S2 adds known aliases, P2 adds curated PL aliases. The axis is vocabulary plus curated aliases, or the arms need one resolver.
- **The panel rule was softened.** The research report still calls strict F1 the headline; the rubric demotes it. The absolute "never a single F1" is too strong; the panel is the default, with named exceptions.
- **The count oracle was exposed.** The only real-database test is #[ignore]d, asserts fixed counts, and never runs a migration. It is a snapshot assertion, not a migration gate.
- **The budget rule was qualified.** The budget incidents are real, but the paper pipeline has no extraction call and no budget validator; its code evidence is the embedding path.
- **Two kinds of unknown were split.** An unknown input demotes a claim and marks it unreviewed; an incomplete evaluation blocks the run and produces no score.

## The final evidence vocabulary

The R0-R3 rubric was useful for the rescan but misleading in the synthesis, because it implied a single confidence axis. The final lessons use five status tags instead:

- **Observed** — seen in the current code or a measured incident.
- **Decided** — an explicit user decision, not necessarily implemented.
- **Specified** — written in a design document or rubric, not executed.
- **Transferred** — a lesson from another project, not yet tested here.
- **Contested** — a live counterargument or unresolved contradiction.

A rule may carry several tags. The tags say what would have to change to strengthen the claim; they are not a score.

## Adjudication rules

1. A quote must be verbatim and tied to a message index. A paraphrase is not evidence.
2. A user statement outranks an assistant proposal. An assistant proposal is not a decision.
3. Code or a later message that contradicts a thread claim wins. Report the contradiction; do not smooth it.
4. The same conversation captured twice counts once. Check the first user message and timestamps.
5. A compound rule must be split before grading. "Manual sync and additive migration" is two claims.
6. A rule that depends on an unbuilt component is conditional, not confirmed. Say what must exist first.
7. Every path must resolve from the paper-graph repository root. A locator that only works from another directory is not a locator.
8. A gate row that cites a command which no longer exists is stale, not green. Re-run it or mark it "no".

## Precedence and ownership

When two documents could answer the same question, the order is:

1. `CONSTITUTION.md` — the invariants; it wins over everything below.
2. `method.md` — the procedure and evidence vocabulary.
3. The owning project doc (`docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md`, `docs/EVALUATION.md`, ...).
4. `README.md` — orientation and status only.

A fact stated twice is a defect: the lower document links up instead of restating. The [owner map](docs/owner-map.md) names the single owner for each subject and the check that enforces it; a subject with two owners is a defect, and a check with no owner is a wish. Paths resolve from the `paper-graph` crate root, and run artefacts live under `paper-graph/_runs/<date>/` — including the docs hash manifest that freezes an eval input.

## Reviewer independence

A review counts as independent only when the reviewer is a different model family from the writer: Gemini cannot review Gemini, Gemma counts as Google, and zcode/GLM-5.3, devin/glm-5-3 and auto/glm-5.3 are one model behind three gateways. A same-family review is allowed but must be labelled *same-family* in the artifact, and a human adjudicates it. No document may describe a same-family review as independent. The authorizing routes and the evidence for them live in `models.md`.

Case note: this file is `method.md` (lowercase) on a case-insensitive filesystem. A write to `METHOD.md` edits this file silently on APFS and creates a second file on Linux; the docs lint rejects case-only twins.

## Reproducing the corpus

The thread ids and message indexes are in corpus.md and in the two JSON files. The code baseline is paper-graph at f1e46d9. The retrieval queries are listed in corpus.md. A rerun should expect the same 18 threads, but message counts and summaries can change if Nowledge Mem re-syncs a source.

## Governance

### Status vocabulary

A status applies to one atomic statement in the current context; statuses may coexist and are not a confidence score.

- **Observed** — Directly seen in current code, an artifact, or a recorded incident; strengthen it with a reproducible observation, a resolvable run or source locator, and a relevant negative case.
- **Decided** — Explicitly chosen by the human, without implying implementation; strengthen it with a cited decision followed by an executed check of the implementation.
- **Specified** — Written as intended behavior but not executed; strengthen it with an executable check or run whose inputs and result are recorded.
- **Transferred** — Supported by experience outside this project but not established here; strengthen it with a local reproduction, incident, or test.
- **Contested** — Facing a live counterargument or unresolved contradiction; strengthen the underlying statement with evidence that addresses both sides and, where a decision or verdict is involved, recorded human adjudication.
- **Untested** — Not yet exercised by an executed test in this context; strengthen it with a recorded test run, its input and code versions, and its unedited result.

### Review rounds and stop rules

A review round starts from frozen inputs and a named scope, records the reviewer family, and produces findings tied to an atomic statement, evidence, and an owning document. The author must resolve or explicitly dispose of every finding; another round is warranted only when those dispositions change the artifact or new evidence arrives.

Stop iterating when all findings are resolved or have an explicit human-owned disposition, no contradiction remains hidden, required checks and gates are either passing or recorded as blocking, and further edits would be polish rather than new evidence. An unexecuted required check or a failed gate blocks promotion or completion; it does not prohibit drafting or recording the blocker.

A contradiction marks the affected statement **Contested** and blocks that statement's promotion, every dependent gate or merge, and any paper claim that relies on it; unrelated statements may proceed. Resolve it against current artifacts and verbatim sources, with human adjudication for decisions and verdicts, rather than averaging incompatible accounts.

A same-family review is allowed for preliminary or mechanical feedback, or when no different-family reviewer is available, but it must be labelled *same-family*, must not count as independent, and requires human adjudication. Additional same-family rounds cannot substitute for the missing independent review.

### One-statement-per-fact enforcement

Each atomic rule, status, and decision has exactly one owning document; every other document links to that owner instead of restating it. The owner map binds each subject to one owner and either an executable check or an explicit human-judgment gate.

The docs lint is specified to reject a subject with zero or multiple owners, an unresolved owner or check reference, a broken relative link, a repeated normative rule outside its owner, and case-only filename twins. It also rejects a check with no owning subject; editorial summaries may describe status, but may not create a second normative copy of a rule.

