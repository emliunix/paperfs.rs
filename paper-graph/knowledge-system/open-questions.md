# Open questions and next experiments

## The six gaps that matter

### 1. The extraction round has never run

The gate ledger marks independent plan review, PDF parse, annotation manual, gold, agreement, questions, seeded defects, model freeze, U2/S2/P2, scorer, blind scoring, and verdict all "no". There is no Concept or Claim node in src/.

**Why it matters.** The corpus can say the design is internally coherent; it cannot say whether the curated ontology beats a generic one, whether the metric panel beats F1, or whether 15 papers suffice. It is one library and one model family.

**Next.** Run the four-development-paper pilot: parse, annotate, U2/S2/P2 extraction, pre-registered panel, two annotators. Keep the 11 test papers closed and record deviations before opening test predictions.

### 2. There is no parsed-text evidence substrate

The pipeline indexes metadata and abstracts only. No PDF parser exists; no chunk/page/offset table; gates.md marks the PDF parse smoke test "no".

**Why it matters.** The exact-quote-plus-locator rule and the evidence coordinates cannot be implemented against text the system never parsed. PL papers put their hardest content in notation, theorems, tables, and two-column layouts, which an abstract-only pipeline never sees.

**Next.** Parse all 15 PDFs; add a chunk/page/offset table; measure quote recovery and locator validity per domain, including theorem/table/code cases; fail the round below the 100% locator gate.

### 3. The evaluator has no validated oracle

There is no deterministic scorer, no seeded defects, no gold agreement, and no downstream questions. The only independent reference implementation in the corpus belongs to doc-verify, a different system. The flow-skills incident shows a zero-case run can report green; the home.conf review shows question edits can leak answers.

**Why it matters.** Even after an arm runs, a green score could be vacuous, leak answers, or reward abstention. The corpus cannot distinguish a correct extraction from a lucky one. "The question is part of the oracle" has no paper-system implementation.

**Next.** Implement the deterministic scorer and the seeded-defect suite (mention-to-contribution, fabricated quote, false merge, unsupported claim). Measure false-pass/false-fail on a two-annotator subset. Lint the competency questions before answers.

### 4. Stable identity is asserted but untested across re-import

item_id is the SQLite primary key; zotero_key is a non-key column. No test exercises a Zotero merge, restore, or duplicate that changes itemIDs. The pre-registered corpus stores only item_id, split, domain, and title, with no zotero_key or DOI.

**Why it matters.** A merge or restore can renumber itemIDs, orphaning every claim, evidence link, and distilled note. Storing zotero_key without joining on it gives the appearance of a durable address without the guarantee. The pre-registration is anchored to an identifier the synthesis itself calls unstable.

**Next.** Clone the library, force a merge/restore that changes itemIDs, sync, and assert join/evidence survival. Add an itemID-to-zotero_key mapping table or promote zotero_key to the durable join. Add zotero_key and DOI to the corpus entries and assert the mapping in the corpus test.

### 5. Derived-state isolation is a convention, not a guarantee

store.rebuild() deletes all works and runs MATCH (n) DETACH DELETE n. The only mitigation is a decision to use a separate derived DB; no such database exists, and there is no guard or test.

**Why it matters.** One sync against the wrong database erases the extraction layer. P01, P03, and P14 all depend on this boundary.

**Next.** Add an integration test that populates a derived DB, runs a metadata sync, and asserts extraction/claim/evidence counts unchanged. Fail closed if derived data is present, or implement a non-destructive upsert.

### 6. There is no per-call provenance or failure classification

SyncReport records only database, model, URL, tokens, elapsed, and counts. Missing usage is coerced to 0. There is no run id, timestamp, attempt, finish reason, or per-batch failure.

**Why it matters.** Timeouts, truncation, empty responses, and app restarts are indistinguishable from model errors or zero cost, so the budget inequality and the metric panel cannot be audited. The diagnosis was possible only from external logs, not from the pipeline's own record.

**Next.** Add a run manifest (run id, timestamp, attempt, finish reason, usage_unreported, per-batch failure) and replay forced timeout, truncation, and restart; assert each is classified and an app restart is never scored as a model failure.

## Decisions still open

1. **Separate derived DB or scoped rebuild?** Isolation is simpler; a scoped rebuild is a better long-term design. Pick one before extraction, not after.
2. **itemID or zotero_key as the durable join?** itemID is fast and already in use; key is durable but unused. A mapping table is the conservative compromise.
3. **Closed kind enum or faceted kind?** The enum is simple but may hide taxonomy. Faceted/multi-valued/attributed is more honest but harder to score.
4. **Keep INTRODUCES or rename it?** The predicate smuggles priority and conflicts with EXTENDS/GENERALIZES. A work-scoped "claims contribution" predicate with a non-priority qualifier is safer.
5. **Which parser?** Native extraction first (PyMuPDF or equivalent); layout-aware only for two-column/formula-heavy pages; OCR only for scanned pages; LlamaParse only with explicit opt-in because it uploads paper content.
6. **One resolver or three?** The U2/S2/P2 axis is confounded unless the resolver is shared and the alias table is the only difference. Fix before running.
7. **Panel or F1 headline?** The rubric says panel; the research report still says F1. Amend the report or state the exception conditions explicitly.

## What the paper can claim now

A defensible paper can claim:

- a proof-of-work architecture for a local, evidence-first paper knowledge system over Zotero and HawDB;
- a pre-registered protocol for testing whether ontology specificity improves extraction, with a metric panel and explicit gates;
- a set of operational findings: destructive metadata rebuilds, dangling attachments, embedding batch limits, title-quality ranking failures, budget/timeout compositions, and eval-integrity failure modes;
- a negative result about the current evidence: the system is a metadata index, not yet an evidence graph.

It cannot yet claim:

- that the PL ontology improves extraction or downstream answers;
- that the metric panel is better than strict F1;
- that 15 papers are enough;
- that evidence-bearing claims improve answer quality;
- that itemID survives a Zotero restore;
- that the separate derived store works.

## The next three experiments

1. **Admission and parse pilot.** Executable PDF admission; parse the four development PDFs; measure page mapping, reading order, math, quote recovery; add a chunk/page/offset table. Fail on any missing file or fabricated locator.
2. **Ontology-specificity pilot.** Freeze the questions and the resolver; cut the vocabulary to five relations; run U2/S2/P2 on the four development papers; score the panel plus seeded defects; record cost and review minutes. Keep the 11 test papers closed.
3. **Integrity and identity drill.** Inject a zero-case run, a wrong-gold case, a question-wording leak, a Zotero re-import that changes itemIDs, and a metadata sync against a populated derived DB. Assert each is caught or blocked. These are cheap and test the rules that the corpus already supports.
