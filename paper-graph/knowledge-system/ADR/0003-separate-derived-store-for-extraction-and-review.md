# 0003 — Separate derived store for extraction and review

- **Status:** accepted
- **Date:** 2026-11-19
- **Owner:** emliunix

## Context

Metadata rebuilds and curated knowledge have different lifetimes. `paper-graph/knowledge-system/CONSTITUTION.md` identifies the current metadata sync and migration hazards: L01 reports destructive replacement of metadata state, L03 an unguarded full wipe, and L14 unsafe startup rebuild behavior. Storing claims, review decisions, or run provenance in that rebuild target would allow routine synchronization to destroy scholarly work. L13 additionally requires explicit promotion before derived material enters durable memory. No extraction experiment or gold annotation is claimed by the constitution, so this ADR establishes boundaries before such state exists rather than inferring safety from current behavior.

## Decision

We will keep Zotero as the sole canonical bibliographic store, the HawDB metadata graph as disposable derived state, and extraction claims, evidence, review state, and run provenance in a separate derived store outside the metadata sync target. Metadata sync will be manual and fail closed whenever non-metadata rows or labels occupy its target. Distilled notes will contain only explicitly promoted summaries with evidence pointers, never automatic copies of extraction or review scaffolding. Schema changes will run as reviewed, additive, one-off scripts against a database copy, gated by an active pre/post count oracle; startup code will not drop or rebuild application objects. Cross-store references will use durable identities defined by the data-model owner, not local metadata row numbers.

## Alternatives rejected

- Add extraction and review tables beside metadata — rejected: the documented wipe boundary can erase them (L01, L03).
- Disable metadata rebuilds permanently — rejected: derived metadata must remain rebuildable rather than becoming an accidental canonical store.
- Rely on backup restoration after destructive sync — rejected: restoration is recovery, not a guard against loss or silent alteration.
- Run automatic startup migrations — rejected: unreviewed destructive changes bypass the count oracle (L14).

## Consequences

- Metadata can be discarded and rebuilt without owning the lifecycle of claims, reviews, or runs.
- Queries crossing metadata and review data require explicit joins through durable identifiers, increasing implementation complexity.
- Operators must preserve separate backup, migration, and recovery procedures even though the extraction store is derived.
- Check pointers: `check_metadata_sync_guard`, `check_sync_count_guard`, and `check_migration_oracle` (planned) will exercise fail-closed sync and pre/post counts; durable promotion remains `judgment (human)`.
