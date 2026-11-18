# Paper knowledge ontology v0

**Status:** design draft, not implemented. The Rust crate builds only Work, Person, Collection, Tag, Venue, and Attachment nodes. There is no Concept or Claim node, no chunk/page/offset table, and no extraction arm. This document is a specification to cut down and test, not a description of a running graph.

## Purpose

Represent the user's PL/type-theory papers so that a question can be answered with a claim, an exact source span, and a direction — not just a list of topically similar PDFs. The ontology is deliberately a property graph, not RDF/OWL: provenance, retrieval, and review matter more than logical inference over class definitions.

## Design principles

1. **No unearned taxonomy.** There is no generic IS_A/subclass edge in v0. Algebraic effects, delimited control, free monads, and continuations are related concepts, not subclasses of one another without an explicit formal statement.
2. **Identity is not similarity.** Encoding, generalization, dependence, and comparison are different relations.
3. **Attribution is explicit.** A survey's description of a technique is not the survey's introduction of it.
4. **Evidence is part of the assertion.** No bare triples.
5. **Claims are not concepts.** A theorem, empirical result, or design claim gets a Claim node with its own evidence and qualifiers.
6. **Keep the first version small.** Extend only when a pre-registered competency question cannot be answered.

Principle 1 has a known loophole: a closed, single-valued Concept.kind enum can encode the same subsumption that a named IS_A edge would. The current draft has both a 17-relation catalogue and a closed kind enum, written before any competency question was frozen. Cut first, then test.

## Node types

| Type | Purpose | Identity |
|---|---|---|
| Work | A paper in the Zotero library | Zotero numeric itemID locally; zotero_key/DOI as durable address |
| Attachment | Source PDF/HTML | Zotero attachment itemID |
| Chunk | Stable parser-derived text unit | attachment id + parser version + ordinal + content hash |
| Concept | Canonical formal notion | reviewed concept id; aliases are separate links |
| Tool | Concrete implementation or system | reviewed tool id |
| Claim | Atomic theorem, definition, empirical result, or design claim | claim id; run id; review state |
| EvidenceSpan | Exact source quote and location | attachment/chunk/page/offset + quote hash |
| Experiment | A described evaluation, not a related-work comparison | local id; review state |
| BenchmarkSuite | Named corpus or suite | local id; review state |

For the first pilot, the research report recommends only Work, Concept, Tool, Claim, EvidenceSpan, and Chunk. Experiment and BenchmarkSuite are added only when a competency question needs them. The current design has nine node types; that is a catalogue, not a pilot schema.

## Concept.kind

| Kind | Examples |
|---|---|
| technique | bidirectional typechecking, normalization by evaluation, recursion schemes |
| formal_system | System F, MLF, a specific dependent type theory |
| language_feature | higher-rank polymorphism, algebraic effects, delimited control, instance parameters |
| mathematical_object | fixed point, initial algebra, free monad, continuation |
| property | soundness, completeness, decidability, confluence, principal typing |
| proof_method | logical relations, induction, coinduction, translation |
| representation | elaboration representation, syntax representation, handler representation |
| needs_review | a difficult categorization that should not invent a class |

The kind is a coarse presentation field, not a subclass graph. A disputed categorization stays needs_review. The DeepSeek critique's warning stands: a single-valued closed enum can still imply subsumption. Before the pilot, make kind faceted, multi-valued, and attributed (who assigned it, on what evidence), or drop it from scoring.

## Relations

| Relation | Domain -> range | Required evidence and exclusion |
|---|---|---|
| INTRODUCES | Work -> Concept or Tool | Explicit own-contribution/definition/novelty evidence; not "X introduced" in related work; not a priority claim |
| USES | Work -> Concept or Tool | The work's method/proof/implementation actually uses it; not a passing mention |
| MENTIONS | Work -> Concept | Explicit occurrence without enough evidence for USES; low-information, never promoted automatically |
| EXTENDS_WORK | Work -> Work | Explicit statement that the citing work builds on/extends the cited work |
| EXTENDS_CONCEPT | Work -> Concept | Explicit own extension of a named formalism/technique |
| GENERALIZES | Concept -> Concept | Explicit formal generalization, direction general -> special; not transitive in the query layer unless separately proven |
| DEPENDS_ON | Concept -> Concept | Explicit prerequisite/dependency, not co-occurrence |
| RELATED_TO | Concept <-> Concept | Weak curated association; symmetric display hint only, non-transitive, never a subclass |
| ASSERTS | Work -> Claim | The work itself advances the claim; background claims carry attribution |
| ABOUT | Claim -> Concept/Tool/Experiment | Claim subject linkage; evidence is still mandatory |
| HAS_EVIDENCE | Claim/Assertion -> EvidenceSpan | stance = supports / refutes / qualifies; exact quote and locator |
| REPORTS | Work -> Experiment | The paper reports the evaluation |
| EVALUATES | Experiment -> Concept/Tool | Object of the reported evaluation |
| ON_SUITE | Experiment -> BenchmarkSuite | Explicit suite identification; unnamed suite remains provisional |
| IMPLEMENTS | Tool -> Concept | Explicit implementation evidence |
| CITES | Work -> Work | Bibliographic citation plus context; citation_function is separate and may be unknown |
| ALIAS_OF | Surface form -> Concept | Reversible alias decision, not an assertion that two concepts are equal |

