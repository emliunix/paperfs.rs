# Constitution — Paper Knowledge System

**Status:** Decided · Observed · Specified (baseline `paper-graph` at `f1e46d9`, 2026-11-19). This document governs design and implementation; no extraction experiment has run, no PDF has been parsed, and no gold annotations exist.

## Precedence and Enforcement

Document authority follows a strict order: **`CONSTITUTION.md` > `method.md` > owning project doc > `README.md` status**.

1. `CONSTITUTION.md` — non-negotiable system invariants that every lower artifact must obey.
2. [method.md](method.md) — evidence vocabulary (`Observed`, `Decided`, `Specified`, `Transferred`, `Contested`), adjudication rules, and reviewer independence.
3. **Owning project doc** (`docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md`, `docs/PIPELINE.md`, `docs/EVALUATION.md`, `docs/OPERATIONS.md`, `docs/EXPERIMENTS.md`, `docs/ROADMAP.md`, `docs/REVIEW.md`, `PRAXIS.md`) — one subject per document.
4. `README.md` — orientation and current status summary only.

Lower documents link upward instead of restating invariants; a fact stated twice is a defect. A rule that can be structurally tested must not remain prose: [docs/owner-map.md](docs/owner-map.md) maps each subject to one owning document and one `check_*` ID or explicit `judgment (human)` gate. Lesson IDs (`L01`–`L14`) refer to [lessons.md](lessons.md); code paths resolve from the `paper-graph` crate root.

---

## The Seven Invariants

### 1. Canonical, rebuildable, and distilled layers stay separate
- **Status:** Decided · Observed · Specified | **Owning doc:** `docs/ARCHITECTURE.md` (distilled lessons in `PRAXIS.md`)
- **Rule:** Zotero is the sole canonical store for bibliographic metadata; the HawDB metadata graph is disposable derived state, extracted claims and review state live in a separate derived store, and distilled notes record only explicitly promoted summaries with pointers to evidence.
- **Why it holds:** `src/store.rs:108-109` runs `DELETE FROM works` and `MATCH (n) DETACH DELETE n` in one transaction on every sync, destroying any extracted or reviewed state stored beside metadata (`L01`), while uncurated writes pollute durable memory with temporary scaffolding (`L13`).
- **Falsifier:** A metadata sync deletes or alters any extracted claim, review state, or run record; bibliographic fields are mutated outside Zotero; or durable memory copies raw text or process narrative without an explicit promotion decision and archive pointer.
- **Check pointer:** `check_metadata_sync_guard` (planned); `judgment (human)` for durable promotion.

### 2. Durable identity and evidence coordinates survive rebuilds
- **Status:** Decided · Observed · Specified | **Owning doc:** `docs/DATA_MODEL.md`
- **Rule:** Numeric Zotero `itemID` is a local join key only; every work persists `zotero_key` and DOI as its durable identity (never title or content hash), and every evidence span persists `attachment_item_id`, `chunk_id`, page, and verified character offsets.
- **Why it holds:** `src/store.rs:11-12` keys `works` on local `item_id` without joining on `zotero_key` (`L02`); Zotero merges or restores can renumber `itemID`s and orphan assertions unless durable keys and exact document coordinates anchor them.
- **Falsifier:** Renumbering `itemID`s during re-import orphans claims or evidence, a paper identity is derived from a title or content hash, or an accepted assertion lacks a resolvable attachment, page, and character span.
- **Check pointer:** `check_corpus_identity` (planned); `check_evidence_locator` (planned).

### 3. Rebuild safety and schema changes require a count oracle
- **Status:** Decided · Observed · Specified | **Owning doc:** `docs/OPERATIONS.md`
- **Rule:** Metadata sync is manual and fails closed when non-metadata rows or labels exist in its target, and schema migrations execute only as reviewed, additive one-off scripts on a database copy gated by an active count oracle.
- **Why it holds:** `src/pipeline.rs` (`sync_library`) performs an unguarded full wipe without schema versioning (`L03`), automatic startup table-rebuilds risk silent object drops (`L14`), and `tests/existing_database.rs:9-21` is `#[ignore]`d rather than enforcing post-migration counts.
- **Falsifier:** Sync runs on a background watcher or overwrites non-metadata labels, startup or migration code executes `DROP TABLE` or `DROP VIEW`, or a schema change lands without a non-ignored pre/post count oracle passing.
- **Check pointer:** `check_sync_count_guard` (planned); `check_migration_oracle` (planned).

