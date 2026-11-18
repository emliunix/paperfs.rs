# DeepSeek ontology critique

**Date:** 2026-11-18  
**Role:** uncited secondary design critique, not primary evidence.  
**Route:** `dsh headless -`, model alias `deepseek-flash`; the configured
`DEEPSEEK_API_KEY` was passed in the process environment without being printed.
Only generic ontology text was sent; no Zotero papers or personal data were
included. The initial attempt without that environment failed with
`MISSING_CREDENTIAL`.

The final response is preserved below. Dsh's intermediate reasoning is not
included.

## Verdict

The provenance-first skeleton is sound, and the three prohibitions (citation≠extension, mention≠use, relatedness≠generalization) are the right ones. But the model is still **document-shaped** (works, chunks, spans, quotes) where the domain is **argument-shaped** (propositions, ascriptions, disputes, hypotheses). Most errors below are symptoms of that mismatch. Second theme: the ontology states prohibitions but omits the vocabulary needed to obey them, so `RELATED_TO` becomes the pressure-release valve for everything unmodelled.

## 1. Most serious remaining type/relation errors

**1. Attribution is a field, not a node — so stance has no target.** The schema literally writes `Claim/Assertion HAS_EVIDENCE`, treating proposition and ascription as one thing. Consequences: you cannot represent two works asserting incompatible claims about the same concept; you cannot represent "A reports that B claimed X, and refutes it"; and `stance=refutes` is ill-typed, because the refuting span lives in a *different* Work than the asserting one — so it refutes *B's assertion*, not A's claim. Fix: `Claim` = proposition; `Assertion` = (agent/Work, Claim, context, polarity, modality); `EvidenceLink` = (Assertion, span, stance, warrant); `Assertion DISPUTES Assertion`. Also add `Claim CONTRADICTS/ENTAILS Claim` — right now claim-vs-claim conflict is smuggled through a span, which is a category error.

**2. `Claim` is unary.** `Claim ABOUT Concept-or-Tool` gives a bag of topics and no proposition. "This extension is conservative over the base", "the translation is type-preserving for the pure fragment", "2.3× faster than GHC on nofib", "more expressive than [x]" are n-ary, comparative, and hypothesis-scoped. With a unary ABOUT edge, claim-evidence validation collapses to lexical overlap, and the qualifier field becomes unaddressable free text. Fix: controlled `claim_type` vocabulary + ordered relata + hypotheses + modality + scope. Comparative claims need ≥2 relata and a dimension; without that, your most citation-worthy claims are unrepresentable.

**3. `Work INTRODUCES` is the wrong subject/arity and smuggles priority.** Either it means "a contribution of this work" — then its subject is the contribution list, not the Concept, and it should be `Work CONTRIBUTES Concept` — or it means "first", which is a time-indexed, global, defeasible priority assertion the ontology cannot express (no year, no firstness, no multi-origin). As written it will be read as "first" downstream and will silently conflict with `EXTENDS_CONCEPT`/`GENERALIZES` on the same pair. Also missing: `RENAMES`, `REPACKAGES`, `SUPERSEDES`, `SYNONYM_OF`.

**4. `EvidenceSpan` is doing three incompatible jobs, and the evidence type is a monoculture.** It is simultaneously a text locator, the support relation, and the stance carrier. Worse, "exact quote/chunk/page" is wrong for most PL claims: a theorem's evidence is its statement plus proof (possibly mechanized); a performance claim's evidence is a table/figure/artifact/configuration, not a quotable sentence. A table row is not a quote. Fix: span = typed locator (prose/definition/theorem/proof/table/figure/code + offsets); separate `EvidenceLink` carrying stance **and warrant** (the interpretation step); add `insufficient`/`ambiguous`/`not_checked` to the stance enum — `qualifies` is not a substitute; add defeaters.

