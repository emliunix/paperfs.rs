# Round 1 plan — ontology specificity

**Status:** pre-registered design, not independently reviewed and not run.

## Hypothesis

A curated PL/type-theory ontology improves canonical concept linking,
per-relation extraction, and evidence-supported compositional answers over a
generic scholarly ontology and unconstrained labels on the same document
inputs.

The named refuting cell is the PL arm failing to beat the generic arm on both
canonical concept F1 and per-relation macro-F1 on held-out papers, or failing a
high-risk evidence/semantic gate despite a higher aggregate F1.

## One axis

**Ontology constraint.** Everything else is held constant: parsed text, chunk
IDs, exact model snapshot, decoding, input order, two-pass call budget, outer
JSON envelope, evidence requirements, retry policy, deterministic quote/page
validation, and conservative resolution policy.

## Candidates

| Arm | Concept and predicate vocabulary | Resolution |
|---|---|---|
| `U2` | Free-form concept categories and relation labels in the same typed outer extraction envelope | Exact normalized-name merge only |
| `S2` | Generic scholarly kinds and predicates: technique, formal system, feature, object, property, proof method, representation; `MENTIONS`, `USES`, own-contribution, extension, generalization, dependency | Same two passes and exact known-alias rules; no PL vocabulary |
| `P2` | `S2` plus the versioned PL concept dictionary, aliases, definitions, relation domain/range rules, and negative examples | Same algorithm and budget; curated PL aliases are the only additional input |

All arms produce evidence-bearing assertions. All model-produced records remain
`unreviewed`; a separate `P2-reviewed` condition measures the intended
human-in-the-loop system. No reviewed output is compared with unreviewed output
to claim a model advantage.

The research report's four-arm `G1`/`G2`/`T1`/`T2` design remains a useful
alternative for a later pipeline-stage round. This round instead holds the
two-pass pipeline constant so ontology specificity is the manipulated variable.

## Predictions recorded before extraction

- `U2` preserves many surface mentions but has the most duplicate concept
  labels and invalid/ambiguous relation types.
- `S2` improves structural and domain/range conformance but still fragments PL
  concepts across aliases and related-but-distinct formal notions.
- `P2` has the best canonical concept F1, per-relation macro-F1, and downstream
  evidence-supported answer score.
- `P2` may have lower raw mention recall because its vocabulary is closed;
  unresolved records count against recall rather than disappearing.
- Simple topic-query nDCG may barely change because the existing metadata
  search is already strong. The primary downstream prediction concerns
  compositional answers and citations, not simple topic retrieval.

## Corpus and blinding

Use `round-001-corpus.json`: four development papers, one per domain, and
eleven held-out test papers. Development work refines the annotation manual and
validators only. Freeze the manual, prompts, schemas, chunker, model, and gates
before test predictions are opened.

Two PL-knowledgeable annotators independently annotate six of the 15 papers,
covering every domain and both splits. Adjudication starts only after agreement
metrics are recorded. The other nine receive one annotation plus a second
review of every high-risk relation and a random 20% of claims/evidence.

Dense annotation zones are fixed before model output: abstract/introduction,
core definitions/method, and theorem/evaluation sections. Whole-document recall
is claimed only for separately annotated salient concepts and relations.

## Execution order

1. Independent plan review; resolve or explicitly disposition every finding.
2. Parse four development PDFs locally. Check page mapping, two-column order,
   mathematical notation, and exact quote recovery. Do not clean failures away.
3. Pilot annotation on two development papers; measure agreement and revise the
   shared human/model annotation manual.
4. Complete gold and 20 downstream questions; freeze all hashes and splits.
5. Run the three arms on development only. Permit at most one prompt revision
   and one chunk revision, both recorded.
6. Run each arm once on the eleven test papers with identical source artifacts
   and call budgets. Preserve raw output, errors, usage, and latency.
7. Run deterministic scorers before unblinding semantic outputs. Blind human
   review to arm and randomize record order.
8. Score seeded defects, provenance, extraction, downstream answers, cost, and
   review burden. Write the verdict only after every track has ended.

## Downstream question design

Create 20 questions: ten deterministic graph/fact questions, six cross-paper
synthesis questions, and four deliberately unanswerable questions. Every gold
question names acceptable answers, required evidence, forbidden inferences, and
an abstention rule.

The four exploratory topic queries in `gates.md` are non-regression controls.
Primary questions must require distinctions such as:

- Does a work use a technique in its own method or merely discuss it?
- Does a citation supply background, a copied notation, a baseline, or an
  explicit extension?
- Which direction does an explicit generalization run?
- Which premises qualify a theorem or design claim?
- Which two concepts are aliases, and which are only related?
- What exact source span supports a cross-paper comparison?

## HawDB and integration boundary

Round 1 uses a separate derived database. The current metadata
`store.rebuild()` executes `MATCH (n) DETACH DELETE n`; extracted domain data
must not be loaded into that graph until the rebuild is narrowed or extraction
is included in the same rebuild transaction.

## Stop conditions

Stop before hosted extraction if the independent review rejects the plan, gold
agreement fails, source text cannot support exact evidence, or the run manifest
cannot enforce the chosen spend cap. Numeric engineering thresholds are in
`round-001-rubric.md`; they are not community benchmarks.
