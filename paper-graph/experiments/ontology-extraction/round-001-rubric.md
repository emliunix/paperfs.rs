# Round 1 scoring rubric

No single aggregate can certify the graph. The headline is a **panel**:
per-relation quality, high-risk violation rate, coverage, evidence validity,
downstream supported answers, and review burden. Strict evidenced relation F1
is retained as one diagnostic, not as the sole criterion.

## 1. Structural and ontology conformance

- **First-pass schema-valid rate:** calls returning valid JSON matching the
  outer schema without repair ÷ all calls.
- **Eventual structural completion:** calls with valid staged output or an
  explicit logged failure after at most one structural retry ÷ all calls.
  Empty content and truncation are failures, never empty gold graphs.
- **Full ontology conformance:** emitted assertions with allowed predicate,
  valid endpoint kinds, correct direction, and resolved local references ÷ all
  emitted assertions.
- **Competency coverage:** pre-registered questions answerable from approved
  graph facts and evidence ÷ all competency questions.
- **Hedge rate:** `RELATED_TO` or unresolved outputs emitted where gold has a
  stronger relation ÷ all stronger-relation predictions.

## 2. Mentions, concepts, and resolution

- **Exact mention P/R/F1:** exact source span and correct coarse kind.
- **Relaxed mention F1:** at least 50% span overlap and correct kind; secondary.
- **Canonical link accuracy and recall:** correct canonical concept links ÷ all
  predicted links, and recovered gold links ÷ all gold links. Unresolved remains
  visible and reduces recall.
- **Role macro-F1:** `MENTIONS`, `USES`, and own-contribution scored separately.
- **Pairwise merge P/R/F1:** predicted same-concept pairs against gold pairs.
- **B-cubed P/R/F1:** mention-weighted predicted versus gold concept clusters.
- Candidate-generation recall is reported separately from adjudication
  precision. One false merge is also reported as one cluster-level incident so
  its downstream blast radius is visible.

## 3. Relations — factorized and strict

Report per predicate and macro averages, with exact counts:

1. **Existence F1:** correct subject/object identities, ignoring predicate and
   evidence for diagnosis only.
2. **Semantic relation F1:** subject + predicate + object + direction correct.
3. **Strict evidenced relation F1:** the semantic tuple and the complete
   evidence set are correct and sufficient. There is no half credit.
4. **Direction accuracy and confusion matrix:** mandatory for `GENERALIZES`
   and `EXTENDS_*`.
5. **High-risk spontaneous-edge violation rate:** unsupported own-contribution,
   extension, generalization, or refutation edges ÷ all predicted high-risk
   edges, plus their absolute count relative to gold.
6. **Risk–coverage curve and matched-coverage F1:** abstentions and discarded
   hard cases remain false negatives. No arm is rewarded simply for emitting
   fewer difficult assertions.

The headline comparison is `P2` versus `S2` on canonical concept F1 and
per-relation macro-F1, together with the high-risk violation gate. A high
aggregate driven by common `MENTIONS` or `CITES` is not a win.

## 4. Claims

- **Atomicity:** 0 = independently evaluable statements are combined or an
  indivisible condition is split away; 1 = atomic with its necessary condition.
- **Normalized correctness:** 0 = wrong/unsupported; 1 = partially correct or
  missing qualifier/attribution; 2 = exact meaning, polarity, scope, qualifier,
  and attribution.
- **Claim precision:** atomic claims scoring 2 ÷ all predicted claims.
- **Claim recall:** recovered gold atomic claims in dense zones ÷ all gold
  claims in those zones.
- **Qualifier exact accuracy** and **polarity/attribution accuracy:** separate,
  with confusion matrices.
- N-ary experimental claims require all roles; they are not decomposed into
  easy independent binary true positives.

Version 0 may decline table/figure/proof-only evidence rather than pretending a
nearby prose quote is sufficient. Such declines remain visible in coverage.

## 5. Evidence and provenance

Deterministic checks:

- **Quote validity:** exact quote occurs in the cited raw parser page/chunk.
  Normalized matching may queue review but cannot auto-approve.
- **Locator validity:** attachment/chunk/page/offset resolves exactly.
- **Provenance completeness:** assertions/claims with required evidence objects
  ÷ all assertions/claims.
- **Evidence deduplication:** overlapping copies of one span are one support,
  not independent corroboration.

Human whole-evidence-set rubric:

