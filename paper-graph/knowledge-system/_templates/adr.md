# ADR template

Copy to `knowledge-system/ADR/NNNN-<slug>.md`. An ADR records a decision that is expensive to reverse; small choices stay in the owning doc.

| Field | Rule |
|---|---|
| `Number` | Zero-padded, sequential, never reused. |
| `Status` | proposed / accepted / superseded by NNNN / rejected. |
| `Date` | ISO date the decision was taken, not the date the file was written. |
| `Owner` | The person who owns the consequence. |
| `Context` | The forces: constraints, measurements, and the cost of being wrong. Link evidence; do not restate it. |
| `Decision` | One paragraph, active voice: "We will ...". |
| `Alternatives rejected` | At least one, each with the reason. |
| `Consequences` | What becomes easier, what becomes harder, what must now be measured. |

---

## Example — ADR 0001 (worked)

```markdown
# 0001 — Freeze the knowledge docs before the first worktree

- **Status:** accepted
- **Date:** 2026-11-19
- **Owner:** emliunix

## Context

`git status` shows `?? paper-graph/knowledge-system/`. A worktree created from
`f1e46d9` therefore contains no lessons, no ontology, and no plan (R14).

## Decision

We will commit the knowledge-system docs and the 2026-11-19 work plan before
creating any worktree, and tag that commit `pg-base`. `f1e46d9` stays the code
baseline.

## Alternatives rejected

- Base every worktree on `f1e46d9` and copy the docs in by hand — rejected: the
  copies drift and every tree starts from a different document set.
- Keep the docs outside git and mount them — rejected: unverifiable provenance.

## Consequences

- Every tree starts from an identical, hashable document set.
- The docs hash in `_runs/2026-11-19/docs.sha256` is the frozen W5 input.
- Any doc edit after the tag needs an explicit re-hash and a new tag.
```
