# Evaluation methodology

**Status:** pre-registered design, not executed. The gate ledger marks plan review, PDF parse, annotation manual, gold, downstream questions, seeded defects, model freeze, U2/S2/P2, deterministic scorer, blind scoring, and verdict all "no". No extraction arm has run; no spend has been incurred.

## Principle

Build the smallest end-to-end artifact over the real corpus, then pre-register one comparison. The search results should tell us how much ingestion sophistication is justified before paying for OCR, extraction, and annotation.

The first end-to-end artifact is already built: 324 Zotero works, metadata and abstracts, BM25 plus exact vector plus weighted-RRF hybrid search, a loopback UI, and a manual full sync (35.238 s, 26,466 bge-m3 tokens). The extraction layer is not built.

## Research questions and falsifiers

**C1 — Domain grounding.** A curated PL ontology improves canonical concept linking and role/relation semantics over a generic scholarly schema, not merely over no schema.

Falsifier: the PL arm does not improve canonical concept F1 and per-relation macro-F1 over the generic arm on held-out papers, or its improvement comes with a high-risk violation rate above the pre-registered gate.

**C2 — Evidence improves trust, not automatic truth.** Exact source locations and deterministic validation reduce fabricated or mislocated evidence, while human review is still required for semantic sufficiency.

Falsifier: schema-valid outputs still contain unverifiable canonical evidence, or reviewers judge that exact quotes routinely fail to support the complete assertion despite passing deterministic checks.

**C3 — Graph value is compositional.** A reviewed domain graph improves answers that require distinguishing MENTIONS/USES/own-contribution, citation/extension, direction, qualifiers, and supporting evidence.

Falsifier: the reviewed graph does not improve the mean evidence-supported answer score over metadata/abstract hybrid search, or simple retrieval quality regresses materially.

**C4 — Resolution must be conservative.** Alias-aware resolution improves clustering over exact-name merging without merging related-but-distinct notions.

Falsifier: pairwise merge precision falls below .98, B-cubed cluster F1 does not improve, or one bad merge creates a material cluster of downstream errors.

## Design

- **Corpus.** 15 papers in experiments/ontology-extraction/round-001-corpus.json: 4 development (one per domain) and 11 held-out test. Domains: type inference, recursion schemes, effects/control, dependent types/Lean. All 15 have a local PDF with a %PDF- signature as of 2026-11-18.
- **Arms.** U2 (free-form concept/relation labels), S2 (generic scholarly kinds and predicates), P2 (S2 plus a versioned PL concept dictionary, aliases, definitions, domain/range rules, negative examples). All produce evidence-bearing assertions; all model output stays unreviewed; a separate P2-reviewed condition measures the intended human-in-the-loop system.
- **Frozen inputs.** Parsed text, chunk ids, exact model snapshot, decoding, input order, two-pass call budget, outer JSON envelope, evidence requirements, retry policy, deterministic quote/page validation, and conservative resolution policy.
- **Blinding.** Development refines the manual and validators only. Manual, prompts, schemas, chunker, model, and gates freeze before test predictions are opened.
- **Annotation.** Two PL-knowledgeable annotators independently annotate six of the 15 papers, covering every domain and both splits. Adjudication starts only after agreement metrics are recorded. The other nine get one annotation plus a second review of every high-risk relation and a random 20% of claims/evidence.
- **Questions.** 20 downstream questions: 10 deterministic graph/fact, 6 cross-paper synthesis, 4 deliberately unanswerable. Each gold question names acceptable answers, required evidence, forbidden inferences, and an abstention rule.

## The axis is not fully constant

The plan says ontology constraint is the only manipulated variable and resolution is held constant. The arm descriptions say otherwise: U2 uses exact normalized-name merge, S2 uses exact known-alias rules, and P2 adds curated PL aliases as its only additional input. Those are different resolvers.

Before running, either:

1. share one resolver across all arms, with the alias table as the only difference; or
2. state the axis as "vocabulary plus curated aliases" and add an arm that separates vocabulary from resolution.

Do not cite the frozen-input list as proof of a constant resolution policy while the arm table describes different merge rules.

## Metric panel

No single aggregate certifies the graph. The headline is a panel:

1. **Structural and ontology conformance.** First-pass schema-valid rate; eventual structural completion after at most one retry; full ontology conformance; competency coverage; hedge rate.
2. **Mentions, concepts, resolution.** Exact and relaxed mention P/R/F1; canonical link accuracy and recall; role macro-F1 for MENTIONS, USES, own-contribution; pairwise merge P/R/F1; B-cubed P/R/F1; candidate-generation recall separately from adjudication precision; one false merge reported as one cluster-level incident.
3. **Relations.** Existence F1; semantic relation F1; strict evidenced relation F1; direction accuracy and confusion matrix for GENERALIZES/EXTENDS; high-risk spontaneous-edge violation rate; risk-coverage curve and matched-coverage F1. Abstentions and discarded hard cases remain false negatives.
4. **Claims.** Atomicity; normalized correctness 0/1/2; claim precision and recall; qualifier exact accuracy; polarity/attribution accuracy; n-ary experimental claims require all roles.
5. **Evidence.** Quote validity (exact substring of the cited raw page/chunk); locator validity; provenance completeness; evidence dedup. Human whole-evidence-set rubric: 0 unrelated/contradictory, 1 partial (omits a premise, qualifier, attribution, or required context), 2 sufficient from the cited source alone. Canonical claims/relations require 2.
6. **Downstream answers.** Answer score 0/1/2; citation precision and recall; abstention on four unsupported questions and wrong abstentions on answerable ones; MRR/nDCG only as secondary, from deterministic gold relevance.
7. **Cost, latency, human burden.** Model snapshot, dated price, input/output/cached tokens, calls, p50/p95 wall clock, queue time, retries, empty responses, truncation; cost per paper, per accepted assertion, per fully evidenced claim; review seconds per record and minutes per paper.
8. **Agreement.** Exact-span F1 and B-cubed/pairwise agreement for concepts; Cohen's kappa for roles and citation functions; Krippendorff's ordinal alpha for evidence and answer scores; relation F1 and disagreement categories. Report before adjudication.