- **0 — unrelated or contradictory.**
- **1 — partial:** omits a premise, qualifier, attribution, or necessary source
  context, or needs unstated external knowledge.
- **2 — sufficient:** a PL-knowledgeable reader can verify every element from
  the cited source alone.

Canonical claims/relations require 2. An average cannot hide a high-risk 0/1.
LLM entailment may triage review but cannot replace this score.

## 6. Downstream answers

- **Answer score:** 0 = wrong/unsupported; 1 = partially correct/incomplete;
  2 = correct and complete for the question.
- **Citation precision:** cited evidence supporting its attached answer
  component ÷ all cited evidence.
- **Citation recall:** required gold evidence components cited ÷ all required
  components.
- **Abstention:** correct refusals on four unsupported questions, and wrong
  abstentions on answerable questions, both reported.
- **MRR and nDCG@10:** secondary, computed only from deterministic gold
  relevance. The four simple exploratory queries are non-regression controls.

Reviewers are blind to arm and order. An LLM judge is optional secondary
agreement data, never the sole grade.

## 7. Cost, latency, and human burden

Record provider/model snapshot, dated price, input/output/cached tokens, calls,
wall-clock p50/p95, queue time, structural retries, empty responses, and
truncation. Report cost per paper, per accepted assertion, and per fully
evidenced accepted claim. Local runs report hardware/runtime rather than an
invented dollar price. Human review seconds per record and minutes per paper
are first-class results.

## 8. Agreement

Before adjudication report:

- exact-span F1 and B-cubed/pairwise agreement for concept annotation;
- Cohen's kappa plus raw agreement/prevalence for roles and citation functions;
- Krippendorff's ordinal alpha for evidence and answer scores;
- relation F1 and disagreement categories.

Adjudicated gold is created only after these values are stored.

## 9. Engineering gates for this small round

These are pre-registered project choices, not external standards.

| Gate | Criterion |
|---|---|
| Annotation readiness | Mention F1 ≥ .80, relation F1 ≥ .80, role/citation κ ≥ .75, evidence α ≥ .80; α < .67 stops the round |
| Structural | First-pass validity ≥ .98; after one retry 100% valid staged output or explicit failure; zero silent errors |
| Canonical provenance | 100% valid locators/quotes and human evidence score 2; zero fabricated locators |
| High-risk semantics | Own-contribution/extension/generalization/refutation precision ≥ .95; pairwise merge precision ≥ .98 |
| Extraction | `P2` canonical concept F1 and per-relation macro-F1 both exceed `S2`; strict evidenced relation micro-F1 ≥ .70 and ≥ .05 above `S2`; B-cubed F1 ≥ .85 |
| Claims | Precision ≥ .85, sufficient-evidence rate ≥ .95, qualifier exact accuracy ≥ .90 |
| Downstream | Reviewed `P2` mean answer ≥ 1.50/2 and ≥ .25 above metadata hybrid; citation precision ≥ .90; at least 3/4 unsupported questions abstain |
| Seeded failures | 100% high-risk seeds and ≥ .90 overall; any mention→contribution, citation→extension, related→generalization, or incomplete-evidence approval fails |
| Operational | Total hosted spend ≤ US$25; `P2` cost per accepted claim ≤ 2× `S2`; p95 ≤ 20 minutes/paper |
| Human burden | Median review ≤ 20 minutes/paper in the final reviewed arm |

Small-sample results with unstable paired-bootstrap intervals are
**inconclusive**, not validated wins. Gates are not relaxed after seeing output.

## 10. Required seeded defects

The executable seed suite must cover:

- contrast/mention versus actual use;
- another work's introduction versus the current work's own contribution;
- introducing a specific algorithm versus inventing a whole technique;
- relatedness/encoding versus subclass or generalization;
- free monads versus freer monads;
- ordinary effects versus algebraic effects;
- citation/notation/baseline versus extension;
- reversed generalization and extension direction;
- conservative extension as a claim, not the token `EXTENDS_WORK`;
- a conditional or negated claim;
- a theorem conclusion missing a positivity/termination premise;
- conjunctive claims with different evidence requirements;
- bibliography-only concept occurrence;
- garbled mathematical notation;
- duplicate evidence in overlapping chunks;
- empty/truncated model output;
- ambiguous citation target, which must remain unresolved;
- tool or benchmark version drift;
- a table/proof-only claim that the prose-only evidence policy must decline.