Seventeen relations is too many for an unrun pilot. Start with the five that the first competency questions need: MENTIONS, USES, INTRODUCES, ASSERTS, HAS_EVIDENCE. Add EXTENDS/CITES/GENERALIZES only when a question cannot be answered without them.

## Required negative distinctions

| Text situation | Correct output | Incorrect output |
|---|---|---|
| "Unlike bidirectional typechecking, our algorithm uses unification." | MENTIONS bidirectional typechecking; USES unification if it is the method | USES bidirectional typechecking |
| "Dunfield and Krishnaswami introduce X." in another paper | CITES with context; attribution to the original work if linked | Current survey INTRODUCES X |
| "We introduce a bidirectional algorithm for S." | Current Work INTRODUCES the named algorithm, not every concept in the sentence | Current Work INTRODUCES bidirectional typing as a whole |
| "Effects can be expressed using delimited continuations." | Explicit encoding/DEPENDS_ON only if defined in the manual; otherwise RELATED_TO plus claim | Delimited control GENERALIZES algebraic effects |
| "Following Reynolds [12], we ..." | CITES; possibly USES with exact context | EXTENDS_WORK automatically |
| "We extend MLF ..." | EXTENDS_CONCEPT MLF; CITES if a specific work is cited | RELATED_TO, or extension of an unrelated cited paper |
| Theorem conclusion quoted without a positivity/termination premise | needs_revision, incomplete evidence | Approved unconditional claim |
| "X is not decidable" / "If R holds, X is complete" | Negative/conditional claim with qualifier | Positive unconditional claim |
| "Free monads" and "freer monads" | Separate concepts unless the text explicitly establishes identity/generalization | Merge because names are similar |
| "effect" as ordinary side effect vs "algebraic effect" | Separate surface meanings; provisional if ambiguous | Merge by token overlap |

## Evidence model

A Claim is atomic: one independently evaluable proposition, with its necessary condition kept attached. It carries:

- normalized statement, claim type, polarity, qualifiers, attribution;
- review state (unreviewed / needs_revision / approved / rejected);
- run id and schema/prompt version.

An EvidenceSpan carries:

- attachment_item_id, chunk_id, page, quote, start_char, end_char;
- stance (supports / refutes / qualifies);
- quote-validation state and locator-validation state.

Claims and evidence are many-to-many. Evidence is the only leaf; claims may reference notes and earlier claims, so the document is a directed acyclic graph flattened in reverse dependency order. One sentence copied into two overlapping chunks is one support, not two. A fabricated locator is a hard failure, not a low score.

Until a parser and a chunk/page/offset table exist, evidence can only be quote-checked against Zotero metadata or abstracts. Locators must be marked unavailable, not assumed.

## Identity and resolution

- Work identity: Zotero itemID locally; zotero_key and DOI as the durable address. No content-hash identity. A merge/restore that renumbers itemIDs must not orphan claims.
- Concept identity: reviewed canonical id; aliases are reversible ALIAS_OF links with their own evidence. A merge is a reviewed union decision, not a destructive node rewrite; all source mentions and evidence links survive.
- Ambiguous references stay unresolved (ExternalWork with a null cited_work_item_id), never a nearest-title guess.
- Candidate-generation recall is reported separately from adjudication precision. Pairwise merge precision is gated at .98; one false merge is reported as a cluster-level incident with its downstream edge count.
- Free monads vs freer monads, graded vs coeffects, and ordinary effects vs algebraic effects are the canonical negative cases.

## What this ontology is not

- It is not a generic scholarly ontology. The domain vocabulary is PL/type theory.
- It is not a large OWL ontology. There is no reasoner and no class hierarchy.
- It is not a GraphRAG clone. Community summaries are deferred; the first deliverable is a browsable list of approved claims with exact PDF quotes.
- It is not implemented. No Concept or Claim node exists in src/; the 17 relations are markdown.

## Open issues before the pilot

1. Freeze the 20 competency questions and cut the vocabulary to what they need.
2. Decide whether Concept.kind is faceted/multi-valued/attributed or removed from scoring.
3. Rename or qualify INTRODUCES so it cannot be read as "first".
4. Implement EvidenceSpan with page/offset validation against parsed text.
5. Add a re-import test that changes itemIDs and asserts claim/evidence survival.
6. Resolve the parser and chunking policy on the four development PDFs before annotating.
