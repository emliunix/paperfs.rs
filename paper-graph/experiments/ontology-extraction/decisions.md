# Decision log

| Timestamp | Decision | Why | Evidence | Result |
|---|---|---|---|---|
| 2026-11-18 | Use a separate derived HawDB store for round 1 | The current metadata rebuild deletes every graph node | `store.rebuild()` contains `MATCH (n) DETACH DELETE n`; research report §1.1 | Domain extraction cannot be erased by a metadata sync |
| 2026-11-18 | Replace `60`, `47`, `5`, `8` in the runnable corpus | Their attachment rows did not resolve to existing local PDFs | Direct filesystem checks under `~/Zotero/storage` | Replaced by `3229`, `1483`, `1348`, `637`; revised corpus 15/15 PDF-signature-valid |
| 2026-11-18 | Make ontology specificity the round-1 axis | A four-arm pipeline comparison would confound schema, completeness, and resolution contributions | Round-1 plan freezes a two-pass pipeline across `U2`/`S2`/`P2` | `P2` versus `S2` isolates the domain vocabulary contribution more directly |
| 2026-11-18 | Use a metric panel rather than strict evidenced relation F1 alone | F1 can hide selective abstention, rare direction errors, and merge blast radius | DeepSeek critique §4; research report §7 | Per-relation, high-risk, coverage, evidence, QA, and review metrics are co-headline |
| 2026-11-18 | Keep simple topic searches as non-regression controls | Existing hybrid search already ranks a sensible work first for all four probes | Four real `/api/search` calls recorded in `gates.md` | Primary downstream gold shifts to compositional, evidence-required questions |
| 2026-11-18 | Treat DeepSeek as critique, not sourced evidence | Its suggestions are uncited model output and include potentially over-broad ontology additions | Successful `dsh headless` response saved separately | Adopt metric cautions; defer wholesale node/relation expansion |
