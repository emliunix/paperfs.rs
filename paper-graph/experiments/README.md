# Paper-graph experiments

Each experiment question owns a directory. A round's plan is written before its
run; predictions are not rewritten as if they had been pre-registered.

Conventions for this new tree:

- `README.md`: programme status, file inventory, and named missing stages.
- `grounding.md`: claims under test and their falsifiers.
- `gates.md`: honest completion ledger; a skip is not a pass.
- `decisions.md`: one row per landed design decision, with evidence.
- `round-N-plan.md`: hypothesis, axis, candidates, predictions, and run protocol.
- `round-N-corpus.json`: paper IDs, splits, and source-admission status.
- `round-N-rubric.md`: exact metrics and human scoring rules.
- `round-N-review.md`: independent plan review, required before execution.
- `round-N-verdict.md`: final result, written only after all scored tracks end.
- `results/`: raw and derived run outputs, when a run exists.

## Programmes

- `ontology-extraction/`: whether a domain-specific PL/type-theory ontology
  improves evidence-bearing graph extraction and downstream answers.

No experiment round has executed. There are no results, independent plan
reviews, or verdicts in this tree yet.
