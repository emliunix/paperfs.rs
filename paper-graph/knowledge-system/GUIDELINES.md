# Operating Guidelines — Paper Knowledge System

**Status:** Decided · Specified | **Governs:** day-to-day agent and human execution across `paper-graph` and `knowledge-system`.

System invariants live in [CONSTITUTION.md](CONSTITUTION.md); evidence tags, adjudication rules, and reviewer independence live in [method.md](method.md); subject ownership lives in [docs/owner-map.md](docs/owner-map.md) and active gate status lives in `STATUS.md`; empirical failure rules (`L01`–`L14`) live in [lessons.md](lessons.md). Lower procedures link to those documents rather than restating their rules. Every procedure below binds an input trigger to ordered steps, a closing check or command, a clean rollback path, and an explicit human **STOP** condition.

---

## GP-01 — Editing a Governing Document

- **When to use:** Creating or modifying `CONSTITUTION.md`, [method.md](method.md), `STATUS.md`, `GLOSSARY.md`, `GUIDELINES.md`, `ADR/*.md`, or any single-subject owning project document listed in [CONSTITUTION.md](CONSTITUTION.md) (`docs/*.md`, `PRAXIS.md`).
- **Steps:**
  1. Verify single-subject ownership against [CONSTITUTION.md](CONSTITUTION.md) Precedence and Enforcement and [docs/owner-map.md](docs/owner-map.md); confirm no parallel writer holds the target file.
  2. Respect filesystem case-folding on APFS (for example, edit existing `method.md` rather than creating `METHOD.md`) and link upward instead of restating higher-precedence invariants or definitions.
  3. Tag every architectural or empirical claim with the evidence vocabulary defined in [method.md](method.md) and attach a `check_*` ID or `judgment (human)` pointer for every enforceable rule.
  4. Dispatch an independent, cross-family reviewer agent (never the drafting agent, per [method.md](method.md) and [CONSTITUTION.md](CONSTITUTION.md) Invariant 7) to audit single-statement facts, upward links, and verified code paths.
  5. Recompute the documentation hash and synchronize `STATUS.md` if any subject owner, status tag, or check ID changed; on defect, revert the target file from the clean worktree base before re-drafting.
- **Closing check:** `spine-audit` review pass (`judgment (human)`) + `cargo test --manifest-path paper-graph/Cargo.toml`.
- **STOP:** The proposed edit weakens or removes an invariant in [CONSTITUTION.md](CONSTITUTION.md), alters a human-owned gate, reassigns subject ownership across two documents, or only a same-family reviewer is available for sign-off (`judgment (human)`).

---

## GP-02 — Running an Evaluation

- **When to use:** Executing any development pilot, docs-effectiveness benchmark, parser gate, or held-out extraction/QA run governed by [CONSTITUTION.md](CONSTITUTION.md) Invariants 6–7 and `L08`–`L12`.
- **Steps:**
  1. Freeze and hash the evaluation manifest (competency questions, single-axis resolver config, metric panel numerators/denominators/thresholds, prompt/schema/chunker snapshots, and model route) before opening test inputs; verify no edits occur mid-run.
  2. Run the pre-flight budget validator (`L12`) to assert context, max-token, and timeout inequalities, and run the evaluator property/seeded-failure suite (`L11`) so evaluator failures—including timeouts, truncated outputs, and unasked atoms—map to `BLOCKED`. Reserve `NEEDS_REVIEW` only for a complete result with every requested atom executed whose interpretation requires human adjudication; no named evaluator failure qualifies.
  3. Diff gold expectation files against the frozen baseline (`L10`) and lint questions for outcome-presupposing wording before invoking any model or scorer.
  4. Execute the harness on a clean checkout, enforcing `executed == corpus_size` and non-zero exit on any skipped or zero-case run (`L10`), while logging per-call provenance with missing token usage recorded literally as `unreported` (`L12`).
  5. Commit the immutable run record (commit SHA, manifest hash, repeat index, per-case expected vs actual, raw outputs, and full metric panel rather than a single aggregate F1 per `L09`); if any harness invariant fails, mark the run `INVALID` in the ledger rather than deleting raw logs.
