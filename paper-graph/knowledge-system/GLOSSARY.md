# Glossary — Paper Knowledge System

Terms are alphabetized; links identify the document that owns the term's governing definition.

- **Admission** — Acceptance of a source into the active corpus after executable byte, signature, and digest checks; not to be confused with ingestion (owner: `docs/PIPELINE.md`).
- **Arm** — A pre-registered experimental condition compared under one frozen protocol; not to be confused with a variant (owner: `docs/EXPERIMENTS.md`).
- **Assertion** — A statement proposed for acceptance that still requires evidence and review; not to be confused with a claim or evidence (owner: `docs/DATA_MODEL.md`).
- **Canonical** — The authoritative layer from which other state can be rebuilt; not to be confused with derived or distilled state (owner: `docs/ARCHITECTURE.md`).
- **Check** — An executable validation with a defined pass condition and owning subject; not to be confused with a gate (schema owner: [ADR 0004](ADR/0004-atomic-claim-evidence-with-quote-and-locator.md); subject/check binding: [owner map](docs/owner-map.md)).
- **Claim** — One atomic proposition bound to a verbatim quote and a resolvable locator; not to be confused with an assertion or its evidence (owner: `docs/DATA_MODEL.md`).
- **Concept** — An ontology object admitted only to answer a frozen competency question; not to be confused with a claim ([owner](ontology.md)).
- **Contested** — A status marking a live counterargument or unresolved contradiction; resolution requires evidence addressing both sides ([owner](method.md#governance)).
- **Decided** — A status marking an explicit human choice, whether or not that choice has been implemented ([owner](method.md#governance)).
- **Derived** — State computed from canonical inputs and disposable under a guarded rebuild; not to be confused with canonical or distilled state (owner: `docs/ARCHITECTURE.md`).
- **Distilled** — A durable summary explicitly promoted with pointers back to its evidence; not to be confused with canonical or automatically derived state (owner: `PRAXIS.md`).
- **Episode** — A recorded occurrence with its context and provenance, from which a reusable lesson may be drawn; not to be confused with a lesson (owner: `PRAXIS.md`).
- **Evidence** — A source artifact or observation used to support or challenge a statement; not to be confused with the assertion or claim it supports (owner: `docs/DATA_MODEL.md`).
- **EvidenceSpan** — A verbatim source substring with attachment, chunk, page, and verified character coordinates; it is the locator-bearing form of evidence (owner: `docs/DATA_MODEL.md`).
- **Gate** — A decision that permits, blocks, or declares a result inconclusive by evaluating named checks and human judgments; not to be confused with a check (owner: `docs/EXPERIMENTS.md`).
- **Ingestion** — Reading or importing source bytes or metadata, which alone grants no corpus status; not to be confused with admission (owner: `docs/PIPELINE.md`).
- **INTRODUCES** — An evidence-graded relation asserting that a source presents a new concept or method, not merely that it mentions or uses one ([owner](ontology.md)).
- **Lesson** — A normalized, reusable rule with mechanism, scope, evidence, and enforcement drawn from one or more episodes; not to be confused with an episode (owner: `PRAXIS.md`).
- **MENTIONS** — The weakest evidence-graded relation, recording reference without implying use or introduction; not to be confused with USES or INTRODUCES ([owner](ontology.md)).
- **Observed** — A status marking direct observation in current code, an artifact, or a recorded incident, rather than mere intention ([owner](method.md#governance)).
- **Plan** — A proposed sequence of actions and dependencies; not to be confused with a specification, which states intended behavior, or with the Specified status (owner: `docs/ROADMAP.md`).
- **Reviewed** — Having a recorded review verdict and reviewer provenance, without implying acceptance or reviewer independence; not to be confused with unreviewed (owner: `docs/REVIEW.md`).
- **Specified** — A status marking documented intended behavior that has not been executed; not to be confused with a plan or an observed result ([owner](method.md#governance)).
- **Transferred** — A status marking support imported from another project but not yet established in this one ([owner](method.md#governance)).
- **Unknown** — An explicit missing or ambiguous value that must remain visible rather than being coerced to a default; not to be confused with unreviewed (owner: `docs/DATA_MODEL.md`).
- **Unreviewed** — Lacking a recorded review verdict, regardless of whether tests have run; not to be confused with reviewed, unknown, or untested (owner: `docs/REVIEW.md`).
- **Untested** — A status marking the absence of an executed test in this context; not to be confused with unreviewed, which concerns judgment rather than execution ([owner](method.md#governance)).
- **USES** — An evidence-graded relation asserting application of an existing concept or method, without implying that the source introduces it; not to be confused with MENTIONS or INTRODUCES ([owner](ontology.md)).
- **Variant** — A concrete version of an artifact, prompt, configuration, or document that need not define a comparison condition; not to be confused with an arm (owner: `docs/EXPERIMENTS.md`).
- **Work** — A bibliographic entity anchored by durable identity such as its Zotero key and DOI, rather than by title or content hash (owner: `docs/DATA_MODEL.md`).
