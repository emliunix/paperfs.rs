# 0004 — Atomic claim evidence with quote and locator

- **Status:** accepted
- **Date:** 2026-11-19
- **Owner:** emliunix

## Context

A claim is auditable only when another reviewer can resolve its exact support. `paper-graph/knowledge-system/CONSTITUTION.md` connects this requirement to unstable local identifiers (L02), unverified corpus attachments (L04), unearned taxonomy (L05), inflated relation semantics (L06), and overlapping or silently defaulted evidence (L07). A quotation without coordinates is difficult to relocate; coordinates without a verbatim quote can drift during parsing. Compound propositions also prevent atom-level review and evaluation. These are design constraints, not measured extraction results.

## Decision

We will represent each claim as one atomic proposition bound to a verbatim source substring and a resolvable locator containing `attachment_item_id`, `chunk_id`, page, and verified character offsets. Works will persist `zotero_key` and DOI as durable identity; numeric Zotero `itemID` remains a local join key, and neither title nor content hash substitutes for identity. Corpus admission will require executable verification of local bytes, PDF signature, and SHA-256 before claims enter an active corpus. Missing or ambiguous support will remain `unknown` or `unreviewed`. Ontology predicates will grow only from frozen competency questions, with evidence-graded `MENTIONS`, `USES`, and `INTRODUCES` roles; a mention will not be promoted to use or introduction without support. Overlapping evidence spans will be reconciled so they cannot count as independent support.

## Alternatives rejected

- Attach a claim to an entire chunk without exact offsets — rejected: support becomes ambiguous and overlaps can be double-counted (L07).
- Identify works by title or content hash — rejected: merges, restores, and editions can break assertions (L02).
- Infer missing values with model defaults — rejected: absence would become fabricated evidence.
- Start with a broad closed taxonomy — rejected: schema would outrun competency questions and graded evidence (L05, L06).

## Consequences

- Reviewers can substring-check quotes, resolve source locations, and adjudicate each atom independently.
- Extraction and review require more records and explicit handling of parser offsets, duplicates, and unknowns.
- Re-imports must demonstrate that durable identity and evidence coordinates survive; unverified attachments remain excluded.
- Check pointers: `check_corpus_identity`, `check_evidence_locator`, `check_corpus_admission`, and `check_question_lint` (planned) enforce these boundaries; gold adjudication remains human-owned.