- **Closing check:** `check_eval_manifest`, `check_budget_preflight`, `check_gold_diff`, and `cargo test --manifest-path paper-graph/Cargo.toml`.
- **STOP:** A run requires paid/hosted API spend, touches the held-out split, proposes modifying an expected gold answer or metric threshold after freeze, or yields `BLOCKED` / `NEEDS_REVIEW` on evaluator self-checks (`judgment (human)`).

---

## GP-03 — Merging a Worktree

- **When to use:** Integrating completed work from an isolated worktree (such as `paperfs.rs-wt/w*`) back into the integration branch after parallel execution.
- **Steps:**
  1. Confirm in `git status` and the task assignment table that all modified paths inside the worktree belong exclusively to that worktree's assigned scope (one writer per file) and contain no untracked scratch artifacts.
  2. Verify every completed deliverable carries concrete execution evidence (command, path, or test log) and passes an independent review check per [method.md](method.md).
  3. Run the crate test and active check suite against the worktree state, confirming no `#[ignore]`d migration or admission checks mask regressions (`L03`, `L04`, `L14`).
  4. Rebase or fast-forward merge onto the target integration commit without force-overwriting concurrent commits; if conflict touches shared manifests (`Cargo.toml`, `STATUS.md`), abort the merge (`git merge --abort`) and reconcile explicitly.
  5. Re-run the full workspace verification suite on the integrated tree before removing or archiving the worktree.
- **Closing check:** `git diff --check` + `cargo test --manifest-path paper-graph/Cargo.toml` (exit code `0` on integrated HEAD).
- **STOP:** Two worktrees modified the same file with semantic overlap, a pre/post integration check regresses, or merging would alter frozen evaluation artifacts during an active eval window (`judgment (human)`).

---

## GP-04 — Resolving a Source Contradiction

- **When to use:** Two primary papers, a Zotero bibliographic record and PDF full text, or a code path and project document disagree on a claim, definition, or empirical fact ([method.md](method.md), [CONSTITUTION.md](CONSTITUTION.md) Invariants 1, 2, and 5).
- **Steps:**
  1. Bind each side of the disagreement to its exact durable source coordinates: `zotero_key` / DOI, `attachment_item_id`, `chunk_id`, page, and verified verbatim substring character offsets (`L02`, `L07`), or exact `file:line` for repository vs doc mismatches.
  2. Test whether the mismatch is an artifact of scope, version, or relation grading (`MENTIONS` vs `USES` vs non-priority `INTRODUCES` per `L06`) rather than a genuine proposition conflict; never merge entities or overwrite Zotero fields outside Zotero (`L01`).
  3. For repository-vs-doc contradictions, inspect live code/tests first and classify the prose defect under [CONSTITUTION.md](CONSTITUTION.md) precedence; for literature contradictions, preserve both atomic claims separately and mark the contested proposition `Contested` ([method.md](method.md)) or `unreviewed` rather than averaging or silently dropping one side (`L07`).
  4. Record both evidence spans and the explicit relation or status tag without mutating canonical source bytes; to roll back an erroneous resolution, restore the prior `unreviewed` / `Contested` state from the claim log.
- **Closing check:** `check_evidence_locator` + `check_corpus_identity` (verifying both cited spans resolve cleanly and remain distinct).
- **STOP:** Resolving the contradiction requires choosing one authoritative truth for gold annotations, merging two candidate `Concept` identities with non-zero downstream edge impact (`L06`), or mutating canonical Zotero metadata (`judgment (human)`).

---

## GP-05 — Adding an Ontology Element

