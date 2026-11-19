# Praxis Schema — Episode, Lesson, and Check Namespace

**Status:** Specified · Observed (baseline `paper-graph` at `f1e46d9`, 2026-11-19). Normalized records in [`lessons.jsonl`](lessons.jsonl) are `Observed` against `round-2-rescan.json` and on-disk repository files; raw conversation thread transcripts are not stored on disk, so thread quotes are audited against the mining snapshot in [`_runs/2026-11-19/praxis-validation.md`](../_runs/2026-11-19/praxis-validation.md). No check binary executes yet (`check_praxis_schema` is planned).

## 1. Namespace Boundary and Ownership

This document owns two rows in [`docs/owner-map.md`](docs/owner-map.md):
- **Workbook / praxis schema** — `Episode`, `Lesson`, and `Check` entities and their relations, enforced by `check_praxis_schema`.
- **Durable distillation** — procedural mechanics for promoting working context into durable praxis records (`judgment (human)`).

Layer separation, precedence, status tags, and reviewer independence are governed upward by [`CONSTITUTION.md`](CONSTITUTION.md) (Invariants 1 and 7) and [`method.md`](method.md) (§Governance) and are linked rather than restated here.

**Separate namespace (`praxis:` vs `paper:`).** The praxis graph records *how the engineering and evaluation system is built, broken, and guarded*; the paper graph ([`ontology.md`](ontology.md)) records *what scholarly works assert about PL and type theory*.
1. `Episode`, `Lesson`, and `Check` live exclusively in the `praxis` namespace (`knowledge-system/lessons.jsonl` and check manifests). They never share tables, node labels, or the rebuild target with bibliographic metadata (`src/store.rs:108-109`) or extracted paper claims.
2. Cross-namespace links are unidirectional governance edges (`APPLIES_TO`, `ENFORCED_BY`): a `Lesson` or `Check` may target a paper-graph stage, schema subject, or code path, while no `Work`, `Concept`, or `Claim` in the paper graph depends on a `Lesson` for scholarly truth.
3. Optional graph projection (planned in W9 `T9.8`) projects reviewed praxis records into a read-only `praxis` label space without mutating canonical Zotero data or paper-claim tables.

---

## 2. Core Entities and Typed Fields

