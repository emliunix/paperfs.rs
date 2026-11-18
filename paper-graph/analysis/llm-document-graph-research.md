# LLM document-to-entity-graph extraction for the PL paper graph

**Research date:** 2026-11-18  
**Status:** research and experiment design only; no extraction pipeline was implemented.  
**Parent update:** the runnable experiment corpus is now the 15-PDF set in `../experiments/ontology-extraction/round-001-corpus.json`, not the initial 12-work proposal in §7.1. A subsequent DeepSeek retry succeeded; see §9 and `deepseek-ontology-critique.md`.  
**Evidence convention:** **Verified** means checked against a primary paper or official project documentation fetched on the research date. **Recommendation** means a design choice for this library, not a result established by those sources. Undated web documentation is explicitly described as undated; no current software version is inferred from a copyright notice or repository landing page.

## Executive recommendation

Build a **small, schema-first, evidence-first extraction layer**, not a general-purpose GraphRAG clone. The most useful combination for this project is:

1. local PDF/TEI parsing with page-preserving text and bibliography parsing (GROBID is a candidate, subject to a smoke test on PL notation);
2. section-aware chunks and strict JSON Schema/Pydantic outputs;
3. a deliberately small PL ontology with `INTRODUCES`, `USES`, and `MENTIONS` kept distinct, and no generic subclass hierarchy;
4. an atomic `Claim` node with one or more exact evidence spans, qualifiers, polarity, and review state;
5. deterministic quote/location/domain/range validation before any LLM-based adjudication;
6. conservative cross-chunk and cross-document entity resolution that preserves aliases and all source links and never merges merely related concepts;
7. human approval for high-impact assertions before they enter the canonical HawDB graph;
8. content-hash invalidation of attachment-derived artifacts while retaining the Zotero numeric `itemID` as the Work identity.

GraphRAG is valuable as a baseline and for its chunk/gleaning/provenance ideas, but its default exact title/type merge, generic relation descriptions, exact-string experiments, and community-summary objective do not directly solve typed PL concept resolution or claim evidence. LangChain and LlamaIndex are reasonable adapter implementations, not quality guarantees. KGGen supplies the most directly relevant recent cross-source entity/relation clustering design, but its published evaluation does not establish gold-standard merge quality or a complete provenance model.

The recommended first experiment uses **12 real works from this Zotero library**, four development and eight held-out test works, four pre-registered extraction arms, deterministic provenance checks, two human annotators on six papers, and 20 downstream questions. The headline metric is **strict, fully evidenced relation F1**, alongside high-risk relation precision, claim evidence sufficiency, false merges, downstream answer/citation quality, and cost per accepted claim.

---

## 1. Local starting point and constraints

### 1.1 What exists now

The project README and source establish the following:

- The library has **324 works**, with 117 abstracts, 347 people, five collections, 24 tags, ten venues, and 161 attachments after the 2026-11-17 sync.
- `Work.id` is Zotero's numeric `itemID`; `Work.key` is the Zotero key. No content-hash Work identity is used.
- `Attachment.id` is also a Zotero numeric item ID, with `parent_item_id`, path, and content type.
- HawDB 0.34 stores a SQL `works` table and graph nodes `Work`, `Person`, `Collection`, `Tag`, `Venue`, and `Attachment`.
- Search is BM25, exact 1024-dimensional vector search, or reciprocal-rank fusion over metadata and abstracts. There is no PDF text, citation, OCR, or claim extraction yet.
- A sync is a full derived-data rebuild, not an incremental indexer.

A crucial integration detail is that `store.rebuild()` currently executes `MATCH (n) DETACH DELETE n`. Loading a domain graph into the same graph and then running today's metadata sync would delete it. The first experiment should therefore use a **separate derived HawDB database or a separate staging store**. A later integration must narrow the rebuild to the metadata subgraph or make domain extraction part of the same transaction.

Relevant local files:

- `/Users/emliunix/Documents/paper-graph/README.md`
- `/Users/emliunix/Documents/paper-graph/src/paper_graph/store.py`
- `/Users/emliunix/Documents/paper-graph/src/paper_graph/zotero.py`

### 1.2 Privacy and operational constraints

**Recommendation:** keep PDFs and extracted text on local routes by default. Do not upload full papers to LlamaParse or a hosted parser without explicit opt-in. A hosted LLM extraction route, if later approved, is a separate disclosure decision from local PDF parsing.

A local operational note supplied during this session records a prior Nowledge KG extraction failure with local `gemma4-12b`: reasoning consumed the structured-output budget (`reasoning_only=true`, `content_len=0`). The successful local configuration used thinking off, timeout 600 seconds, `max_tokens=32000`, and llama.cpp context size 40960. This is **local operational evidence, not a universal requirement for DeepSeek or every model**. The extraction harness should nevertheless always check empty content, truncation/finish reason, and schema validity, and should reserve output budget separately where the serving stack permits it.

---

## 2. What the primary research and official tools establish

### 2.1 Comparison at a glance

| Approach | Verified contribution | Important limitation for this project | Recommended role |
|---|---|---|---|
| Microsoft GraphRAG | LLM entity/relation/optional claim extraction, TextUnit provenance, gleanings, graph communities and summaries; incremental-output metadata | Generic schema, default exact title/type merge, claims off by default, community summaries address corpus-wide sensemaking rather than typed scientific claims | Generic baseline; reuse TextUnit and gleaning ideas, not the whole default product |
| LangChain `LLMGraphTransformer` | Node/relation allowlists, optional directed triples, strict filtering, structured output/function calling, optional properties | No documented automatic source-Document/provenance edge; filtering does not establish semantic correctness | Thin implementation adapter if LangChain is already desired |
| LlamaIndex `SchemaLLMPathExtractor` | Pydantic/structured-output path validation, entity/relation/valid-combination schema, strict mode, ingestion and graph stores | Per-chunk extraction still needs custom evidence, coreference, and conservative canonicalization; default `raise_on_error=False` can hide failures | Strongest of the two framework adapters for a typed prototype, with `raise_on_error=True` |
| SPIRES/OntoGPT | Detailed user-defined nested schemas, recursive extraction, ontology/vocabulary grounding; demonstrated in biomedical and other domains | Not evidence of PL extraction quality or complete claim provenance; human-curation assistance remains necessary | Design reference for schema-generated prompts and grounding, not a required dependency |
| Text2KGBench | Ontology-driven extraction task with fact P/R/F1, ontology conformance, and subject/relation/object hallucination metrics | Original conformance metric checks canonical relation names, not full domain/range validity; sentence-level rather than full PL papers | Metric template; strengthen conformance and evidence requirements locally |
| KGGen | Two-stage entity then triple extraction, cross-source aggregation, hybrid candidate retrieval, iterative LLM entity/relation clustering, relation-to-chunk links | Over-/under-merging remains; no gold pairwise/cluster resolution benchmark; complete merged lineage is not documented | Best recent model for conservative candidate generation plus adjudication; add stronger review and provenance |
| SciREX | Full-document scientific mentions, coreference, salient entities, binary and N-ary relations; shows severe end-to-end degradation | ML/Papers-with-Code schema is not a PL ontology; annotation excludes some non-salient repairs and equations/figures | Evaluation design warning: score salience, document-level coreference, and end-to-end relations separately |
| SciFact | Expert-written scientific claims with SUPPORTS/REFUTES and evidence rationales | Verification of given claims over abstracts, not automatic claim extraction from full papers | Claim/evidence annotation model; do not cite it as an extractor |
| GROBID | Maintained scholarly PDF-to-TEI structuring, references, citation contexts, consolidation, PDF coordinates | Mathematical PL notation and two-column layouts still require a local quality check | Candidate local parser/reference extractor, complementing rather than replacing LLM IE |
| GoLLIE | Fine-tuning to follow detailed annotation guidelines improved unseen-task zero-shot IE | Not a prompt-only recipe and not evaluated on this PL ontology | Evidence that precise guidelines and examples matter; do not assume zero-shot instructions are enough |

### 2.2 GraphRAG: useful mechanics, different objective