**5. The relation set is missing the PL-critical edges, and `DEPENDS_ON` conflates four things.** Absent: `ALIAS_OF`/`SYNONYM_OF` (the #1 practical failure mode — without it the graph fragments), `EQUIVALENT_TO`/`ENCODES`, `INSTANCE_OF` (distinct from generalization), `REFINES`/`SUPERSEDES`, `CONSERVATIVE_EXTENSION_OF`, `INCOMPATIBLE_WITH`, `PART_OF` (feature→language), `ASSUMES`/`UNDER_HYPOTHESIS`. And `DEPENDS_ON` currently merges semantic dependency, implementation dependency, proof dependency, and prerequisite. Because these are missing, `RELATED_TO` will absorb all of them — a legitimate catch-all that is also an unfalsifiable hedge.

**6. No identity, scope, or version layer.** Concepts are theory-relative and version-relative ("linear type" in linear logic vs. Rust; "graded" vs. "coeffects" in partly-overlapping senses). With no alias edge, no scope/theory node, and no definition locus, you will both merge distinct notions and split identical ones — and a single bad merge poisons dozens of edges. `Tool` also mixes software artifacts with machine-checked developments (a Rocq/Lean library is artifact, formalization, and evidence at once), and `BenchmarkSuite` mixes corpus, harness, and configuration. No version ⇒ "GHC", "Lean", "nofib" claims are unfalsifiable and cross-paper comparisons unsound.

**7. The measurement layer is a stub.** `Work REPORTS Experiment`, `Experiment EVALUATES X`, `ON_SUITE Y` has no Metric, Baseline, Configuration, or Result. "2.3× faster" is not checkable, and `Experiment EVALUATES Concept` is ill-typed: you evaluate an implementation/configuration, never a concept in the abstract.

**8. No modality, strength, or absence.** Theorem vs. conjecture vs. remark vs. informal argument are indistinguishable; "we believe this scales" is quotable and will be scored as supported. Negative and absence claims ("does not support polymorphic recursion", "no prior work", "proof omitted as standard") have no representation except polarity, which is not the same thing.

**9. Banning `IS_A` did not remove subsumption — it relocated it.** It now lives in the closed, single-valued, mixed-granularity `kind` enum (a type system is simultaneously technique, formal_system, and representation), with no provenance and no multi-assignment. That is the thing you banned, in a worse place. Kinds should be faceted and multi-valued with attribution, not a scalar.

**10. `Chunk` is a pipeline artifact, not a domain entity.** Its identity depends on chunking parameters, it has no domain relations, and it will leak into analytics. Make it a locator/index; keep `EvidenceSpan` as the only first-class textual object. Relatedly: no declared relation-subsumption lattice, so `MENTIONS` coexists with `USES`/`INTRODUCES` and degree centrality will be read as importance.

## 2. Subtle negative examples (PL papers)

- **Presentation variant:** "For presentation we use a variant of the calculus of [x], equivalent to it." → `EQUIVALENT_TO`/`ENCODES`, not `INTRODUCES` and not `EXTENDS_CONCEPT`. Repackaging is not contribution.
- **"Extension" as a technical property:** "Our extension is conservative over the base." → a `Claim` whose *topic is the relation itself*; emitting `EXTENDS_WORK` from the token "extension" is a lexical trap. A later paper can refute this claim — which requires claims to be able to be about relations.
- **Direction inversion:** "The simply typed λ-calculus is the special case of our system." Surface order runs special→general; `GENERALIZES` must be emitted general→special and validated against cue words ("special case", "instance", "generalizes"), never adjacency.
- **Extension ≠ generalization:** adding mutable state yields a syntactic superset but destroys the old metatheory. `EXTENDS` and `GENERALIZES` are orthogonal; naming them adjacently guarantees conflation.
- **Notation borrowing:** "Following [x], we write τ for types." → `MENTIONS` with citation_function `notation`; not `USES`, not `DEPENDS_ON`.
- **Baseline reimplementation:** "We compare against [x]'s system, which we reimplemented for fairness." → the Experiment `EVALUATES` the *competitor* tool and `USES` nothing of [x] conceptually. Conversely, "we use the same suite as [x]" is not "evaluates [x]".
- **Suite variant:** "nofib, excluding three benchmarks that don't compile." → `ON_SUITE` needs a variant/subset qualifier, or all cross-paper numbers compare different suites under one name.
- **Related Work prose:** "Our work is related to gradual typing" is a discourse label, not a conceptual relation. Mining `RELATED_TO` from Related Work sections yields thousands of harmless-looking, useless edges.
- **Alias with disclaimed equivalence:** "'Graded types' [x] calls 'coeffects' [y], though the systems differ in …" → a *qualified* `ALIAS_OF`, not a node merge.
- **Gloss as evidence:** intro says "our translation preserves typing"; the actual claim is Theorem 4.1, stated for the pure fragment, proved in the appendix by logical relations. Evidence = statement + proof; the intro is a gloss; the fragment restriction belongs in qualifiers, and the claim without it is false. Note the *proof method* is a Concept `USES`, not `INTRODUCES`, even when it is the contribution.
- **Priority disclaimer:** "We do not claim novelty of the calculus; it is due to [x]" / "the idea is folklore." → explicitly *suppresses* `INTRODUCES`; attribution = `folklore` with no supporting span. Symmetrically, "we are not aware of prior work" is a negative novelty claim at low confidence, never a priority fact.
- **Absence/comparison:** "Unlike [x], we require no global uniqueness check" → negative property claim, comparative, attributed to a cited work.
- **Version drift:** "GHC" in 2005 vs. GHC 9.6, "Lean" pre-4 vs. Lean 4 — tool claims are version-relative and must not merge.

## 3. Highest-value deterministic validators

1. **Quote-substring check** against normalized chunk text (NFKC, whitespace collapse, dehyphenation, ligatures); record the normalized match. Cheapest high-precision check available — it turns hallucinated evidence into a hard failure. Bound quote length; reject bare stopwords.
2. **Evidence closure and provenance consistency:** every Claim/Assertion has ≥1 evidence link; every span resolves (page in range, offsets in chunk); an external-source span cannot carry `supports` for the paper's own claim.
3. **Cue-constrained relation validation:** `USES` requires a first-person method cue ("we use/adopt/implement"), with a veto list ("we do not use", "unlike", "following"); `INTRODUCES` requires a contribution cue *and* absence of disclaimer cues ("standard", "folklore", "due to", "we do not claim"). Reject edges whose only support is a token match.
4. **Citation/extension separation:** `EXTENDS_WORK` requires a concept-level extension or `EVOLVES_FROM`; a citation with function `background|baseline|contrast` may not co-occur with `EXTENDS_WORK` from the same work. Deterministically catches the citation⇒extension leak.
5. **Direction and acyclicity:** `GENERALIZES`/`DEPENDS_ON` acyclic, no self-loops, no bidirectional generalization path; `RELATED_TO` symmetric, irreflexive, never transitively closed; forbid `RELATED_TO` where a stronger relation exists.
6. **Numeric grounding:** any numeric claim's parsed value must appear in the span with unit normalization (catches 2.3 vs 1.3, × vs %, mean vs per-benchmark); comparative claims must name baseline, metric, configuration, and suite variant, else `incomparable`.
7. **Claim well-formedness:** predicate from controlled vocabulary, polarity + modality set, ≥1 relatum, hypotheses present for conditional claim types; `modality=theorem` requires a proof locus or explicit `proof_omitted`.
8. **Domain/range and kind matrix:** `Tool IMPLEMENTS Concept` requires an artifact locator; `Experiment EVALUATES` a bare Concept without an implementation link is flagged `evaluates_uninstantiated_concept`; Concept/Tool label collisions require an explicit `IMPLEMENTS`/`DISTINCT_FROM` decision.
9. **Identity blockers:** normalized-label collisions and near-duplicates (edit distance/token Jaccard) block merges pending review; blocklist ambiguous short names ("effect", "row", "linear", "borrow", "graded", "mode").
10. **Support-inflation and circularity:** dedupe spans by fingerprint so one sentence cannot count as ten evidences; flag claims whose only support is the asserting work and which are then used to justify an edge; hash spans at review time and auto-demote approvals on hash mismatch (`stale`).
11. **Aggregate stance coherence:** a claim with only `refutes` from its own work, or only `qualifies`, is incoherent/inconclusive; the aggregate status must be stanced, never boolean.
12. **Citation resolution:** every `CITES` resolves; `[1–3]` expands to N edges; no self-citation misclassification; citation cluster spans attributed per clause, not per work.

**Human gates (not automatable):** (a) origin/priority — `INTRODUCES` read as "first" always needs adjudication, multi-origin is normal; (b) concept merge/split and qualified alias — the highest-leverage gate, sample-based; (c) direction of `GENERALIZES`/`EXTENDS`/`CONSERVATIVE_EXTENSION`, because inversions are structurally load-bearing; (d) warrant adequacy for `supports`, stratified by claim type (one-line proofs, expressiveness, complexity); (e) negative/absence claims, which invert defaults; (f) stance adjudication when support and refute coexist — adjudicate, never majority-vote; (g) gold construction with IAA and adjudication. Run the validators *on gold*: a gold item failing a validator is a triage event, not something to suppress.

## 4. Why strict fully-evidenced relation F1 is the wrong headline

- **It's a conjunction wearing one number.** "Fully evidenced" = right edge type ∧ right endpoints ∧ right span ∧ right stance ∧ right qualifiers ∧ reviewed. F1 over that is a product of component accuracies, dominated by the weakest, and uninterpretable: 0.4 cannot be attributed to extraction, coreference, span-matching, or review throughput. Report the factorized metrics and a loss cascade.
- **Abstention becomes the dominant variable.** Precision on a self-selected "fully evidenced" subset is a selective-prediction trap: drop 40% of hard edges and F1 on the retained set rises. Discarded edges must stay in the denominator as FN. Report risk–coverage / precision at fixed recall, never precision at each system's natural output.
- **It rewards banality.** Recall is measured against an unbounded, arbitrary gold boundary; "all relations" is not well-defined. The metric therefore favours high-frequency, cue-rich `CITES`/`USES`/`INTRODUCES` and punishes exactly the rare relations that carry the design's value (`GENERALIZES`, `DEPENDS_ON`, conservative extension, refutation). `RELATED_TO` then becomes a scoring escape hatch: collapse any uncertain edge into it and it counts as correct. Score `RELATED_TO` separately and report its hedge rate.
- **It doesn't measure the thesis.** The design's actual claim is about prohibitions — citation must not imply extension, mention must not imply use. F1 over positive edges is blind to violations of prohibitions. Spurious-edge audit rate must be a co-headline.
- **It treats asymmetric errors as symmetric.** Inverting `GENERALIZES` is a semantic catastrophe that flips a partial order and corrupts every downstream query; a span off by one sentence is not. F1 counts both as one miss. Direction and stance need their own confusion matrices, separate from existence errors.
- **Its items aren't independent.** One bad concept merge generates a cluster of FP/FN across dozens of "independent" edges and dominates the score while misattributing the cause. Error accounting must be cluster-aware (per merge, per work).
- **Gold is review-limited, so the metric measures reviewer allocation.** Reviewed subsets are salient, not random; F1 there is biased and not comparable across corpora. And the ceiling is human IAA: if κ on generalization direction is 0.5, an F1 of 0.85 is meaningless.

**Recommended headline panel instead:** per-relation-type P/R and macro-F1 with a trivial-baseline floor; separate existence-vs-type-vs-direction-vs-stance confusion matrices; evidence exactness rate (substring + span minimality + duplicate-span penalty) as its own audited number; selective F1 at matched coverage plus a risk–coverage curve; spontaneous-edge violation rate; downstream provenance-traced QA accuracy on canonical questions ("what supersedes what", "what is contested", "what does this depend on"); and human audit minutes per 100 accepted edges. Report human IAA as the ceiling on every one of them.

## Parent disposition

This critique is not adopted wholesale. In particular, round 1 keeps a small
ontology, does not assume generalization must always be acyclic, and does not
turn superficial first-person cue heuristics into automatic semantic approval.
Its strongest adopted cautions are factorized metrics, matched coverage,
high-risk violation counts, version/evidence limitations, and review burden.
Claims/propositions shared across works, table/proof evidence, and a larger
relation vocabulary require later competency questions before expansion.