Every praxis entity uses stable, never-reused identifiers and closed status vocabularies from [`method.md`](method.md#status-vocabulary) (`Observed`, `Decided`, `Specified`, `Transferred`, `Contested`, `Untested`). Missing required fields or empty status arrays are schema defects.

### 2.1 `Episode` — provenance unit

An `Episode` is one bounded historical occurrence (a recorded conversation session, an audited commit/incident, or a dated experiment run) from which lessons are distilled; see [`GLOSSARY.md`](GLOSSARY.md). Currently catalogued episodes are the 18 thread records (17 independent sessions; `28b5b22b` is the 27-message prefix of `d0e2d0bd`) in [`threads.json`](threads.json) plus repository/experiment snapshots.

| Field | Type | Constraints |
|---|---|---|
| `id` | `string` | `ep:<8-hex-thread-prefix>` (e.g., `ep:71ffc1b7`), `ep:repo:<commit>`, or `ep:run:<date>:<slug>`. Stable; never reused. |
| `kind` | `"thread" \| "incident" \| "audit" \| "experiment_run"` | Closed discriminator of the occurrence source. |
| `source_ref` | `string` | Full UUID in `threads.json`, commit SHA, or `paper-graph`-relative artifact path. |
| `date` | `string` | ISO-8601 date (`YYYY-MM-DD`) recorded in the source manifest. |
| `independent_of` | `string \| null` | `null` when independent; canonical `Episode.id` when a session was captured twice (e.g., `ep:28b5b22b` aliases prefix of `ep:d0e2d0bd`). |
| `role` | `string` | Role tag from `threads.json` (e.g., `core-lineage`, `migration-safety`, `eval-integrity`). |
| `summary` | `string` | Concise summary replacing raw transcript detail, paired with `source_ref` as archive pointer. |
| `raw_on_disk` | `boolean` | `true` only if the raw transcript/artifact bytes resolve inside the checked-out repository; `false` for external thread captures. |

### 2.2 `Lesson` — normalized rule (`lessons.jsonl`)

A `Lesson` is one reusable operational or architectural rule distilled from one or more `Episode`s and reconciled against repository evidence ([`_templates/lesson.md`](_templates/lesson.md)). [`lessons.jsonl`](lessons.jsonl) stores one JSON object per line with the exact 13-field contract below:

| Field | JSON Type | Constraints |
|---|---|---|
| `id` | `string` | `^L[0-9]{2}$` for the 14 baseline lessons (`L01`–`L14`) or `^P[0-9]{2,}$` for subsequent praxis lessons. Unique, immutable. |
| `title` | `string` | Single-line imperative or declarative headline; no `"we should"` hedging. |
| `rule` | `string` | Actionable normative rule statement; no silent defaults. |
| `mechanism` | `string` | Causal explanation naming concrete code paths, symbols, measurements, or failure mechanics. |
| `scope` | `string` | Exact boundary of components, stages, or artifacts governed by `rule`. |
| `boundary` | `string` | What remains unproven, conditional, unexecuted, or out of scope, plus residual failure modes; carries `QUALIFIES`/ `CONTRADICTS` context where applicable. |
| `status` | `string[]` | Non-empty array (`len >= 1`) drawn from `["Observed", "Decided", "Specified", "Transferred", "Contested", "Untested"]`. |
| `derived_from` | `string[]` | Non-empty array (`len >= 1`) of `Episode` identifiers (e.g., `"ep:71ffc1b7"`) encoding `DERIVED_FROM`. Duplicate session captures must be disclosed in `boundary`. |
| `evidence` | `EvidenceItem[]` | Non-empty array (`len >= 1`) encoding `EVIDENCED_BY`; **every lesson must include at least one `kind: "thread_quote"` item** with a verbatim quote. |
| `applies_to` | `AppliesTo` | Structured target object encoding `APPLIES_TO` (owner-map subjects, lifecycle stages, and `paper-graph`-relative paths). |
| `enforcement` | `Enforcement` | Structured operational check pointer encoding `ENFORCED_BY` (`check_id` or `judgment (role)`, check type, operational specification, and implementation state). |
| `rejected_alternative` | `string` | Concrete option considered and rejected, with the failure reason. Never empty. |
| `supersedes` | `string[]` | Array of prior lesson or Round-2 draft IDs (e.g., `["R2:P03"]`) encoding `SUPERSEDES`; `[]` when none. |

#### Nested types inside `Lesson`

- **`EvidenceItem`**:
  - `kind`: `"thread_quote" | "code_fact" | "doc_fact"`
  - `locator`: `"thread <8-hex>#<msg_index>"` for `thread_quote`; `"<repo-or-crate-path>:<line[-end]>"` for `code_fact` and `doc_fact`.
  - `role`: `"user" | "assistant" | null` (required non-null for `thread_quote`; `null` for file facts).
  - `quote`: `string | null` — exact verbatim substring for `thread_quote` (or exact file excerpt when quoted); `null` only when `fact` records a structural observation.
  - `fact`: `string | null` — concise verified observation for `code_fact`/`doc_fact`, or context note.
  - `verification`: `"verified_on_disk" | "verified_in_r2_snapshot_raw_thread_off_disk" | "unverifiable_raw_thread_off_disk" | "verified_external_audit_off_worktree"` — explicit provenance state recorded by `T2.3`.
- **`AppliesTo`**:
  - `subjects`: `string[]` (`len >= 1`) — exact subject names from [`docs/owner-map.md`](docs/owner-map.md).
  - `stages`: `string[]` (`len >= 1`) — subset of `["admit", "parse", "chunk", "extract", "validate", "review", "project", "retrieve", "distil", "sync", "migrate", "evaluate"]`.
  - `paths`: `string[]` — `paper-graph`-relative code, test, or doc paths touched by the rule.
- **`Enforcement`**:
  - `check_id`: `string` — registered `check_*` identifier from [`docs/owner-map.md`](docs/owner-map.md) (or `"judgment (human)"` where human-gated).
  - `secondary_checks`: `string[]` — additional `check_*` or `judgment (human)` gates participating in enforcement.
  - `kind`: `"lint" | "fixture" | "preflight" | "gate" | "review"` (matching [`_templates/check.md`](_templates/check.md)).
  - `status`: `"planned" | "implemented" | "human_gate"`.
  - `spec`: `string` — concrete pass/fail predicate and seeded-defect target; `"review carefully"` is invalid.

### 2.3 `Check` — structural enforcement unit

A `Check` is an executable test, lint, preflight validator, or explicit human rubric gate registered in [`docs/owner-map.md`](docs/owner-map.md) and shaped by [`_templates/check.md`](_templates/check.md) (schema note in [`ADR 0004`](ADR/0004-atomic-claim-evidence-with-quote-and-locator.md)).

| Field | Type | Constraints |
|---|---|---|
| `check_id` | `string` | `^check_[a-z0-9_]+$` or `^judgment \([a-z_]+\)$`. Registered in `docs/owner-map.md`. |
| `lessons` | `string[]` | Non-empty list of `Lesson.id` values enforced by this check. |
| `type` | `"lint" \| "fixture" \| "preflight" \| "gate" \| "review"` | Execution class (`review` requires a written rubric and human role). |
| `owning_subject` | `string` | Exact Subject row in `docs/owner-map.md`. |
| `path` | `string` | `paper-graph`-relative implementation or rubric path (`planned:<target>` until W4 lands). |
| `command` | `string` | Non-interactive CLI command (e.g., `cargo test -p paper-graph --test ...`) or rubric invocation. |
| `pass_condition` | `string` | Deterministic predicate over exit code and structured output. |
| `seeded_defect` | `string` | Deliberate mutation/fixture that must turn the check red, with expected failure output. |
| `owner` | `string` | Accountable workstream or human role from `docs/owner-map.md`. |

---

## 3. The Seven Praxis Relations

Praxis edges are typed, directional, and closed to the seven relations below. No generic `RELATED_TO` or `IS_A` edge exists in `praxis`.

| Relation | Domain -> Range | Cardinality | JSONL / Manifest Carrier | Required Warrant and Semantics |
|---|---|---|---|---|
| `DERIVED_FROM` | `Lesson -> Episode` | `1..*` per `Lesson` | `Lesson.derived_from[]` | Names every historical episode from which the lesson was distilled. Prefix/continuation captures (`ep:28b5b22b` / `ep:d0e2d0bd`) count as one independent session. |
| `EVIDENCED_BY` | `Lesson \| Episode -> EvidenceItem` | `1..*` per `Lesson` (`>=1` `thread_quote`) | `Lesson.evidence[]` | Binds the rule to verbatim quotes (`thread <id>#<msg>`) and verified `path:line` code/doc observations, each carrying an explicit `verification` state. |
| `APPLIES_TO` | `Lesson \| Check -> Subject \| Stage \| Path` | `1..*` | `Lesson.applies_to`, `Check.owning_subject` | Scopes where the lesson or check governs across `docs/owner-map.md` subjects, pipeline lifecycle stages, and repository paths. |
| `ENFORCED_BY` | `Lesson -> Check` | `1..*` per `Lesson` | `Lesson.enforcement`, `Check.lessons[]` | Links a normative lesson to the structural `check_*` or `judgment (human)` gate that turns red on violation. |
| `QUALIFIES` | `Lesson \| EvidenceItem -> Lesson` | `0..*` | `Lesson.boundary` + inline `qualifies` refs | Records scope narrowing, unmet preconditions, or environmental bounds (e.g., `L03` and `L14` qualify `L01`'s disposability claim so only metadata — never derived claims — may be wiped). |
| `CONTRADICTS` | `EvidenceItem \| Lesson -> Lesson \| DocClaim` | `0..*` | `Lesson.boundary` + `status: ["Contested"]` | Preserves live counterarguments or artifact disagreements without smoothing them away (e.g., `deepseek-ontology-critique.md:36` vs `L05` closed `kind` enum; `llm-document-graph-research.md:23,643` vs `L09` panel headline). Marks affected claims `Contested`. |
| `SUPERSEDES` | `Lesson -> DraftLesson \| Lesson` | `0..*` | `Lesson.supersedes[]` | Records lineage when a refined or regraded lesson replaces an earlier formulation (e.g., `L01`–`L14` superseding Round-2 `R2:P01`–`R2:P14` after Round-3 audit corrections in `rescan.md`). Retired records stay in history and drop from active enforcement. |

---

## 4. Distillation Mechanics and Validation Rules

Per [`CONSTITUTION.md`](CONSTITUTION.md) Invariant 1 and [`GUIDELINES.md`](GUIDELINES.md) GP-08, working context becomes a `praxis` entity only on an explicit promotion trigger (an adjudicated failure, a closed mining round, or a recorded human design decision):

1. **Archive pointer over transcript copy.** `Episode.summary` and `Lesson.mechanism` state the compressed causal finding and point to `source_ref` / `locator`; temporary debugging spikes and process narrative are excluded (`L13`).
2. **Verbatim quote check (`T2.3`).** `check_praxis_schema` parses `lessons.jsonl`, asserts all 13 required keys, non-empty `status` and `derived_from`, `>=1` `thread_quote` per record, valid `docs/owner-map.md` `check_id` references, and cross-checks every quote and file line against on-disk sources. Where raw thread transcripts are absent from the repository checkout, quotes that match `round-2-rescan.json` are marked `verified_in_r2_snapshot_raw_thread_off_disk`, and quotes appearing only in `lessons.md` prose without raw message logs on disk are marked `unverifiable_raw_thread_off_disk` — never silently deleted or fabricated.

---

## 5. Worked Example — `L03` Across `Episode`, `Lesson`, and `Check`

### 5.1 `Episode` records (referenced)

```json
{"id":"ep:71ffc1b7","kind":"thread","source_ref":"71ffc1b7-6183-43a3-b95e-7f103170e669","date":"2026-11-17","independent_of":null,"role":"core-lineage","summary":"Designed Zotero-to-HawDB sync, discovered store::rebuild wipes all graph nodes via MATCH (n) DETACH DELETE n, and chose manual sync plus a separate derived DB for round 1.","raw_on_disk":false}
{"id":"ep:d9cbb41d","kind":"thread","source_ref":"d9cbb41d-773e-4917-a187-1d83c30991eb","date":"2026-05-05","independent_of":null,"role":"migration-safety","summary":"User rejected automatic startup table-rebuild migration containing DROP statements in favor of a reviewed one-off additive ALTER TABLE script.","raw_on_disk":false}
```

### 5.2 `Lesson` record (line 3 of `lessons.jsonl`)

```json
{"id":"L03","title":"Make sync manual, and never let a metadata rebuild delete derived knowledge","rule":"Run metadata synchronization only on an explicit manual trigger, isolate extracted claims and human review state outside the destructive rebuild target (or scope deletion strictly to owned metadata labels), forbid DROP TABLE or DROP VIEW in startup paths, and gate every rebuild or schema change with a count oracle.","mechanism":"src/store.rs:108-109 and src/pipeline.rs (sync_library) execute DELETE FROM works WHERE item_id IS NOT NULL followed by MATCH (n) DETACH DELETE n in one transaction with no partial or upsert path, wiping any non-metadata node stored alongside works; in a sibling SQLite store (ep:d9cbb41d), automatic table-rebuild migrations with DROP statements risked silent object loss and were replaced by a reviewed additive ALTER statement.","scope":"All CLI and HTTP sync entry points (src/pipeline.rs, src/store.rs, src/ui.rs) and any schema initialization or migration routine touching metadata or derived stores.","boundary":"Isolation via a separate derived HawDB is a recorded requirement (experiments/ontology-extraction/decisions.md:5), not yet implemented. The additive ALTER remedy is transferred from SQLite/bub_sf; HawDB ALTER TABLE support is unverified, store::initialize (src/store.rs:50-58) only creates works when absent, INSERT_WORK (src/store.rs:26-31) hardcodes 13 columns, and no schema version table exists. Qualifies L01 so 'disposable' applies only to metadata.","status":["Decided","Observed","Specified","Transferred"],"derived_from":["ep:71ffc1b7","ep:d9cbb41d"],"evidence":[{"kind":"thread_quote","locator":"thread 71ffc1b7#45","role":"user","quote":"we can make it just one-off synchronization for a proactive cause only, or some menu triggers only","fact":null,"verification":"verified_in_r2_snapshot_raw_thread_off_disk"},{"kind":"thread_quote","locator":"thread d9cbb41d#62","role":"user","quote":"this is dangerous, provide the one off migration sql to me and remove that logic","fact":null,"verification":"verified_in_r2_snapshot_raw_thread_off_disk"},{"kind":"thread_quote","locator":"thread d9cbb41d#86","role":"user","quote":"you have drops in schema? how dare you say it's a migration","fact":null,"verification":"verified_in_r2_snapshot_raw_thread_off_disk"},{"kind":"thread_quote","locator":"thread 71ffc1b7#175","role":"assistant","quote":"`store.rebuild()` currently runs `MATCH (n) DETACH DELETE n`, so a normal metadata sync would erase any extracted domain graph.","fact":null,"verification":"verified_in_r2_snapshot_raw_thread_off_disk"},{"kind":"code_fact","locator":"src/store.rs:108-109","role":null,"quote":"transaction.query_sql(\"DELETE FROM works WHERE item_id IS NOT NULL\")?;\n    transaction.query(\"MATCH (n) DETACH DELETE n\")?;","fact":"Every sync wipes all SQL work rows and detaches/deletes every graph node in one transaction.","verification":"verified_on_disk"},{"kind":"doc_fact","locator":"experiments/ontology-extraction/decisions.md:5","role":null,"quote":"Domain extraction cannot be erased by a metadata sync","fact":"Records the separate-derived-HawDB decision as an unexecuted round-1 requirement.","verification":"verified_on_disk"}],"applies_to":{"subjects":["Rebuild safety and manual metadata sync","Operations, backend selection, and additive schema migration"],"stages":["sync","migrate"],"paths":["src/store.rs","src/pipeline.rs","src/ui.rs","experiments/ontology-extraction/decisions.md"]},"enforcement":{"check_id":"check_sync_count_guard","secondary_checks":["check_migration_oracle"],"kind":"fixture","status":"planned","spec":"Populate a derived store with non-metadata nodes/claims, invoke metadata sync, and assert derived row/edge counts remain unchanged while failing closed if non-metadata labels exist in the sync target; reject DROP TABLE/VIEW in startup/migration lint."},"rejected_alternative":"Upsert-in-place or automatic startup table-rebuild inside the single metadata database — rejected because unguarded rebuild transactions wipe co-located non-metadata nodes and startup DROP statements risk silent schema/data loss without a count oracle.","supersedes":["R2:P03"]}
```

### 5.3 `Check` record (planned target for W4)

```json
{"check_id":"check_sync_count_guard","lessons":["L03","L01"],"type":"fixture","owning_subject":"Rebuild safety and manual metadata sync","path":"planned:tests/check_sync_count_guard.rs","command":"cargo test -p paper-graph --test check_sync_count_guard","pass_condition":"exit 0; non-metadata target triggers fail-closed error and separate derived DB preserves 3/3 claim and evidence counts across sync_library","seeded_defect":"disable pre-sync non-metadata label count guard in store::rebuild; test must fail with 'derived rows changed: 3 -> 0'","owner":"W3.5"}
```

### 5.4 Relation trace on `L03`

- `L03 --[DERIVED_FROM]--> ep:71ffc1b7`, `ep:d9cbb41d`
- `L03 --[EVIDENCED_BY]--> thread 71ffc1b7#45`, `thread d9cbb41d#62`, `thread d9cbb41d#86`, `thread 71ffc1b7#175`, `src/store.rs:108-109`, `experiments/ontology-extraction/decisions.md:5`
- `L03 --[APPLIES_TO]--> "Rebuild safety and manual metadata sync"`, `"Operations, backend selection, and additive schema migration"`
- `L03 --[ENFORCED_BY]--> check_sync_count_guard` (secondary: `check_migration_oracle`)
- `L03 --[QUALIFIES]--> L01` (scopes disposability strictly to bibliographic metadata, never derived extraction or review state)
- `L03 --[SUPERSEDES]--> R2:P03` (splits data-wipe sync from SQLite schema-migration evidence after Round-3 audit)