The panel is the default. Strict evidenced relation F1 remains a useful diagnostic and may be the headline only when gold is complete, one relation type dominates, and abstention cannot be rewarded. A high aggregate driven by common MENTIONS or CITES is not a win.

## Gates

These are pre-registered project choices, not community benchmarks. Representative gates:

| Gate | Criterion |
|---|---|
| Annotation readiness | Mention F1 >= .80, relation F1 >= .80, role/citation kappa >= .75, evidence alpha >= .80; alpha < .67 stops the round |
| Structural | First-pass validity >= .98; after one retry 100% valid staged output or explicit failure; zero silent errors |
| Canonical provenance | 100% valid locators/quotes and human evidence score 2; zero fabricated locators |
| High-risk semantics | Own-contribution/extension/generalization/refutation precision >= .95; pairwise merge precision >= .98 |
| Extraction | P2 canonical concept F1 and per-relation macro-F1 both exceed S2; strict evidenced micro-F1 >= .70 and >= .05 above S2; B-cubed >= .85 |
| Claims | Precision >= .85; sufficient-evidence rate >= .95; qualifier exact accuracy >= .90 |
| Downstream | Reviewed P2 mean answer >= 1.50/2 and >= .25 above metadata hybrid; citation precision >= .90; at least 3/4 unsupported questions abstain |
| Seeded failures | 100% high-risk seeds and >= .90 overall; any mention->contribution, citation->extension, related->generalization, or incomplete-evidence approval fails |
| Operational | Total hosted spend <= US$25; P2 cost per accepted claim <= 2x S2; p95 <= 20 minutes/paper |
| Human burden | Median review <= 20 minutes/paper in the final reviewed arm |

Small-sample results with unstable paired-bootstrap intervals are inconclusive, not validated wins. Gates are not relaxed after seeing output.

## Integrity protocol

An eval is evidence only when:

1. **Execution is proven.** The run reports executed, skipped, and failed counts; executed must equal corpus size; any skip exits non-zero; a zero-case run is a harness failure, not a score.
2. **The run is traceable.** One committed record per execution: commit, model id, extra body/config, repeat index, per-case expected vs actual, raw output files, token usage, latency, and error class. A score that cannot be traced to a run is not a score.
3. **Gold is intact.** Expected answers never move to match a model. A failing case gets exactly one label: skill finding, ambiguous question, or out-of-date case; only the last two permit editing, with a cited deciding sentence and a second reviewer.
4. **The question and schema are part of the oracle.** Lint questions for outcome-presupposing wording; lint schema fields for conflating two decisions. A no-expected-answer-changed diff is insufficient.
5. **The accounting is validated.** Execution accounting is a self-report. Validate it against an independent oracle (a known-case fixture, a second runner, or a manual count) before a green run counts.
6. **There is a stop rule.** If a prompt or definition tweak fixes some cases and breaks others, revert and stop. Further iteration is tuning the test to the model.
7. **The gate ledger is re-executed.** A row that cites a command which no longer exists is stale, not green. The current ledger still cites "uv run pytest: 5 passed" after the Rust port; it must be re-run or marked no.

## Evaluator validation

Three layers, cheapest first:

1. **Invariants over generated programs.** certain is a subset of possible; answering one more atom never grows the gap; answering every demanded atom makes the bounds equal; results are independent of rule and fact order; PASS implies every error constraint has a certain goal.
2. **A naive reference for the deterministic subset.** The obvious implementation must agree with the production evaluator on derived relations.
3. **A three-valued reference for the rest.** The doubling transformation through the naive evaluator, differentially tested against the direct evaluator.

Map failures correctly: timeout, budget exhaustion, parse/backend failure, and unasked atoms go to BLOCKED or unknown, never PASS. In doc-verify, the review's blocking bug was exactly this: kernel budget exhaustion returned PASS instead of BLOCKED.

This pattern is transferred from doc-verify and has not been applied to the paper pipeline. The 24-document corpus does not exercise prefix demand or the refusals; the property suite is nightly (~1,006 s).

## Current state and next pilot

The plan exists, the corpus is admitted, and the gate ledger is honest. What is missing: independent plan review, PDF parse smoke test, annotation manual, gold, agreement, questions, seeded defects, model/provider freeze, the three arms, the deterministic scorer, blind review, and the verdict.

Minimal next pilot:

1. Parse the four development PDFs; record page mapping, reading order, math, and exact quote recovery. Do not clean failures away.
2. Ask and lint the 20 competency questions before freezing the claim schema.
3. Pilot annotation on two development papers; measure agreement; revise the manual.
4. Freeze the resolver and the axis; fix or document the U2/S2/P2 alias difference.
5. Run U2/S2/P2 on the four development papers only; at most one prompt and one chunk revision, recorded.
6. Implement the deterministic scorer and the seeded-defect suite before any test run.
7. Only then open the 11 test papers. Report the panel, the raw failures, cost, review burden, and any inconclusive gate.

## What a paper can claim from this

The defensible contribution is a **pre-registered protocol for ontology specificity plus an evidence-bearing claim model**, illustrated by a real Zotero/HawDB proof of work. It is not a demonstrated extraction improvement, not a validated ontology, and not a downstream QA result. Those require the pilot and the held-out run.
