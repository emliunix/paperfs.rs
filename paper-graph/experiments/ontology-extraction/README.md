# Domain ontology and graph extraction

**Status:** round 1 planned; PDF source admission complete; gold annotation and
execution not started.

## Question

Does increasing ontology constraint—from unconstrained concept labels, through
a generic scholarly schema, to a curated PL/type-theory ontology—improve
evidence-bearing extraction and compositional question answering when the
model, chunks, prompt envelope, call budget, and resolution policy are held
constant?

## Files

- `grounding.md`: claims and falsifiers.
- `round-001-plan.md`: three-arm pre-registration and protocol.
- `round-001-corpus.json`: 15 works, four development and eleven test.
- `round-001-rubric.md`: extraction, evidence, downstream, cost, and agreement
  scoring.
- `gates.md`: completed and incomplete checks.
- `decisions.md`: corpus and metric decisions with evidence.

Research inputs:

- `../../analysis/llm-document-graph-research.md`: sourced primary-literature
  and official-tool survey.
- `../../analysis/deepseek-ontology-critique.md`: secondary design critique,
  used as criticism rather than evidence.

## Not created yet, and why

- `round-001-review.md`: an independent seat must review the frozen plan before
  any model extraction run.
- `annotation-manual.md` and `gold/`: require human PL annotation and agreement
  measurement; no model-generated output will be promoted to gold silently.
- `questions.json`: candidate questions exist in the plan, but required answers,
  evidence spans, and abstentions are not reviewed gold yet.
- `seeds.json`: the rubric names the required cases; executable fixtures are not
  implemented.
- `results/` and `round-001-verdict.md`: no extraction arm has run.
- Extraction/scoring harness: only corpus-structure tests exist at this stage.

The current metadata database remains untouched. Round 1 must use a separate
derived HawDB database because `store.rebuild()` currently deletes every graph
node.
