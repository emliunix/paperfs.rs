# 0001 — Freeze docs before worktrees

- **Status:** accepted
- **Date:** 2026-11-19
- **Owner:** emliunix

## Context

Worktrees are useful only when every participant reads the same governing documents. The worked example in `paper-graph/knowledge-system/_templates/adr.md` identifies the risk of creating trees from the code baseline without the knowledge-system documents. `paper-graph/knowledge-system/CONSTITUTION.md` fixes `f1e46d9` as the observed code baseline, while `worklog/2026-11-19/todo.md` makes W1 the documentation spine that unblocks later work. L13 warns against allowing temporary process material to become uncontrolled durable state; L08 limits what pre-verdict artifacts may claim. A mutable or manually copied document set would make review provenance ambiguous and could send downstream worktrees in different directions.

## Decision

We will commit the knowledge-system documents and the 2026-11-19 work plan before creating any project worktree, then tag that commit `pg-base`. Every W1-and-later worktree will start from `pg-base`; `f1e46d9` remains the code baseline rather than the documentation baseline. We will record the frozen document digest in `_runs/2026-11-19/docs.sha256` and treat that digest as an input to later review and evaluation work. No agent will silently edit frozen docs inside a worktree. A necessary correction requires an explicit review, a new commit and tag, a regenerated digest, and an update to dependent run manifests before further work claims the new baseline.

## Alternatives rejected

- Base worktrees on `f1e46d9` and copy documents manually — rejected: copies can drift and provenance is not hashable.
- Read documents from a shared directory outside git — rejected: concurrent edits can change the rules during a run.
- Track the moving head of the documentation branch — rejected: two trees can then legitimately resolve different rules under the same work identifier.

## Consequences

- Every worktree receives an identical, reviewable document set, and later artifacts can name their exact baseline.
- Documentation corrections become deliberate baseline changes rather than invisible local edits; this adds coordination overhead.
- W5 and other downstream consumers must record the selected tag and digest without claiming empirical validation from documentation alone (L08).
- Verification: an explicit script step verifies the worktree base, tag resolution, and recorded digest; `judgment (human)` approves any re-freeze.