- **When to use:** Proposing a new node kind, facet, or relation predicate beyond the frozen `v0` vocabulary governed by [CONSTITUTION.md](CONSTITUTION.md) Invariant 5, `ontology.md`, and `L05`–`L06`.
- **Steps:**
  1. Cite the exact frozen competency question that cannot be answered by the existing `v0` vocabulary; reject catalogue-driven additions outright (`L05`).
  2. Screen the proposed element against structural anti-patterns: forbid `IS_A`, `subClassOf`, and `BROADER_THAN`; require an explicit textual cue span for `GENERALIZES`; keep `kind` faceted, multi-valued, and attributed; and enforce graded attribution (`MENTIONS`, `USES`, non-priority `INTRODUCES`) (`L05`, `L06`).
  3. Draft positive, negative, and boundary decision rules plus verbatim counter-examples in `ontology.md` / `docs/DATA_MODEL.md`, defining when an ambiguous span stays `unknown` (`L07`).
  4. Lint competency questions and schema definitions, then pilot-annotate development-split spans only (never held-out) to verify substring/locator binding and inter-annotator clarity; drop the candidate element on rollback if pilot precision or cue-span rules fail.
- **Closing check:** `check_question_lint` + script step (grep for forbidden hierarchy predicates) + `check_evidence_locator`.
- **STOP:** No frozen competency question is blocked without the element, the addition occurs after an evaluation manifest freeze (requiring a logged protocol deviation per `L08`), or the change alters gold annotation guidelines (`judgment (human)`).

---

## GP-06 — Adding a Dependency or Changing Storage Schema

- **When to use:** Adding or upgrading a crate/engine in `paper-graph/Cargo.toml` or modifying HawDB / SQLite / derived-store tables, columns, or views ([CONSTITUTION.md](CONSTITUTION.md) Invariants 1 and 3; `L01`, `L03`, `L14`).
- **Steps:**
  1. Justify why existing stdlib/workspace crates cannot satisfy the requirement, and record the candidate engine's exact version, minimal compiled feature flags, license, and Rust toolchain floor in an ADR or `docs/OPERATIONS.md` (`L14`).
  2. Verify the dependency or schema change preserves the strict boundary between Zotero canonical state, the disposable HawDB metadata target, and the separate derived claims/review store (`L01`), introducing zero background file watchers or automatic startup `DROP TABLE` / `DROP VIEW` statements (`L03`, `L14`).
  3. For any schema change, write an additive, reviewed one-off migration script that executes inside a transaction against a database copy and bumps the explicit schema version (`L03`, `L14`).
  4. Run a non-ignored pre/post count and content oracle proving zero loss of works, claims, evidence coordinates, and review states (`L03`, `L14`); retain the untouched pre-migration DB copy as the immediate rollback target.
- **Closing check:** `check_migration_oracle`, `check_sync_count_guard`, and `cargo test --manifest-path paper-graph/Cargo.toml --locked`.
- **STOP:** The dependency pulls native C/C++ build toolchains, network services, or heavy transitive trees, raises the workspace MSRV, requires a destructive column/table drop, or touches a live derived database without a verified backup and passing count oracle (`judgment (human)`).

---

## GP-07 — Admitting a Source to the Corpus

- **When to use:** Promoting one or more Zotero items into an active development or held-out evaluation corpus ([CONSTITUTION.md](CONSTITUTION.md) Invariants 2 and 4; `L02`, `L04`).
- **Steps:**
  1. Resolve the candidate work's durable keys (`zotero_key` and DOI, keeping numeric `itemID` strictly local per `L02`) and locate its local PDF attachment path from Zotero metadata.
  2. Execute byte-level verification on the local file: `stat` non-zero byte length, verify the literal `%PDF-` magic header, and record the SHA-256 digest — never trust static status strings such as `"pdf_verified_gold_pending"` (`L04`).
  3. Run the PDF parser smoke check to confirm non-empty page extraction and stable character-offset indexing before scheduling any annotation (`L04`, `L07`).
  4. Record any missing attachment, corrupt header, or empty parse explicitly in the admission manifest (no silent cleanup) and attach a metadata-coverage ceiling report to the corpus snapshot (`L04`); roll back failed candidates to `rejected_missing_pdf` or `parse_blocked` status.
