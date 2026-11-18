# Grounding

## Assets already verified

- The live metadata graph contains 324 works and uses Zotero numeric `itemID`
  as Work identity.
- The 15 works in `round-001-corpus.json` each have an attachment row, at least
  one existing local PDF, and a literal `%PDF-` file signature, checked on
  2026-11-18.
- Four earlier candidates (`60`, `47`, `5`, `8`) had dangling PDF attachment
  records and were excluded rather than treated as readable documents.
- Exploratory metadata/abstract hybrid search returned a sensible rank-1 work
  for four simple topic questions. Those questions are therefore non-regression
  controls, not the primary evidence that an ontology helps.
- The sourced research report supports schema-constrained extraction, exact
  provenance, conservative identity resolution, and document-level evaluation.
  It does not establish extraction quality on this PL corpus.

## Claims under test

### C1 — Domain grounding

A curated PL ontology improves canonical concept linking and role/relation
semantics over a generic scholarly schema, not merely over no schema.

**Falsifier:** the PL arm does not improve canonical concept F1 and per-relation
macro-F1 over the generic arm on held-out papers, or its improvement comes with
a high-risk semantic violation rate above the pre-registered gate.

### C2 — Evidence improves trust, not automatic truth

Exact source locations and deterministic validation reduce fabricated or
mislocated evidence, while human review is still required for semantic
sufficiency.

**Falsifier:** schema-valid outputs still contain unverifiable canonical evidence,
or reviewers judge that exact quotes routinely fail to support the complete
assertion despite passing deterministic checks.

### C3 — Graph value is compositional

A reviewed domain graph improves answers that require distinguishing
`MENTIONS`/`USES`/own-contribution, citation/extension, direction, qualifiers,
and supporting evidence.

**Falsifier:** the reviewed graph does not improve the mean evidence-supported
answer score over metadata/abstract hybrid search, or simple retrieval quality
regresses materially.

### C4 — Resolution must be conservative

Alias-aware resolution improves clustering over exact-name merging without
merging related-but-distinct notions such as free monads and freer monads.

**Falsifier:** pairwise merge precision falls below .98, B-cubed cluster F1 does
not improve, or one bad merge creates a material cluster of downstream errors.

## What this round cannot establish

- Broad scientific significance across PL literature.
- Full-document mention recall outside densely annotated sections.
- Parser quality before a real PDF parse smoke test.
- The value of community summaries, fine-tuning, or a large OWL ontology.
- Whether a particular provider is best; model/provider is held fixed in round 1.