### 4. Sources enter only by verified bytes and signature
- **Status:** Observed · Specified | **Owning doc:** `docs/PIPELINE.md`
- **Rule:** A work enters an active corpus only when an executable check resolves its local attachment file, stats its byte length, verifies a literal `%PDF-` header, and records its SHA-256 digest; metadata coverage caps metadata-only retrieval claims.
- **Why it holds:** Four initial corpus candidates (`itemID`s `60`, `47`, `5`, `8`) had Zotero attachment rows with no local PDF, and `tests/experiment_corpus.rs:15` asserts only the status string `"pdf_verified_gold_pending"` (`L04`).
- **Falsifier:** Any admitted corpus item lacks a local file, `%PDF-` signature, or recorded SHA-256; a static status string passes for admission; or retrieval results omit metadata-coverage limits.
- **Check pointer:** `check_corpus_admission` (planned).

### 5. Evidence is atomic and unknown stays visible
- **Status:** Specified | **Owning doc:** `docs/DATA_MODEL.md` (ontology vocabulary in [ontology.md](ontology.md); evaluation use in `docs/EVALUATION.md`)
- **Rule:** Ontology predicates grow only from frozen competency questions with evidence-graded roles (`MENTIONS`, `USES`, `INTRODUCES`), every claim binds one atomic proposition to a verbatim substring quote and resolvable locator, and missing or ambiguous inputs remain `unknown` or `unreviewed`.
- **Why it holds:** Unchecked catalogues smuggle taxonomy through `IS_A` or closed `kind` enums (`L05`), ungraded mentions inflate into contributions or false merges (`L06`), and overlapping chunks or silent defaults fabricate support unless evidence is leaf-atomic and unknowns propagate (`L07`).
- **Falsifier:** Schema types are added without a blocking competency question, a bare mention is scored as `USES`/`INTRODUCES`, a quote fails substring or locator checks, overlapping spans count twice, or missing data silently coerces to a value.
- **Check pointer:** `check_evidence_locator` (planned); `check_question_lint` (planned).

### 6. Evaluation is pre-registered, executed, and recorded
- **Status:** Decided · Observed · Specified | **Owning doc:** `docs/EVALUATION.md` (call budgets in `docs/OPERATIONS.md`, gates in `docs/EXPERIMENTS.md`)
- **Rule:** Evaluations freeze questions, single-axis resolvers, metric panels, and call-budget inequalities before opening test runs, require `executed == corpus_size`, validate scorers so timeouts or unasked atoms yield `BLOCKED`, and record per-call provenance with missing usage marked `unreported`.
- **Why it holds:** Shifting resolvers or single-F1 headlines hide high-risk errors (`L08`, `L09`), unverified harnesses have silently executed zero cases (`L10`) or mapped budget exhaustion to `PASS` (`L11`), and `src/embeddings.rs:109-112` plus `src/pipeline.rs:20-29` coerce missing usage to `0` without run IDs (`L12`).
- **Falsifier:** Test predictions run before input hashes freeze, `executed != corpus_size` or skipped cases exit zero, evaluator failures return `PASS`, call budgets violate context/max-token/timeout inequalities, or unreported usage is stored as `0`.
- **Check pointer:** `check_eval_manifest` (planned); `check_budget_preflight` (planned).

### 7. Human owns gold, verdict, spend, and the claim
- **Status:** Decided · Specified | **Owning doc:** `docs/REVIEW.md` (reviewer independence procedure in [method.md](method.md#reviewer-independence); paper framing in `docs/ROADMAP.md`)
- **Rule:** A human authors all gold annotations, adjudicates final verdicts and same-family reviews, approves hosted spend, and authorizes every paper claim; model output never moves an expected gold answer.
- **Why it holds:** Editing expected answers or writing outcome-presupposing questions tunes the oracle to the model (`L10`), same-family model routes share correlated blind spots ([method.md](method.md)), and pre-verdict artifacts justify only architectural and protocol claims (`L08`, `L13`).
- **Falsifier:** An agent or model authors, modifies, or moves any expected gold answer, regardless of approvals; a same-family review is marked independent; paid extraction runs without approval; or docs claim empirical extraction/QA gains before a held-out verdict.
- **Check pointer:** `check_gold_diff` (planned); a cited human decision and second review remain required checks for human gold changes and never authorize agent/model authorship or movement; `judgment (human)` for gold, verdict, spend, and paper claim.