- **Closing check:** `check_corpus_admission` + `check_corpus_identity`.
- **STOP:** Assigning works between development and held-out splits, substituting an alternate PDF version whose SHA-256 differs from a previously frozen manifest, or admitting scanned PDFs requiring OCR changes (`judgment (human)`).

---

## GP-08 — Writing a Lesson or Promoting Distilled Memory

- **When to use:** Appending or revising an operational rule in [lessons.md](lessons.md) / `PRAXIS.md` or promoting a durable summary to distilled memory ([CONSTITUTION.md](CONSTITUTION.md) Invariant 1; `L13`).
- **Steps:**
  1. Confirm an explicit promotion trigger exists (a closed topic, verified failure mode, or adjudicated design decision) rather than transient session scaffolding, process narrative, or unexecuted plans (`L13`).
  2. Verify the target does not duplicate an existing invariant in [CONSTITUTION.md](CONSTITUTION.md) or rule `L01`–`L14`; if refining an existing lesson, supersede or update in place with provenance rather than forking a second rule.
  3. Structure the entry with all required fields (`L13`): concrete decision/rule, causal why, verified repository or run evidence (`file:line`, commit, or run ID), rejected alternative, open issues, and an executable **Operational check** mapped to a `check_*` ID in [docs/owner-map.md](docs/owner-map.md).
  4. Replace raw transcript detail with a concise summary plus archive pointer (`L13`); if subsequent verification falsifies the premise, revert or mark superseded rather than leaving stale guidance active.
- **Closing check:** `check_praxis_schema` (verifying required lesson fields, evidence pointers, and [docs/owner-map.md](docs/owner-map.md) check mapping).
- **STOP:** Promoting durable memory into cross-session knowledge stores without an explicit human promotion decision ([CONSTITUTION.md](CONSTITUTION.md) Invariant 1), or generalizing a `Transferred` external finding as `Observed` on this repository (`judgment (human)`).

---

## GP-09 — Launching a Subagent Fan-Out

- **When to use:** Delegating parallel drafting, code audits, check encoding, or cross-review across multiple subagents or a workflow script ([method.md](method.md)).
- **Steps:**
  1. Partition targets so every writer agent owns strictly disjoint file paths or isolated git worktrees (one writer per file) and cap concurrency within guardrail budgets (at most 4 concurrent writers; at most 15 workflow agents per phase).
  2. Supply each child with a self-contained prompt naming the exact read budget, target output path, governing spine links ([CONSTITUTION.md](CONSTITUTION.md), [method.md](method.md)), forbidden side effects (no opportunistic edits, no `git` mutations unless assigned), and required return evidence.
  3. Assign reviewers per [method.md](method.md#reviewer-independence): independence requires both no self-review and a different model family. Label same-family reviews *same-family*, route them to human adjudication, and never count them as independent.
  4. Monitor completions without busy-polling; halt the phase immediately upon two consecutive child failures to triage root causes rather than retrying blindly.
  5. Collect all child results, inspect `git diff` across the integrated workspace, and run the closing checks directly in the integrating agent before reporting completion; discard out-of-scope child edits via targeted checkout.
- **Closing check:** script step (verify writer disjointness via `git status` / diff scope) + independent reviewer sign-off + `cargo test --manifest-path paper-graph/Cargo.toml`.
- **STOP:** Two consecutive subagent failures occur in a phase, a child requests wider sandbox/network/spend permissions than granted, fan-out exceeds concurrency caps, or a task requires mutating shared canonical/gold files concurrently (`judgment (human)`).
