# Lesson template

One file per lesson in `lessons.jsonl`; this Markdown form is the human-readable twin.
Every field is required. A missing field is a defect, not an omission.

| Field | Rule |
|---|---|
| `id` | `Lnn` for a rule carried from lessons.md; `Pnn` for a praxis record. Stable, never reused. |
| `title` | One line, imperative or declarative; no "we should". |
| `rule` | The sentence a stranger can act on. One claim, no hedging. |
| `mechanism` | Why it holds: the code path, incident, or measurement. Named symbols and paths, not adjectives. |
| `scope` | Where it applies; the boundary of the claim. |
| `boundary` | What has *not* been shown, and the failure mode that remains. |
| `status` | One or more of Observed / Decided / Specified / Transferred / Contested (lessons.md §tags). |
| `evidence` | >=1 record: `thread <id>#<msg> (<role>): '<verbatim quote>'` or `<path>:<lines>: <fact>`. Exact substrings only; paraphrases are rejected. |
| `enforcement` | The operational check: a command, test, or lint that fails when the rule is violated. "Review carefully" is not enforcement. |
| `rejected_alternative` | The option considered and dropped, and why. Required: it is how a future reader knows the rule was a decision. |

---

## Example — L03 (worked)

```yaml
id: L03
title: Make sync manual, and never let a metadata rebuild delete derived knowledge
rule: Scope a metadata rebuild to owned metadata, or isolate extraction in a separate database, and prove it with a count oracle; never let sync delete extraction or human judgment.
mechanism: src/store.rs:108-109 and src/pipeline.rs sync_library run a full delete-and-reinsert in one transaction; there is no upsert or partial path, so any derived row stored beside metadata disappears on the next sync.
scope: Every sync entry point that touches the metadata store, including startup paths.
boundary: The additive-ALTER remedy is proven only on SQLite/bub_sf; HawDB ALTER support is unverified, INSERT_WORK hardcodes 13 columns, and there is no schema version.
status: [Decided, Observed, Specified, Transferred]
evidence:
  - thread 71ffc1b7#45 (user): 'we can make it just one-off synchronization for a proactive cause only, or some menu triggers only.'
  - thread d9cbb41d#62 (user): 'this is dangerous, provide the one off migration sql to me and remove that logic'
  - src/store.rs:108-109 - every sync runs the full delete-and-reinsert path.
enforcement: check_sync_count_guard - populate a derived DB, run a metadata sync, assert extraction/claim/evidence counts unchanged; fail closed when derived data is present.
rejected_alternative: Upsert-in-place inside the same destructive transaction - rejected because a partial path still commits the wipe when the first statement succeeds and the second fails.
```