**Verified.** The GraphRAG paper, *From Local to Global: A Graph RAG Approach to Query-Focused Summarization*, was submitted 2024-04-24 and revised 2025-02-19 ([arXiv:2404.16130](https://arxiv.org/abs/2404.16130), [full text](https://arxiv.org/html/2404.16130)). Its evaluation used 600-token chunks with 100-token overlap. Its multipart prompt extracts entity name/type/description and source/target/relationship-description/strength. A separate optional prompt extracts claims/covariates. Gleaning asks whether entities were missed and permits additional extraction. Repeated descriptions are summarized; repeated relationships are merged and counts become weights. Hierarchical Leiden communities and their summaries support global query-focused summarization.

The paper's experiments used exact-string entity matching and acknowledge duplicate entities. Its evaluation compared global community levels with source-text summarization and semantic search on podcast and news corpora, using LLM pairwise judgments and claim-based comprehensiveness/diversity measures. It did not evaluate typed PL extraction, exact quote faithfulness, or gold cross-chunk entity resolution. Podcast indexing took 281 minutes under the reported setup; this is not a portable latency estimate for this library.

**Verified from current official docs, undated pages fetched 2026-11-18.** The [default dataflow](https://microsoft.github.io/graphrag/index/default_dataflow/) now documents a configurable default TextUnit size of 1,200 tokens, distinct from the paper's experimental 600. Entities and relationships carry `text_unit_ids`; covariates carry a `text_unit_id` and `source_text` in the [outputs documentation](https://microsoft.github.io/graphrag/index/outputs/). Entities with the same title and type are merged. Claim extraction is optional, off by default, and the [configuration guide](https://microsoft.github.io/graphrag/config/yaml/) warns that claim prompts need user tuning. Community `period` and `size` support incremental-update merges, and `update_output_storage` preserves original outputs, but the fetched pages do not specify a complete conflict-resolution algorithm.

**Recommendation.** Use GraphRAG's TextUnit lineage and explicit completeness pass. Do not adopt exact title/type merging, unrestricted relationship descriptions, or community reports as the first milestone. The first downstream task is evidence-backed concept/claim querying, not global summarization.

### 2.3 LangChain and LlamaIndex property graphs

**Verified.** The official [LangChain `LLMGraphTransformer` reference](https://reference.langchain.com/python/langchain-neo4j/graph_transformers/llm/LLMGraphTransformer) documents `allowed_nodes`, `allowed_relationships` (including directed source/relation/target tuples), `strict_mode=True`, optional node and relationship properties, custom prompts, and native structured output/function calling by default. Empty allowlists allow all types. Setting properties to `True` allows arbitrary properties; a list limits their names. The page documents no automatic source-Document/provenance relationship. It is undated and links source at commit `664226bd558d8cbe7514c01a5709362f64684dd0`; its example import still refers to the experimental package, so the exact package version must be pinned when implementing.

**Verified.** The official [LlamaIndex Property Graph guide](https://developers.llamaindex.ai/python/framework/module_guides/indexing/lpg_index_guide/) documents `SimpleLLMPathExtractor`, `DynamicLLMPathExtractor`, and `SchemaLLMPathExtractor`. The schema extractor uses Pydantic and structured outputs to validate paths against allowed entities, relations, and combinations; `strict=False` permits out-of-schema triples. It supports per-chunk path limits, custom prompts/schema classes, later `insert`/`insert_nodes`, and explicit `HAS_SOURCE` links to `TextNode`s. `include_text=True` returns source chunks with retrieved paths. The guide notes that API errors are logged and swallowed by default (`raise_on_error=False`), which is unacceptable for a measured experiment unless changed.

**Recommendation.** If a framework is used, LlamaIndex's schema extractor is the closer fit. In either framework, keep raw responses and errors, set failures to stop the run, attach evidence independently, and treat schema filtering as syntactic enforcement only. Do not make “LangChain versus LlamaIndex” the experimental variable; the ontology, evidence, and resolution stages are the scientifically interesting variables.

### 2.4 Schema-constrained extraction and structured outputs

**Verified.** SPIRES was posted in 2023, revised 2023-12-22, and published in *Bioinformatics* on 2024-03-04 ([arXiv:2304.02711](https://arxiv.org/abs/2304.02711), DOI [10.1093/bioinformatics/btae104](https://doi.org/10.1093/bioinformatics/btae104)). It recursively prompts an LLM against a detailed user schema and grounds extracted values in existing ontologies/vocabularies. The abstract places accuracy around the middle of existing relation-extraction methods and presents the approach as assistance for knowledge-base population, not guaranteed autonomous correctness.

**Verified.** [Text2KGBench](https://arxiv.org/abs/2308.02357), posted 2023-08-04 and published at ISWC 2023, evaluates ontology-driven fact generation using fact P/R/F1, ontology conformance, and subject/relation/object hallucination. Its small-ontology prompts include domain/range information and similarity-selected demonstrations. Baseline fact F1 was modest even while relation-name conformance was high. Its reported conformance only checks whether a relation is canonical, so this project must additionally validate endpoints, direction, and evidence.

**Verified.** OpenAI's current [Structured Outputs guide](https://developers.openai.com/api/docs/guides/structured-outputs) describes JSON Schema output with `strict: true` and a separate function-calling path. Only completed, non-refusal output is schema-constrained; applications must handle refusal, API errors, missing content, and truncation. JSON validity/schema adherence does not guarantee factual or business correctness. The page is undated as fetched.

**Verified.** Pydantic's current [JSON Schema documentation](https://pydantic.dev/docs/validation/latest/concepts/json_schema/) documents Draft 2020-12 schema generation from `BaseModel` and `TypeAdapter`, validation versus serialization modes, shared `$defs`, and schema customization. The page is undated as fetched.

**Recommendation.** Generate the wire schema from Pydantic models where convenient, but retain three distinct layers:

1. **decoder/provider constraint:** valid supported JSON Schema;
2. **Pydantic validation:** types, required fields, enums, list lengths, local references;
3. **semantic validation:** quote equality, page/offset correctness, evidence sufficiency, domain/range, attribution, and ontology rules.

Never treat a successful function call or Pydantic parse as proof that an assertion is true. Save raw output, parsed output, validation errors, retries, exact model ID, and finish reason. Reject empty content and truncation before parsing.

### 2.5 Detailed guidelines are part of the model interface

**Verified.** GoLLIE, *Annotation Guidelines Improve Zero-Shot Information-Extraction*, was first posted 2023-10-05, revised through 2024-03-06, and published at ICLR 2024 ([arXiv:2310.03668](https://arxiv.org/abs/2310.03668)). It is **fine-tuned** to follow annotation guidelines; its ablation identifies detailed guidelines as important. It is not evidence that a prompt-only model will follow a new ontology reliably.

**Recommendation.** Write the ontology as an annotation manual first: definitions, inclusion/exclusion rules, direction, attribution, edge cases, and positive/negative examples. Use the same versioned manual for humans and the LLM prompt. Fine-tuning is out of scope for the first experiment; if prompt-only results fail, guideline ambiguity is one hypothesis to test before adding model complexity.

### 2.6 Scientific document IE: salience and document-level relations are the hard parts

**Verified.** SciREX, published at ACL 2020 ([ACL Anthology](https://aclanthology.org/2020.acl-main.670/), [full text](https://arxiv.org/html/2005.00512)), contains 438 fully annotated ML documents with Dataset, Metric, Task, and Method mentions, coreference, salience, and N-ary relations. In its corpus, 57% of binary and 99% of four-way relations cross sentences; 20% and 55% cross sections. Its end-to-end four-way F1 was .008, rising to .268 with gold salient-cluster information. Salient-entity identification was a principal bottleneck. It also shows why an N-ary result should not always be decomposed into independent binary facts.

**Recommendation.** For PL papers, separately annotate and score:

- all concept mentions in dense gold sections;
- canonical concept clusters across the full document;
- whether a work merely mentions, actually uses, or claims to introduce a concept;
- document-level relations and N-ary experimental claims;
- evidence and qualifiers.

Do not use mention frequency as a proxy for salience or introduction. Do not claim full-document mention recall if only selected sections received dense gold annotation.

### 2.7 Claims and evidence

**Verified.** SciFact, published at EMNLP 2020 ([ACL Anthology](https://aclanthology.org/2020.emnlp-main.609/)), supplies about 1,400 expert-written claims, evidence abstracts, SUPPORTS/REFUTES labels, and rationale annotations. Its task is retrieval and verification of **given** claims, not claim extraction. GraphRAG's optional covariates establish that an LLM pipeline can emit claim-like records, but its official docs explicitly require prompt tuning.

**Recommendation.** Model a claim as a first-class, atomic, qualified statement rather than an edge description. Keep the exact author-facing text, a normalized statement, attribution, polarity/conditionality, and all evidence needed to support it. Evidence can support, refute, or qualify a claim. A conclusion quoted without a necessary premise is incomplete evidence and cannot enter the approved graph.

### 2.8 Cross-chunk and cross-document resolution

**Verified.** KGGen v2 was posted 2025-11-06 (v1 2025-02-14; [full text](https://arxiv.org/html/2502.09956v2)). It extracts entities first, then triples using those entities, aggregates sources, normalizes exact duplicates, and runs separate entity and relation resolution. Candidate generation uses clusters, fused BM25/semantic retrieval, and nearest candidates; an LLM is instructed to merge only true duplicates and retain a canonical representative and aliases. Relations link to source chunks. The authors' MINE benchmarks measure information retention and downstream QA, not gold pairwise/cluster entity resolution. The paper explicitly notes over-/under-merging and specialized-domain limitations.

**Recommendation.** Borrow the two-stage candidate/adjudication pattern but make it more conservative:

- candidate retrieval recall is measured separately from adjudication precision;
- two concepts merge only when they refer to the same formal notion, not when one generalizes, encodes, or is related to another;
- adjudication sees both definitions and local contexts, not just names;
- all aliases and evidence links survive a merge;
- merge/split decisions are reversible and reviewable;
- unresolved candidates remain provisional rather than being forced into an existing concept.

### 2.9 Citations and references

**Verified.** The official [GROBID repository](https://github.com/grobidOrg/grobid) describes scholarly PDF-to-TEI conversion, sections/paragraphs/reference callouts, bibliographic references and DOI/PMID consolidation, and PDF coordinates. Its README reports task-specific F1 values on biomedical corpora, but these are not guarantees for PL PDFs. No exact current release date/version was visible on the fetched landing page.

**Verified.** CitationIE, published August 2021 ([ACL Anthology](https://aclanthology.org/2021.acl-long.59/)), shows that citation graph structure/content can improve scientific IE. It models referential links, not fine-grained entailment that a citing work extends a cited work.

**Recommendation.** Parse references and citation contexts deterministically first, link to local Zotero Works by DOI and verified bibliographic matches, and leave ambiguous references unresolved. `CITES` is a bibliographic fact. `EXTENDS_WORK`, `USES`, `COMPARES`, and `REFUTES` require explicit citation-context evidence; none is inferred from a bibliography entry alone.

---

## 3. Recommended small PL ontology

### 3.1 Design principles

1. **No unearned taxonomy.** There is no generic `IS_A`/subclass edge in v0. In particular, “algebraic effects”, “delimited control”, “free monads”, and “continuations” are related concepts, not subclasses of one another without an explicit formal statement.
2. **Identity is not similarity.** Encoding, generalization, dependence, and comparison are different relations.
3. **Attribution is explicit.** A survey's description of a technique is not the survey's introduction of it.
4. **Evidence is part of the assertion.** No bare triples.
5. **Claims are not ordinary concepts.** A theorem, empirical result, or design claim gets a `Claim` node.
6. **Keep the first version small.** Extend only when a competency question cannot be answered.

### 3.2 Node types

| Type | Purpose | Examples in the target areas |
|---|---|---|
| `Work` | Existing Zotero library work | Numeric Zotero `itemID` remains the ID |
| `Attachment` | Existing Zotero attachment | Source PDF/HTML, numeric Zotero attachment ID |
| `Concept` | Canonical formal notion with `kind` below | Higher-rank polymorphism, bidirectional typechecking, initial algebra, shift/reset |
| `Tool` | Concrete implementation/system | Lean, a named typechecker or effects library |
| `Claim` | Atomic theorem, definition claim, empirical result, or design claim | “The algorithm is complete under restriction R” |
| `EvidenceSpan` | Exact source quote and location | Attachment/chunk/page/offsets/text |
| `Experiment` | Described evaluation, not merely a related-work comparison | A benchmark of elaboration or type inference |
| `BenchmarkSuite` | Named corpus, suite, or set of programs | A paper-specific suite when no public dataset exists |
| `Chunk` | Stable parser-derived text unit | Section/page-preserving unit |

`Concept.kind` is a **presentation/category field**, not a subclass graph:

- `technique`: bidirectional typechecking, normalization by evaluation, recursion schemes;
- `formal_system`: System F, MLF, a specific dependent type theory;
- `language_feature`: higher-rank polymorphism, algebraic effects, delimited control, instance parameters;
- `mathematical_object`: fixed point, initial algebra, free monad, continuation;
- `property`: soundness, completeness, decidability, confluence, principal typing;
- `proof_method`: logical relations, induction, coinduction, translation;
- `representation`: elaboration representation, syntax representation, handler representation.

These categories are intentionally coarse. A difficult categorization can remain `needs_review`; it does not justify inventing ontology edges.

### 3.3 Relations

| Relation | Domain → range | Required evidence and exclusion |
|---|---|---|
| `INTRODUCES` | Work → Concept or Tool | Explicit own-contribution/definition/novelty evidence; not “X introduced” in related work |
| `USES` | Work → Concept or Tool | The work's method/proof/implementation actually uses it; not a passing mention |
| `MENTIONS` | Work → Concept | Explicit occurrence without enough evidence for `USES`; low-information and never promoted automatically |
| `EXTENDS_WORK` | Work → Work | Explicit statement that the citing work builds on/extends the cited work |
| `EXTENDS_CONCEPT` | Work → Concept | Explicit own extension of a named formalism/technique |
| `GENERALIZES` | Concept → Concept | Explicit formal generalization, direction = general → special; not transitive in the query layer unless separately proven |
| `DEPENDS_ON` | Concept → Concept | Explicit prerequisite/dependency; not mere co-occurrence |
| `RELATED_TO` | Concept ↔ Concept | Weak curated association; symmetric display hint only, non-transitive, never used as subclass |
| `ASSERTS` | Work → Claim | The work itself advances the claim; attribution required for background claims |
| `ABOUT` | Claim → Concept/Tool/Experiment | Claim subject linkage; evidence is still mandatory |
| `HAS_EVIDENCE` | Claim/Assertion → EvidenceSpan | `stance = supports|refutes|qualifies`; exact quote and locator |
| `REPORTS` | Work → Experiment | The paper reports the evaluation |
| `EVALUATES` | Experiment → Concept/Tool | Object of the reported evaluation |
| `ON_SUITE` | Experiment → BenchmarkSuite | Explicit suite identification; unnamed suite remains a provisional node |
| `IMPLEMENTS` | Tool → Concept | Explicit implementation evidence |
| `CITES` | Work → Work | Bibliographic citation plus context; `citation_function` is separate and may be `unknown` |
| `ALIAS_OF` | Surface form → Concept | Reversible alias decision, not an assertion that two concepts are equal |

`GENERALIZES` is included only because PL papers often state it explicitly. It is not a substitute for a subclass hierarchy, and its inverse is not stored as an independent fact.

### 3.4 Required negative distinctions

| Text situation | Correct output | Incorrect output |
|---|---|---|
| “Unlike bidirectional typechecking, our algorithm uses unification.” | `MENTIONS bidirectional typechecking`; `USES unification` if this is the work's method | `USES bidirectional typechecking` |
| “Dunfield and Krishnaswami introduce …” in another paper | `CITES` with context; attribution to the original work if locally linked | Current survey `INTRODUCES` the concept |
| “We introduce a bidirectional algorithm …” | Current Work `INTRODUCES` the named algorithm, not necessarily every concept in the sentence | Current Work `INTRODUCES bidirectional typechecking` as a whole |
| “Effects can be expressed using delimited continuations.” | Explicit encoding/`DEPENDS_ON` only if defined in the manual; otherwise `RELATED_TO` plus claim | Delimited control `GENERALIZES` algebraic effects, or a subclass edge |
| “Following Reynolds [12], we …” | `CITES`; possibly `USES` with exact context | `EXTENDS_WORK` automatically |
| “We extend MLF …” | `EXTENDS_CONCEPT MLF`; `CITES` if a specific work is cited | Mere `RELATED_TO`, or extension of an unrelated cited paper |
| Theorem conclusion quoted without a necessary positivity/termination premise | `needs_revision`, incomplete evidence | Approved unconditional claim |
| “X is not decidable” / “If R holds, X is complete” | Negative/conditional claim with qualifier | Positive unconditional claim |
| “Free monads” and “freer monads” | Separate concepts unless the text explicitly establishes identity/generalization | Merge because names are similar |
| “effect” as ordinary side effect vs “algebraic effect” | Separate surface meanings; provisional if ambiguous | Merge by token overlap |

---

## 4. Example typed extraction schema

The following JSON Schema is a concrete per-chunk contract. In implementation it can be generated from equivalent Pydantic models. Provider support for each JSON Schema keyword must be checked; Pydantic and custom validators remain authoritative even where the provider ignores `minItems` or other constraints.

All model-produced review states must initially be `unreviewed`. `start_char`/`end_char` may be null from the model, but the deterministic ingest validator must populate and verify them against parser text before staging. Every assertion, claim, and citation function has at least one exact chunk/page/quote evidence object.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "PLGraphChunkExtraction",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "schema_version", "work_item_id", "chunk_id", "concepts",
    "mentions", "assertions", "claims", "citations", "unresolved"
  ],
  "properties": {
    "schema_version": {"const": "pl-graph-v0"},
    "work_item_id": {"type": "integer", "minimum": 1},
    "chunk_id": {"type": "string", "minLength": 1},
    "concepts": {
      "type": "array",
      "items": {"$ref": "#/$defs/Concept"}
    },
    "mentions": {
      "type": "array",
      "items": {"$ref": "#/$defs/Mention"}
    },
    "assertions": {
      "type": "array",
      "items": {"$ref": "#/$defs/Assertion"}
    },
    "claims": {
      "type": "array",
      "items": {"$ref": "#/$defs/Claim"}
    },
    "citations": {
      "type": "array",
      "items": {"$ref": "#/$defs/Citation"}
    },
    "unresolved": {
      "type": "array",
      "items": {"$ref": "#/$defs/Unresolved"}
    }
  },
  "$defs": {
    "ReviewStatus": {
      "enum": ["unreviewed", "approved", "rejected", "needs_revision"]
    },
    "Evidence": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "attachment_item_id", "chunk_id", "page", "quote",
        "start_char", "end_char"
      ],
      "properties": {
        "attachment_item_id": {"type": "integer", "minimum": 1},
        "chunk_id": {"type": "string", "minLength": 1},
        "page": {"type": "integer", "minimum": 1},
        "quote": {"type": "string", "minLength": 1},
        "start_char": {"type": ["integer", "null"], "minimum": 0},
        "end_char": {"type": ["integer", "null"], "minimum": 0}
      }
    },
    "Concept": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "local_id", "canonical_name", "kind", "aliases",
        "definition_evidence", "review_status"
      ],
      "properties": {
        "local_id": {"type": "string", "pattern": "^c[0-9]+$"},
        "canonical_name": {"type": "string", "minLength": 1},
        "kind": {
          "enum": [
            "technique", "formal_system", "language_feature",
            "mathematical_object", "property", "proof_method",
            "representation", "needs_review"
          ]
        },
        "aliases": {"type": "array", "items": {"type": "string"}},
        "definition_evidence": {
          "anyOf": [{"$ref": "#/$defs/Evidence"}, {"type": "null"}]
        },
        "review_status": {"$ref": "#/$defs/ReviewStatus"}
      }
    },
    "Mention": {
      "type": "object",
      "additionalProperties": false,
      "required": ["concept_local_id", "surface", "mention_kind", "evidence"],
      "properties": {
        "concept_local_id": {"type": "string"},
        "surface": {"type": "string", "minLength": 1},
        "mention_kind": {
          "enum": ["name", "abbreviation", "pronoun", "description"]
        },
        "evidence": {"$ref": "#/$defs/Evidence"}
      }
    },
    "Assertion": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "local_id", "subject_ref", "predicate", "object_ref",
        "qualifiers", "assertion_status", "evidence", "review_status"
      ],
      "properties": {
        "local_id": {"type": "string", "pattern": "^a[0-9]+$"},
        "subject_ref": {"type": "string", "minLength": 1},
        "predicate": {
          "enum": [
            "INTRODUCES", "USES", "MENTIONS", "EXTENDS_WORK",
            "EXTENDS_CONCEPT", "GENERALIZES", "DEPENDS_ON",
            "RELATED_TO", "EVALUATES", "IMPLEMENTS"
          ]
        },
        "object_ref": {"type": "string", "minLength": 1},
        "qualifiers": {"type": "array", "items": {"type": "string"}},
        "assertion_status": {"enum": ["explicit", "uncertain"]},
        "evidence": {
          "type": "array",
          "minItems": 1,
          "items": {"$ref": "#/$defs/Evidence"}
        },
        "review_status": {"$ref": "#/$defs/ReviewStatus"}
      }
    },
    "ClaimEvidence": {
      "type": "object",
      "additionalProperties": false,
      "required": ["stance", "span"],
      "properties": {
        "stance": {"enum": ["supports", "refutes", "qualifies"]},
        "span": {"$ref": "#/$defs/Evidence"}
      }
    },
    "Claim": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "local_id", "text", "normalized_statement", "kind", "polarity",
        "qualifiers", "subject_refs", "attributed_to_work_item_id",
        "evidence", "review_status"
      ],
      "properties": {
        "local_id": {"type": "string", "pattern": "^cl[0-9]+$"},
        "text": {"type": "string", "minLength": 1},
        "normalized_statement": {"type": "string", "minLength": 1},
        "kind": {
          "enum": ["theorem", "definition", "empirical", "design", "background"]
        },
        "polarity": {
          "enum": ["asserted", "negated", "conditional", "uncertain"]
        },
        "qualifiers": {"type": "array", "items": {"type": "string"}},
        "subject_refs": {"type": "array", "items": {"type": "string"}},
        "attributed_to_work_item_id": {"type": ["integer", "null"]},
        "evidence": {
          "type": "array",
          "minItems": 1,
          "items": {"$ref": "#/$defs/ClaimEvidence"}
        },
        "review_status": {"$ref": "#/$defs/ReviewStatus"}
      }
    },
    "Citation": {
      "type": "object",
      "additionalProperties": false,
      "required": [
        "raw_reference", "cited_work_item_id", "citation_function",
        "evidence", "review_status"
      ],
      "properties": {
        "raw_reference": {"type": "string", "minLength": 1},
        "cited_work_item_id": {"type": ["integer", "null"]},
        "citation_function": {
          "enum": ["background", "uses", "extends", "compares", "refutes", "unknown"]
        },
        "evidence": {
          "type": "array",
          "minItems": 1,
          "items": {"$ref": "#/$defs/Evidence"}
        },
        "review_status": {"$ref": "#/$defs/ReviewStatus"}
      }
    },
    "Unresolved": {
      "type": "object",
      "additionalProperties": false,
      "required": ["text", "reason"],
      "properties": {
        "text": {"type": "string"},
        "reason": {"type": "string"}
      }
    }
  }
}
```

### Deterministic semantic checks beyond the schema

- The root Work/chunk/attachment IDs match the requested input; the model cannot redirect provenance.
- Every local reference resolves to an object in the same output or an explicitly supplied Work/Concept ID.
- Quote text occurs verbatim in the specified parser chunk and page; offsets are regenerated and checked. Fuzzy matching can queue review but cannot auto-approve.
- No assertion is accepted with `assertion_status=uncertain`; it is routed to review.
- Domain/range and direction rules hold. `GENERALIZES` is general → special.
- Every claim has complete evidence for its normalized statement, polarity, attribution, and all qualifiers. A model cannot self-certify this; humans score it.
- `CITES` never creates `EXTENDS_WORK`; `MENTIONS` never creates `USES`; `RELATED_TO` never creates `GENERALIZES`.
- Duplicate evidence caused by overlapping chunks is deduplicated by attachment/page/offsets/quote, not by dropping distinct supporting passages.

---

## 5. Robust extraction prompt

Use a frozen, versioned prompt and the JSON Schema above. The input supplies `WORK_ITEM_ID`, `ATTACHMENT_ITEM_ID`, `CHUNK_ID`, section path, page-marked text, and a short preceding-heading context. Do not put private library content into a secondary analyst or a hosted route without approval.

```text
You are extracting a reviewable programming-languages knowledge graph from one
source chunk. Use only the supplied source text. Do not use prior knowledge to
complete a theorem, identify a citation, infer authorship, or resolve an ambiguous
concept. Return only an object conforming to PLGraphChunkExtraction.

IDENTITY AND PROVENANCE
- Echo only the supplied WORK_ITEM_ID, ATTACHMENT_ITEM_ID, and CHUNK_ID.
- Every assertion, claim, and citation function needs at least one verbatim quote
  with its PDF page. Never invent a quote, page, identifier, or bibliographic match.
- The evidence must support the entire normalized statement, including direction,
  attribution, negation, conditions, and qualifiers. If the quote supports only
  part, emit no approved assertion; put the issue in unresolved.
- If mathematical notation or text is too garbled to read, put it in unresolved.
- Set every review_status to unreviewed. Use assertion_status=uncertain when the
  wording does not explicitly establish the assertion.

CONCEPTS AND COREFERENCE
- Extract only concepts explicitly present in this chunk. Preserve aliases and
  abbreviations. Do not merge concepts merely because they are similar, related,
  encodable in one another, or used together.
- A pronoun or descriptive mention may be linked only when its antecedent is
  explicit in the supplied context. Otherwise leave it unresolved.
- Do not infer subclass relationships. There is no IS_A output.

RELATIONS
- INTRODUCES: only the current work's explicitly stated own contribution.
  “X introduced ...” in related work is attribution to X, not this work.
- USES: the current work's own method, proof, implementation, or evaluation uses
  the concept/tool. Background, comparison, and future work are not USES.
- MENTIONS: explicit occurrence with insufficient evidence for USES or INTRODUCES.
- EXTENDS_WORK / EXTENDS_CONCEPT: require explicit extension/build-on wording and
  the correct target. A citation alone is never extension.
- GENERALIZES: require explicit formal generalization. Direction is the more
  general concept to the more specific one. RELATED_TO is weak and non-transitive.
- DEPENDS_ON: require an explicit prerequisite, not co-occurrence.
- Keep a bibliography citation as CITES with citation_function=unknown unless the
  citation context explicitly supports background, uses, extends, compares, or
  refutes. CITES does not create any of those semantic relations by itself.

CLAIMS
- Extract atomic theorem, definition, empirical, design, and background claims.
  Split conjunctions when each conjunct needs separate evidence.
- Preserve exact qualifiers, assumptions, scope, polarity, and attribution.
- Theorem conclusions without their required premises are incomplete evidence.
- Mark conditional or negated claims accordingly. A proof sketch's presence alone
  does not prove that a claim is correct; record only what the source asserts.
- Use evidence stance=qualifies for assumptions/scope and supports/refutes for
  the claim itself. Multiple spans are allowed and may all be necessary.

Do not add explanations outside the schema. Prefer an empty list or an unresolved
record to guessing.
```

For a second completeness pass, supply the original chunk plus the already extracted local IDs and ask only for **missing explicit mentions/claims with evidence**. Do not ask the model to “improve” or rewrite accepted assertions. Merge new records by deterministic IDs/evidence and run all validators again.

---

## 6. Pipeline stages and HawDB mapping

### 6.1 Pipeline

1. **Manifest and snapshot.** Record Zotero `itemID`, attachment `itemID`, attachment path availability, file SHA-256, parser version, and extraction-run configuration. The hash invalidates derivatives only; it never replaces Work identity.
2. **Local parse.** Produce page-marked text, section hierarchy, bibliography, citation callouts, and coordinates where possible. Start with native text extraction; OCR only pages with insufficient text. Preserve the original parser output so quote checks do not depend on later cleanup.
3. **Parse-quality gate.** Measure text coverage per page, empty pages, replacement characters, reading-order anomalies, and garbled math. Route bad pages to review/OCR. Do not silently extract from damaged text.
4. **Chunk.** Start with section-aware 600–900-token chunks and 100-token overlap as an experimental setting, not a universal optimum. Keep theorem/definition statements with their premises where possible. Give every chunk a stable ID derived from attachment ID, parser version, ordinal, and content hash.
5. **References.** Parse raw references and citation contexts; link DOI/exact verified bibliographic matches to existing Zotero Works. Retain unresolved references with raw text rather than guessing.
6. **Per-chunk extraction.** Use the strict schema/prompt. Store raw response, status/finish reason, token usage, latency, and validation errors. Retry at most once for structural failure; never hide a failed call.
7. **Deterministic evidence validation.** Resolve quotes and offsets, check pages/chunks/IDs, local references, domain/range, and duplicate overlap. Quarantine failures.
8. **Within-document coreference.** Build alias/abbreviation candidates across chunks, including explicit definitions and section context. Use lexical+vector retrieval for candidates and an adjudicator only for exact identity. Measure candidate recall separately.
9. **Cross-document resolution.** Prefer reviewed canonical IDs and exact known aliases. New/ambiguous concepts remain provisional. Merges are reversible union/find decisions with alias and evidence lineage, not destructive node rewrites.
10. **Claim and high-risk review.** Humans review all `INTRODUCES`, `EXTENDS_*`, `GENERALIZES`, `REFUTES`, uncertain evidence, proposed merges, and a stratified sample of `USES`/`MENTIONS`. Record approve/reject/revise and reviewer time.
11. **Transactional load.** Only approved canonical assertions enter the query graph. Raw/unreviewed records remain in staging tables. A run is idempotent by run ID and content hash.
12. **Incremental update.** Reprocess only changed attachments, changed parser/schema/prompt versions, or requested reannotation. Supersede old extraction assertions while preserving prior raw responses and review history. Deleted attachments receive tombstones.
13. **Retrieval and QA.** Combine existing BM25/vector Work retrieval with reviewed graph traversal and evidence retrieval. Answers must cite EvidenceSpans. Community summaries are deferred until a measured question requires them.

### 6.2 SQL staging/canonical tables

Exact HawDB DDL should be validated in implementation; these are logical tables, not untested SQL claims.

| Table | Important keys/fields |
|---|---|
| `extraction_runs` | `run_id` PK, exact provider/model, parameters, prompt/schema/parser hashes, code version, start/end, input/output/cached tokens, latency, cost, status |
| `chunks` | `chunk_id` PK, `work_id`, `attachment_id`, content hash, parser version, ordinal, section path, page range, raw text, offsets |
| `concepts` | `concept_id` PK, canonical name, kind, review state, created/updated run; never use normalized name alone as proof of identity |
| `concept_aliases` | alias ID, concept ID, surface form, evidence ID, review state |
| `mentions` | mention ID, concept ID, work/chunk/evidence IDs, surface, mention kind |
| `assertions` | assertion ID, subject type/ID, predicate, object type/ID, qualifiers JSON, assertion/review states, run ID, supersedes ID |
| `claims` | claim ID, Work ID, text, normalized statement, kind, polarity, qualifiers JSON, attributed Work ID, review state, run ID |
| `evidence` | evidence ID, owner type/ID, stance, attachment/chunk IDs, page, start/end offsets, exact quote, quote-validation state |
| `citations` | citation ID, citing Work ID, raw reference, cited Work ID nullable, DOI nullable, citation function, review state |
| `experiments` / `benchmark_suites` | Stable local IDs, names/descriptions, review state |
| `reviews` | target type/ID, reviewer ID, decision, note, timestamp, previous/new value |
| `entity_decisions` | candidate pair/cluster, decision merge/distinct/uncertain, canonical/alias mapping, evidence, reviewer/run, reversible operation ID |

### 6.3 Graph projection

- Existing `Work(id: itemID)` and `Attachment(id: attachmentItemID)` nodes are reused, never duplicated under hash IDs.
- Add `Concept`, `Claim`, `EvidenceSpan`, `Chunk`, `Experiment`, `BenchmarkSuite`, and `Tool` nodes.
- Project each approved assertion to its named edge with `assertionId`, `runId`, and `reviewStatus`; link it to all supporting `EvidenceSpan`s.
- Project `Work-[:ASSERTS]->Claim`, `Claim-[:ABOUT]->Concept`, and `Claim-[:HAS_EVIDENCE {stance}]->EvidenceSpan`.
- `CITES` carries `citationFunction` and `assertionId`; multiple citation contexts can produce multiple assertion records.
- Keep all source links after an entity merge. A merge changes canonical lookup, not the original mention or evidence.
- Keep unapproved staging out of the canonical graph rather than relying on every query to remember a review filter.

---

## 7. Pre-registered experiment and evaluation

### 7.1 Gold set drawn from the actual library

The subagent initially selected the following 12 works from local Zotero metadata and confirmed PDF **attachment rows**, not readable files. A later parent filesystem check found that `60` and `5` lacked their referenced local PDFs, so this initial list is **not run-ready**. The authoritative revised corpus is `../experiments/ontology-extraction/round-001-corpus.json`: 15 works whose local PDFs all exist and passed a `%PDF-` signature check. PDF parse quality has **not** yet been tested.

| Split | Zotero `itemID` | Title | Primary stratum |
|---|---:|---|---|
| Development | 3154 | Complete and Easy Bidirectional Typechecking for Higher-Rank Polymorphism | Higher-rank + bidirectional |
| Development | 713 | Fantastic Morphisms and Where to Find Them: A Guide to Recursion Schemes | Recursion schemes |
| Development | 49 | Freer Monads, More Extensible Effects | Algebraic effects |
| Development | 5 | Elaboration in Dependent Type Theory | Dependent types/elaboration |
| Test | 60 | FreezeML: Complete and Easy Type Inference for First-Class Polymorphism | Higher-rank polymorphism |
| Test | 2628 | Bidirectional Typing | Bidirectional typechecking |
| Test | 2314 | Unifying structured recursion schemes | Recursion schemes/generalization |
| Test | 7 | Induction, Coinduction, and Fixed Points: Intuitions and Tutorial | Fixed points |
| Test | 675 | Answer-Type Modification without Tears: Prompt-Passing Style Translation for Typed Delimited-Control Operators | Delimited control |
| Test | 1483 | Hefty Algebras: Modular Elaboration of Higher-Order Algebraic Effects | Higher-order algebraic effects |
| Test | 6 | The Lean Theorem Prover (System Description) | Lean/dependent types |
| Test | 774 | Use and abuse of instance parameters in the Lean mathematical library | Lean/instance parameters |

Use the four development papers to refine the manual, prompt, and deterministic validators. Freeze them before touching test predictions. Run the eight test papers once per arm. Eight test documents are enough for an engineering gate, not a claim of broad scientific significance.

### 7.2 Annotation protocol

- Annotate full documents for salient canonical concepts, own-work `INTRODUCES`/`USES`, document-level extensions/generalizations, and citation links.
- Dense-annotate three predefined section types per paper: abstract/introduction, core definitions/method, and theorem/evaluation sections, plus all citation contexts for selected high-risk relations. Record exact section boundaries before seeing model output.
- Do not score whole-document mention recall outside dense zones.
- Two PL-knowledgeable annotators independently annotate six of 12 papers, covering every stratum. Adjudicate only after agreement metrics are computed. The remaining six receive one full annotation plus a second review of all high-risk relations and a random 20% of claims/evidence.
- Annotate canonical identity separately from surface mention labeling. A merge requires a definition-level justification.
- Claims are atomic. Record all premises/qualifiers, polarity, attribution, and whether evidence is full, partial, or unrelated.
- Use a versioned manual with the negative examples above. Freeze gold before unblinding predictions.

### 7.3 Comparison arms

All arms use the same parsed text, chunks, exact model snapshot, decoding settings, input order, and retry policy. Cache raw responses. Record all calls, including validator/adjudicator calls. A fixed random seed of **20261118** controls ordering and bootstrap resampling.

| Arm | Extraction | Resolution/repair | Purpose |
|---|---|---|---|
| `G1` | GraphRAG-style generic entity/relationship prompt, one pass, no PL schema | Exact title+type merge | Generic baseline |
| `G2` | Same generic prompt with completeness gleaning | Same total maximum LLM-call budget as `T2`; generic repair only | Budget-matched generic control |
| `T1` | Typed JSON Schema/prompt, one pass | Exact known aliases only; deterministic validation | Isolate ontology/structured-output contribution |
| `T2` | Typed prompt plus targeted completeness pass | Deterministic evidence repair, hybrid candidate retrieval, conservative identity adjudication | Proposed full method |

No human corrections are allowed when scoring raw arm output. A separate `T2-reviewed` condition measures the intended human-in-the-loop system. Do not compare a reviewed `T2` graph against unreviewed generic output and attribute the gain to the model.

Frameworks are implementation details. If GraphRAG/LlamaIndex adapters cannot represent the required evidence, store evidence in the sidecar schema rather than weakening `T2`.

### 7.4 Pre-registered hypotheses

- **H1:** `T2` improves strict, fully evidenced relation micro-F1 by at least 0.05 over `G2`, with high-risk relation precision at least 0.95.
- **H2:** `T2` improves claim evidence-sufficiency and qualifier accuracy over `G2` without exceeding 2× its LLM cost per accepted claim.
- **H3:** conservative resolution improves B-cubed cluster F1 over exact-name matching while maintaining pairwise merge precision of at least 0.98.
- **H4:** the reviewed typed graph improves downstream answer quality over the existing metadata/abstract hybrid search and produces at least 0.90 citation precision.

Report paired document-bootstrap 95% intervals and exact counts. With eight test papers, a point improvement without a stable interval is “inconclusive”, not a validated win.

### 7.5 Metrics and exact scoring rubrics

#### A. Ontology and structural validity

- **Schema-valid rate:** outputs passing JSON/Pydantic validation after at most one structural retry ÷ all model calls. Also report first-pass rate.
- **Full ontology conformance:** assertions with an allowed predicate, valid domain/range, correct direction, and resolved endpoint types ÷ all emitted assertions. This is stricter than Text2KGBench's relation-name-only OC.
- **Hallucinated subject/relation/object rates:** each unsupported component ÷ all corresponding components, with exact source-text/ontology checks plus manual review of aliases.
- **Competency-question coverage:** answerable pre-registered graph questions ÷ all competency questions, using only approved graph facts and evidence.
- **Duplication and false-merge rates:** reviewed duplicate canonical concepts ÷ canonical concepts; false merged pairs ÷ all predicted merges.

#### B. Entities, mentions, and coreference

- **Exact mention F1:** true positive requires exact span and `Concept.kind`; report micro P/R/F1.
- **Relaxed mention F1:** span overlap at least 50% with correct kind; secondary only.
- **Canonical link accuracy:** mentions linked to the gold canonical concept ÷ all linked mentions; unresolved predictions count separately for recall.
- **Pairwise merge P/R/F1:** predicted same-concept mention pairs versus gold pairs.
- **B-cubed P/R/F1:** per-mention precision/recall of predicted versus gold clusters, averaged by mention.
- **Salience/role macro-F1:** `MENTIONS`, `USES`, and `INTRODUCES` scored as distinct labels, with per-class counts. This prevents the common `MENTIONS` class from hiding rare but important introduction errors.

#### C. Relations

Report two scores:

1. **Semantic relation F1:** subject ID + predicate + object ID + direction correct, ignoring evidence quality.
2. **Strict evidenced relation F1 (headline):** the semantic tuple is correct **and** its evidence set is complete and sufficient.

A true positive requires all elements; there is no half credit in the headline score. Report an endpoint-only diagnostic separately. Also report per-predicate P/R/F1 and direction accuracy for `GENERALIZES`/`EXTENDS_*`.

#### D. Claims

- **Atomicity:** 0 = combines independently evaluable claims or splits one indivisible condition; 1 = atomic with necessary condition attached.
- **Normalized correctness:** 0 = unsupported/wrong; 1 = partially correct or missing qualifier/attribution; 2 = exact statement, polarity, scope, qualifiers, and attribution.
- **Claim precision:** claims scoring 2 for correctness and 1 for atomicity ÷ all predicted claims.
- **Claim recall:** gold atomic claims recovered with correct normalized meaning ÷ all gold claims in dense zones.
- **Qualifier exact accuracy:** predicted claims with exactly the gold qualifiers ÷ matched claims.
- **Polarity/attribution accuracy:** exact match, reported separately with confusion matrices.
- **N-ary experiment accuracy:** all required Experiment/Concept/Tool/Suite/result roles correct; do not decompose a genuinely N-ary result into independent binary true positives.

#### E. Provenance faithfulness

Deterministic checks:

- **Quote validity:** quote occurs exactly in the cited parser page/chunk; valid ÷ all quotes.
- **Locator validity:** attachment/chunk/page/offset combination resolves; valid ÷ all evidence spans.
- **Provenance completeness:** claims/relations with at least one evidence object ÷ all claims/relations.

Human evidence rubric, applied to the **whole evidence set**:

- **0 — unrelated/contradictory:** evidence does not support the statement or supports the opposite.
- **1 — partial:** supports only part, omits a premise/qualifier/attribution, or requires unstated external knowledge.
- **2 — sufficient:** a PL-knowledgeable reader can verify every element from the quoted source alone.

An approved claim/relation must score **2**. Averages cannot hide a score-0/1 high-risk assertion. LLM-assisted entailment may prioritize review but cannot replace this human score.

#### F. Downstream query-answer quality

Create 20 questions: ten deterministic graph/fact questions, six cross-paper synthesis questions, and four unanswerable/unsupported questions. Each has a gold answer, acceptable aliases, required evidence Works/spans, and an abstention rule.

Compare existing metadata/abstract hybrid retrieval, `G2` graph+hybrid, raw `T2` graph+hybrid, and reviewed `T2` graph+hybrid.

- **Answer score:** 0 = wrong/unsupported; 1 = partially correct or incomplete; 2 = correct and complete for the question.
- **Citation precision:** cited evidence that actually supports its attached answer component ÷ all cited evidence.
- **Citation recall:** required gold evidence components cited ÷ all required components.
- **Retrieval MRR and nDCG@10:** computed from deterministic gold relevance for fact questions.
- **Abstention accuracy:** correct refusal to invent an answer on the four unsupported questions plus wrong abstentions on answerable questions.
- Blind human scoring to arm and order. An LLM judge may be reported as a secondary agreement study only, never as the sole grade.

#### G. Cost, latency, and review burden

- Input/output/cached tokens and calls per 1,000 source tokens and per paper.
- Wall-clock p50/p95 per paper and per stage; queue time reported separately.
- Actual spend using the dated provider price in the run manifest; for local inference, report hardware/runtime rather than inventing a dollar price.
- Cost and calls per schema-valid output, per accepted assertion, and per fully evidenced accepted claim.
- Structural retry, empty-content, truncation, and silent-error rates.
- Human review seconds per assertion/claim, minutes per paper, and fraction auto-approved after deterministic checks.

#### H. Inter-annotator agreement

- Exact-span F1 between annotators before adjudication for mentions.
- B-cubed and pairwise agreement for concept clusters.
- Cohen's kappa plus raw agreement and prevalence for citation function, polarity, and `MENTIONS`/`USES`/`INTRODUCES`.
- Krippendorff's alpha (ordinal) for the 0/1/2 evidence rubric and answer-quality rubric.
- Report disagreements by category; do not hide low agreement by adjudicating first.

### 7.6 Seeded failure cases

These short synthetic cases belong in a deterministic regression suite. Their expected labels are fixed before model runs.

| Seed | Input pattern | Required result / failure caught |
|---|---|---|
| Mention vs use | “Unlike bidirectional typechecking, we use unification.” | No `USES bidirectional typechecking`; role contrast correct |
| Own vs attributed introduction | “Dunfield and Krishnaswami introduce X.” | No current-Work `INTRODUCES X` |
| Broad introduction overreach | “We introduce a bidirectional algorithm for S.” | Introduces the specific algorithm, not bidirectional typing generally |
| Related vs subclass | “Delimited continuations can encode algebraic effects.” | No `GENERALIZES`/subclass edge |
| Similar names, distinct concepts | “Free monads” beside “freer monads” | No automatic merge |
| Same abbreviation, distinct senses | “effects” as ordinary side effects vs algebraic effects | Provisional/distinct, no merge |
| Citation vs extension | “Following [12], we define …” | `CITES`; no `EXTENDS_WORK` without explicit text |
| Explicit extension direction | “Our system S extends MLF.” | `S/Work EXTENDS_CONCEPT MLF`, not reverse |
| Wrong generalization direction | “G generalizes S.” | `G GENERALIZES S`, never `S GENERALIZES G` |
| Incomplete theorem evidence | Conclusion quote missing “assuming positivity” | Evidence score 1; no approved claim |
| Negation | “The problem is not decidable.” | Negated claim, not decidability |
| Conditional | “If R holds, the algorithm is complete.” | Conditional claim with R qualifier |
| Conjunctive claims | “Sound and complete, with completeness requiring R.” | Atomic claims/evidence sets, no unconditional conjunction |
| Bibliography-only occurrence | Reference title mentions a concept | No `USES` or `INTRODUCES` from bibliography text |
| Garbled math | Replacement characters in the decisive premise | `unresolved`, no guessed claim |
| Overlap duplication | Same theorem appears in two overlapping chunks | One assertion with shared evidence, not two independent supports |
| Empty model content | `content=""`, finish reason length | Hard failure/retry policy invoked; never counted as empty extraction |
| Unknown citation target | Ambiguous abbreviated reference | `cited_work_item_id=null`; no nearest-title guess |

### 7.7 Pass/fail gates

These are recommended engineering thresholds for this small first experiment, not externally established benchmarks.

| Gate | Exact criterion | Failure action |
|---|---|---|
| Annotation readiness | On double annotation: mention F1 ≥ .80, relation F1 ≥ .80, role/citation kappa ≥ .75, evidence alpha ≥ .80; any alpha < .67 is an automatic stop | Revise manual and reannotate before model comparison |
| Structural reliability | First-pass schema validity ≥ .98; after one retry 100% of calls have either valid staged output or an explicit logged failure; zero silent swallowed errors | Arm fails; no imputation of empty graphs |
| Provenance validity | 100% of canonical claims/relations have valid chunk/page/quote/offset provenance and evidence rubric 2 | Quarantine offending records; canonical release fails if any remain |
| High-risk semantics | `INTRODUCES`/`EXTENDS_*`/`GENERALIZES`/`REFUTES` precision ≥ .95; pairwise merge precision ≥ .98 | Do not enable automatic publication for that class |
| Headline extraction | `T2` strict evidenced relation micro-F1 ≥ .70 and ≥ .05 above `G2`; B-cubed cluster F1 ≥ .85 | Prefer simpler arm or revise ontology/resolution |
| Claims | Claim precision ≥ .85; sufficient-evidence rate ≥ .95; qualifier exact accuracy ≥ .90 | Keep claims in review-only staging |
| Downstream QA | Reviewed `T2` mean answer score ≥ 1.50/2 and ≥ .25 above metadata hybrid; citation precision ≥ .90; zero fabricated locators; at least 3/4 unsupported questions abstain | Do not present graph QA as an improvement |
| Seeded failures | 100% of high-risk seeds pass and overall seed accuracy ≥ .90; any mention→introduce, citation→extends, related→generalizes, or incomplete-evidence approval is an automatic fail | Fix class and rerun all seeds and affected papers |
| Operational | Total hosted spend for all arms ≤ US$25; `T2` LLM cost per accepted claim ≤ 2× `G2`; p95 extraction latency ≤ 20 minutes/paper | Reduce scope/model/calls or stop; report rather than hide truncation |
| Human burden | Median review ≤ 20 minutes/paper in the final reviewed arm; otherwise report as not operationally acceptable even if F1 passes | Simplify ontology or triage review queue |

If sample size is too small to distinguish a threshold statistically, mark the result **inconclusive**. Do not relax a pre-registered gate after seeing results.

### 7.8 Evaluation harness outline

No harness was implemented in this research task. A future minimal layout could be:

```text
eval/
  preregistration.md          # hypotheses, arms, gates, prices, frozen hashes
  manifest.json               # item/attachment IDs, source hashes, splits, seed
  annotation-manual.md        # same definitions as the model prompt
  gold/<item_id>.json          # mentions, clusters, assertions, claims, evidence
  questions.json              # answers, aliases, required citations, abstentions
  seeds.json                  # synthetic negative cases above
  runs/<arm>/<run_id>/
    config.json               # exact model, parameters, code/schema/prompt hashes
    raw/<chunk_id>.json        # raw response + finish reason + usage + errors
    validated/<chunk_id>.json
    metrics.json
  score.py                     # deterministic structural/span/tuple/quote scoring
  review.py                    # blinded queue and human scores
  report.py                    # paired document bootstrap and exact-count tables
```

Run order:

1. verify hashes/splits and that development and test sets are disjoint;
2. validate gold and annotation agreement;
3. execute all arms with identical source artifacts and frozen configurations;
4. run deterministic validators/scorers before unblinding semantic outputs;
5. blind reviewers to arm, randomize record order, and collect human rubric scores;
6. adjudicate human disagreement separately from model errors;
7. run paired bootstrap by document with seed 20261118, report exact per-class counts and intervals;
8. run every seeded failure and all quote/location checks;
9. publish the run manifest, raw failures, cost/review burden, and any inconclusive gates.

The scorer itself should make no network calls. LLM-as-judge results, if collected, go in a separate file and are never substituted for the gold/human metrics.

---

## 8. Recommended first experiment, step by step

1. **Parse smoke test:** run the local parser on the four development PDFs only. Check page mapping, two-column order, math, references, and exact quote recovery. Record failures rather than cleaning them away invisibly.
2. **Manual/gold pilot:** independently annotate two development papers, calculate agreement, revise definitions, then complete the remaining development and test gold under the frozen manual.
3. **Freeze:** ontology v0, prompt, Pydantic/JSON Schema, parser version, chunker, exact model snapshot, decoding, retry, call budget, US$25 total cap, and all gates.
4. **Development tuning:** compare `G1`, `G2`, `T1`, and `T2` only on the four development works. Select at most one prompt revision and one chunk configuration; record every change. Do not tune against test gold or predictions.
5. **Held-out run:** execute all four arms once on the eight test works with the same source artifacts and call budget for `G2`/`T2`. Preserve raw failures.
6. **Score:** deterministic structural/provenance/tuple scores first, then blind human semantic/evidence/claim/QA scoring and agreement.
7. **Decision:** adopt `T2` only if it passes high-risk, provenance, seeded-failure, cost, and review-burden gates and beats `G2` on strict evidenced relation F1. Otherwise retain metadata search and iterate on the simplest failing stage.
8. **Only then integrate:** change the current full graph wipe/rebuild behavior, add incremental attachment processing, and expose reviewed evidence in the UI. Do not begin with community summaries, fine-tuning, or a large ontology.

The first practical deliverable should be a browsable list of **approved claims/relations with their exact PDF quotes**, not a visually impressive but unreviewed graph.

---

## 9. DeepSeek secondary-analyst attempt

The requested bounded critique was attempted with this exact command and generic research-only standard input:

```bash
dsh headless - <<'EOF'
Research-only bounded critique. Do not browse, do not invent citations, and do not assume access to any private library. Critique this proposed small ontology for extracting concept and claim graphs from programming-languages/type-theory papers:
Node types: Concept (subtypes Technique, Calculus, LanguageFeature, Property, Theorem, Representation), Work, Claim, EvidenceSpan, Experiment, DatasetOrSuite, Tool.
Relations: Work INTRODUCES Concept; Work USES Concept; Work MENTIONS Concept; Work EXTENDS Work-or-Concept; Concept GENERALIZES Concept (only for explicit generalization, not mere relatedness); Concept DEPENDS_ON Concept; Concept RELATED_TO Concept (weak, never transitive); Work PROVES Claim; Claim SUPPORTED_BY EvidenceSpan; Claim REFUTED_BY EvidenceSpan; Experiment EVALUATES Concept; Experiment USES_DATASET DatasetOrSuite; Tool IMPLEMENTS Concept; Work CITES Work with citation_function enum background/uses/extends/compares/refutes.
Every assertion has exact quote, chunk/page, extraction status, and review status. Claims must be atomic and carry scope/qualifiers.
Give: (1) the most serious type/relational design errors, (2) cases where related concepts are wrongly made subclasses, (3) negative examples distinguishing mention/use/introduce and citation/extension, (4) evaluation failure cases and concrete pass/fail gates. Be concise and treat this as design critique, not sourced evidence.
EOF
```

**Result:** failed before analysis with exit code 1:

```text
dsh: MISSING_CREDENTIAL: llm-deepseek: no API key for provider route "deepseek-official"; store DEEPSEEK_API_KEY through the credentials service (the web Models page writes it), or export DEEPSEEK_API_KEY in the launching environment
```

No credential contents were inspected by the subagent, no private Zotero content or personal information was sent, and it did not retry. Its report therefore contained no DeepSeek findings.

**Parent retry, 2026-11-18:** the user had explicitly pointed to `~/.dsh` for the configured key. The parent found a `DEEPSEEK_API_KEY` entry in `~/.dsh/.credentials.yaml`, passed its value directly into the `dsh` process environment without printing it, and reran `dsh headless -` with generic ontology text only. The retry succeeded using the profile's `deepseek-flash` alias. DeepSeek's final critique is preserved in `deepseek-ontology-critique.md`; its intermediate reasoning is not included. Its strongest contributions were cautions about factorized metrics, matched coverage, high-risk violation rates, evidence types, and claim attribution. These remain uncited model suggestions, not primary research evidence, and the parent disposition in that file rejects wholesale ontology expansion.

---

## 10. Unresolved uncertainties and non-claims

- No PDF was parsed and no extraction arm was run. Attachment existence is verified; parser quality and actual extraction quality are unknown.
- GROBID's published biomedical parsing numbers may not transfer to PL papers with dense notation, inference rules, two-column layouts, or old scans.
- Exact current releases for GraphRAG, LangChain, LlamaIndex, Pydantic, and GROBID were not established from undated documentation pages. Implementation must pin and record versions rather than treating this report as a compatibility test.
- KGGen's entity/relation clustering is promising, but its MINE evaluation does not provide gold pairwise merge precision/recall or prove complete lineage after merges.
- SciREX and SciFact are evaluation/design references, not PL extractors. No benchmark located in this timeboxed survey directly measures the proposed PL ontology.
- The 12-paper test is intentionally small and stratified, not representative of all PL/type-theory literature. The eight-paper held-out set limits statistical power.
- Dense gold sections do not justify whole-document mention-recall claims. Full-document salient relations and dense-section mention recall must be reported separately.
- The proposed numeric gates are engineering choices. They should be pre-registered but not described as community standards.
- Model/provider choice and current pricing were not selected. The US$25 cap is a stopping rule, not a cost prediction. Local reasoning/structured-output behavior must be smoke-tested on the actual route.
- Citation-function labels can be ambiguous even for humans; agreement gates may reveal that the six-way scheme needs simplification.
- Formula normalization is unresolved: exact-quote validation should use raw parser text, while human-readable display may need a parallel normalized representation. Fuzzy quote recovery must never silently approve provenance.
- The survey was timeboxed after the core primary sources and concrete evaluation design. It did not exhaustively review every 2025–2026 GraphRAG variant, biomedical claim graph, ontology-learning method, or annotation product.

---

## Primary sources and official documentation

1. Edge et al. **From Local to Global: A Graph RAG Approach to Query-Focused Summarization.** v1 2024-04-24, v2 2025-02-19. [arXiv abstract](https://arxiv.org/abs/2404.16130), [full text](https://arxiv.org/html/2404.16130).
2. Microsoft. **GraphRAG default dataflow, outputs, and configuration.** Official docs, undated as fetched 2026-11-18. [Dataflow](https://microsoft.github.io/graphrag/index/default_dataflow/), [outputs](https://microsoft.github.io/graphrag/index/outputs/), [configuration](https://microsoft.github.io/graphrag/config/yaml/).
3. LangChain. **`LLMGraphTransformer` API reference.** Official docs, undated as fetched 2026-11-18. [Reference](https://reference.langchain.com/python/langchain-neo4j/graph_transformers/llm/LLMGraphTransformer).
4. LlamaIndex. **Using a Property Graph Index.** Official docs, undated as fetched 2026-11-18. [Guide](https://developers.llamaindex.ai/python/framework/module_guides/indexing/lpg_index_guide/).
5. Caufield et al. **Structured Prompt Interrogation and Recursive Extraction of Semantics (SPIRES).** arXiv v2 2023-12-22; *Bioinformatics* 2024-03-04. [arXiv](https://arxiv.org/abs/2304.02711), [DOI](https://doi.org/10.1093/bioinformatics/btae104).
6. Mihindukulasooriya et al. **Text2KGBench: A Benchmark for Ontology-Driven Knowledge Graph Generation from Text.** arXiv 2023-08-04; ISWC 2023. [arXiv](https://arxiv.org/abs/2308.02357), [publisher DOI](https://dl.acm.org/doi/10.1007/978-3-031-47243-5_14).
7. Sainz et al. **GoLLIE: Annotation Guidelines Improve Zero-Shot Information-Extraction.** ICLR 2024; arXiv v5 2024-03-06. [arXiv](https://arxiv.org/abs/2310.03668).
8. **KGGen: Extracting Knowledge Graphs from Plain Text with Language Models.** v1 2025-02-14, v2 2025-11-06. [Full text v2](https://arxiv.org/html/2502.09956v2).
9. Jain et al. **SciREX: A Challenge Dataset for Document-Level Information Extraction.** ACL 2020. [ACL Anthology](https://aclanthology.org/2020.acl-main.670/), [full text](https://arxiv.org/html/2005.00512).
10. Wadden et al. **Fact or Fiction: Verifying Scientific Claims (SciFact).** EMNLP 2020. [ACL Anthology](https://aclanthology.org/2020.emnlp-main.609/), DOI [10.18653/v1/2020.emnlp-main.609](https://doi.org/10.18653/v1/2020.emnlp-main.609).
11. **CitationIE: Leveraging the Citation Graph for Scientific Information Extraction.** ACL-IJCNLP 2021. [ACL Anthology](https://aclanthology.org/2021.acl-long.59/).
12. GROBID. **Official repository and documentation.** Current landing page fetched 2026-11-18; no exact current release date inferred. [Repository](https://github.com/grobidOrg/grobid), [coordinates documentation](https://grobid.readthedocs.io/en/latest/Coordinates-in-PDF/).
13. OpenAI. **Structured model outputs.** Official guide, undated as fetched 2026-11-18. [Guide](https://developers.openai.com/api/docs/guides/structured-outputs).
14. Pydantic. **JSON Schema concepts.** Official docs, undated as fetched 2026-11-18. [Guide](https://pydantic.dev/docs/validation/latest/concepts/json_schema/).
