# Sutra-OS: a self-hosted ontology platform with one write door

**Research date:** 2026-11-18  
**Subject:** [`0nelight/sutra-os`](https://github.com/0nelight/sutra-os), commit `ce8d4f7` — the repository's only commit, dated 2026-11-17.  
**Status:** research note only. Nothing in the repository was executed, installed, or integrated.  
**Evidence convention:** **Verified** means read in the repository at that commit, with the file path (and line where useful). **Claim** means a statement the project makes about itself, quoted or paraphrased, which this note did not independently establish. **Assessment** means a judgement for `paper-graph`. **Not verified** marks questions this note deliberately leaves open. File, line, namespace and test counts are static counts of the checkout, not results of a build or test run.

## Why this note exists

The two research records already in this repository cover opposite ends of the extraction problem. `llm-document-graph-research.md` surveys extraction, schema-constrained output, entity resolution and evaluation. `deepseek-ontology-critique.md` attacks the ontology's semantics: attribution versus claim, evidence spans, direction of generalization, and the vocabulary needed to avoid a catch-all relation. Neither covers the layer between an extracted assertion and a graph other tools may trust: how a write is proposed, checked, approved, attributed, versioned and refused. The repository also turned out to contain a document-intelligence subsystem that is closer to this project's extraction stage than its README suggests; §5.5 records it.

Sutra-OS is the most complete open-source implementation of that layer examined so far. It is a Palantir-Foundry-shaped ontology platform — object types, link types and action types — with a single interactive write door, a validate/stage/publish split, branches and revisions, per-property provenance, markings, and a machine-readable register of published interfaces with conformance kits. It is also a beta, one-person, single-commit project written in Clojure. It has a genuine document-intelligence subsystem — page Markdown by OCR, layout or vision model; typed entities grounded in a line and a quote; and an LLM judge — but it does not build a paper or claim graph. Both the engineering and the caveats matter.

A BM25 probe of the local 324-work library for `ontology`, `provenance`, `knowledge graph` and `access control` returned nothing directly on ontology-oriented software development; the nearest hit was *MemOS: A Memory OS for AI System* (Zotero itemID 4633), which is adjacent to graph memory but not to this design. Sutra-OS is therefore treated here as **design prior art**, not as a paper to index.

## Executive assessment

Sutra-OS models a domain as typed objects, typed links and typed actions, written in an EDN dialect named `.ent`, installed into PostgreSQL, and served to a web UI, CLI, REST API, generated Python SDK, git and AI agents through the same rules. The checkout contains 2,764 tracked files, about 256k lines of `.clj`, 38.5k of `.cljc`, 93k of `.cljs` (including an 11.9k-line vendored Blueprint port), 9.8k of Python, and 4,853 `deftest` forms. It is a serious piece of engineering, not a sketch.

Six things in it matter for `paper-graph`:

1. **There is one interactive write door.** The object HTTP surface has no endpoint that updates an object's attributes; object changes go through `POST /api/actions` with `dry-run`, `stage` and `publish` modes. The action interpreter refuses direct `sys/*` edits outside an action.
2. **Deciding is separated from doing.** A call can be validated (nothing written), staged on a per-user staging branch, or published. Agent calls default to proposals the asker accepts or rejects; branch merges are a separate proposal with checks and a landing step.
3. **Provenance is per property, not per row.** Import writes `{entity-id, property, value, source, run-id}` records, and survivorship resolves competing sources per property. This is the closest thing in the repository to the value-level provenance this project needs.
4. **Document intelligence is quote-grounded.** Page extraction offers raw-text, OCR (Tesseract), layout-OCR and VLM strategies; entity extraction makes the model return `{value, line, quote}` for every leaf, locates the quote deterministically (exact, normalized, relocated, unlocated), rejects a value that does not occur in its quote, and re-asks once with the failures. That is close to the evidence-span validator this project needs, though it is still page/document extraction, not a claim graph.
5. **The architecture is enforced by static tests, not convention.** A purity test parses every namespace, infers pure cores from a declaration file, and fails on ambient IO/state, forbidden layer edges and upward module calls. A second test budgets every direct control write, HTTP client, world commit and capability lookup per file, with exact counts, and fails if a budget is stale. A third forbids the lower-level commit primitive outside the store. A fourth checks that every grammar key appears in the living grammar file.
6. **Interfaces are published as a register with conformance kits.** Fourteen boundaries carry a version, standards, source files, generated schema, conformance test paths and a status. Seven kits have committed results; one check (LLM structured output) is committed as failing.

**Recommendation.** Borrow the patterns, not the platform. Reimplement, in Python and against HawDB, the five things that make the graph and its writes trustworthy: action-gated promotion with validate/stage/publish, per-property provenance with an explicit survivorship policy, quote-grounded entity extraction with deterministic location, a published extraction contract with a conformance kit, and static architecture guards. Do not adopt Clojure/PostgreSQL as a dependency, do not copy AGPL code, and do not expect Sutra to supply a paper/claim graph or a stance model. A bounded next experiment is proposed in §10.

The larger caveat is maturity, not design: the public snapshot is one commit, has no CI configuration, no tags, no long-form documentation beyond a 555-line README and the self-documenting grammar, and its committed SBOM names a commit that is not in the history. The claims below are therefore claims about *code that exists at this commit*, not about a released product.

---

## 1. What the repository contains

### 1.1 Facts

| Property | Value at `ce8d4f7` |
|---|---|
| License | AGPL-3.0-or-later with a section 7 exception for linking EPL libraries (`LICENSE`); `boundaries/` specs are Apache-2.0 (`boundaries/LICENSE`) |
| Languages | Clojure 1.12, ClojureScript, Python 3 (SDK and transform runtime), shell; JDK 21 |
| Size | 2,764 tracked files; ~255.9k `.clj`, 38.5k `.cljc`, 93.1k `.cljs`, 9.8k `.py` lines; ~1,600 namespaces |
| Model files | 42 `.ent` files (4.7k lines); the reference grammar alone is 1,539 lines |
| Tests | 4,853 `deftest` forms; 4,458 in the six Kaocha suites (`tests/grammar` 765, `tests/platform` 2,264, `tests/scenario` 190, `tests/apps` 347, `tests/ontology` 790, `libs/store/test` 102) |
| Build | `deps.edn`, `tests.edn`; Kaocha; no CI configuration (no `.github/`, no GitLab/Circle/Jenkins file) |
| Runtime | Linux x86_64, JDK 21, Clojure CLI, Node.js, Python 3; embedded PostgreSQL (zonky) for the solo distribution |
| Docs | one README (555 lines); `resources/presets/grammar.ent` doubles as reference documentation; no other Markdown |
| History | exactly one commit; no tags; no releases; no changelog |
| Status | README badge says beta; the README says model language and APIs may still change |

The counts are static: `git ls-files | wc -l`, `wc -l` over the relevant extensions, and `grep -rho 'deftest'`. They say nothing about whether the tests pass.

### 1.2 Module map

| Path | Role |
|---|---|
| `libs/kernel` | Reference interpreter for actions, lowering, install and object sets; reads through Σ, writes as values |
| `libs/world` | "The world as a db value": element deltas, indexes, derivations, gates stepped by what a delta touched |
| `libs/commits` | A commit as the store keeps it, read as the world's element changes |
| `libs/service` | The one merge of two sides at touched nodes, and who may do what, each reading only its slice |
| `libs/dialect` | The language: grammar, desugaring, resolution, spec, value types |
| `libs/core` | Authoring, projections, lint, impact, apps and views shared with the frontend |
| `libs/store`, `libs/store-pg` | Store contract, semantics, scans and plans; PostgreSQL engine |
| `src/sutra` | The server: access, actions, agents, authoring, branching, builds, catalog, compute, documents, evals, governance, lineage, objects, sources, and more |
| `src/data-plane/python` | Transform runtime and extraction libraries: media/OCR, page extraction, entity extraction, LLM and interchange SDKs |
| `clients/` | Frontend (ClojureScript), CLI, language server, Python SDK, babashka tools for `change` and `enta` |
| `boundaries/` | Published interfaces, conformance kits and results, SBOM, compatibility lock (Apache-2.0) |
| `resources/presets` | The `.ent` grammar, templates, and example models (farm, household, data rights, cost center) |

The dependency graph is declared, not inferred: `cores.edn` names the pure cores, their roots or namespaces, the allowed edges and seams, the memoised caches, and the module tiers. `tests/platform/purity_test.clj` checks the declaration against the source. That is the mechanism behind §7's architecture tests.

---

## 2. The model language

### 2.1 Five authoring languages

The authoring boundary publishes five dialects, each with a generated JSON Schema under `boundaries/authoring-language/`:

| Extension | What it describes |
|---|---|
| `.ent` | The ontology: value types, objects, links, actions, lifecycles, object sets, functions, sources, datasets, projects, applications, agents |
| `.enta` | An app (pages, widgets, bindings) |
| `.entp` | A pipeline |
| `.entt` | A document template |
| `.entv` | An object view |

Only `.ent` is examined in this note. It is EDN, read with `clojure.edn/read-string`, then structurally validated by `parser.ent/validate!` against a **closed** grammar: an unknown section, entity key or mistyped field is rejected before lowering. The runtime vocabulary — source types, profiles, model IDs, object/link/action names — stays open and is resolved against a registry or the live schema. The header of `grammar.ent` states both the guarantee and its limit:

> This is a *reference* ontology-installation file: it exercises EVERY section and EVERY grammar KEY the single `.ent` door (`parser.ent`) accepts, each with an inline description. ... `unit/template-coverage-test` MECHANICALLY asserts that every key of the closed `EntFile` grammar appears here: add a grammar key without documenting it here and that test FAILS.

That is a living-spec device worth stealing: the documentation and the acceptance test are the same artifact.

An instance serves exactly one ontology, not a library of them: a top-level `:ontology` key is rejected because the database name is the ontology. The files under `resources/presets/` are therefore not a built-in domain model. `grammar.ent` is a grammar/spec reference (its cross-references are illustrative and it is not meant to be installed as-is); `template.ent` is an aircraft-maintenance template (operator, offer, work order, invoice, payment, mechanic, time booking); the `beispiele/` files cover a farm (field, buyer, harvest, lot, sale), a household (commitment, payment), data rights, a cost centre and teaser rates (loan); `tests/support/fixtures/werft/werft.ent` is the executable MRO flight-log/defect model; and `_identity/identity.ent` is a user/marking/clearance preset. None of them is a paper or claim ontology.

### 2.2 The three core constructs

```clojure
;; An object type: a typed table with a primary key, display fields,
;; constrained attributes, and read-time derived values (from farm.ent).
:field
{:name "Field" :plural "Fields" :symbol "🌾"
 :primary-key :field-id :match-keys [:field-id]
 :title :name
 :attributes
 [[:field-id :string {:visibility :hidden}]
  [:name :string {:name "Name" :required true}]
  ;; …
  [:hectares :decimal {:name "Hectares" :format {:suffix " ha" :decimals 2}}]
  ;; …
  [:cost :decimal
   {:derived {:language :clojure
              :expression [:* [:get :hectares] [:get :cost-per-hectare]]
              :inputs [:hectares :cost-per-hectare]}}]]}
```

```clojure
;; A link type: cardinality, foreign key, and a name in both directions
;; (README, "Link types").
[:sale-lot :sale :lot :many-to-one
 {:fk :lot :forward-api-name :lot :reverse-api-name :sales}]
```

```clojure
;; An action type: the only way to change a lot. Parameters are typed;
;; submission criteria are server-enforced; rules are a step tree.
:sell-hay
{:on :lot
 :params {:lot {:type :lot :required true}
          :bales {:type :integer :required true :constraints {:min 1}}
          :buyer {:type :string :required true}
          :sale-nr {:type :string :required true}}
 :submission-criteria
 [{:message "Not that many bales in the barn."
   :test {:want [:get :bales] :have [:get :lot :in-barn]}
   :pred [:<= :want :have]}]
 ;; …
 :rules [{:op :create :object-type :sale
          :values {:sale-nr [:get :sale-nr] :buyer [:get :buyer]
                   :bales [:get :bales]}}]
 :action-log true}
```

The action excerpt above is condensed from the README's farm walkthrough; the full farm file is `resources/presets/beispiele/farm.ent`, and the action grammar with every key is `resources/presets/grammar.ent` around line 888.

### 2.3 What else the model expresses

| Construct | Meaning | Enforcement point |
|---|---|---|
| `:value-types` | Named scalar type with constraints and display format (currency, IBAN, EAN, regex, ranges) | Validation at write and install |
| `:shared-properties` / `:interfaces` | Reusable property lists; an object type implements interfaces and inherits properties | Install-time resolution; `effective-interface` in `world/gate.cljc` |
| `:lifecycle` | `:property`, ordered `:states`, `:transitions` naming actions | Install validates that transitions name real actions and states; a state-changing action is only allowed from its `:from` states |
| `:derived` | Read-time value: `:rollup` over a link, or a `:language :clojure` expression with declared `:inputs` | Computed by the server; not stored |
| `:object-sets` | Named predicate/selection over a type, with ordering and limit | Query planning |
| `:sources` / `:datasets` / `:transforms` | Where data comes from, how it is shaped, and what writes object types | Builds; direct writes outside actions (see §4.4) |
| `:agents` | Model, source, temperature, turn cap, tool kinds, `:writes :propose` or `:apply` | Agent runtime |

Two authoring details are small but telling. API names must be ASCII; display names need not be, and the refusal names the transliteration (`prüfer` → `pruefer`). And the template is written in German and ordered by the OOUX design sequence — objects, relationships, actions, formats, provenance, project — with each comment explaining the decision the key makes (`resources/presets/template.ent`). The template has its own test that it is structurally valid, lint-clean, OOUX-clean and ASCII-named (`tests/platform/template_coverage_test.clj`).

**Assessment.** `.ent` is a real domain DSL with a parser, a closed structural grammar, lowering passes, install-time lint and a coverage test. It is considerably richer than a JSON Schema for the same domain, and considerably more expensive to adopt: it is EDN, its semantics live in Clojure, and its install path is tied to the platform. For `paper-graph`, the transferable idea is the *discipline* — one declarative model, one validator, one lint, one template that is guaranteed to exercise the grammar — not the file format.

---

## 3. Execution semantics

### 3.1 The world is a value

`libs/world` is described in `cores.edn` as "the world as a db value, element deltas, indexes, derivations and gates stepped by what a delta touched." The implementation matches: `db.cljc` builds a map with `:ents`, `:idx`, `:views`, `:view-rows` and `:delta`; `with` returns `{:db-before :db-after :delta}`; gates are selected by the nodes a delta touched, not by a global re-scan.

```clojure
(defn with
  [db changes]
  (let [changes (d/completed (:ents db) (d/squash changes))
        ;; …
        W1      (d/apply-to (:ents db) changes)
        db1     (assoc db
                       :ents W1
                       :idx (ix/step (:idx db) (:ents db) W1 (d/nodes entries))
                       :delta (into (:delta db) changes))]
    {:db-before db
     :db-after  (step-views (:view-rows db) db db1 entries)
     :delta     changes}))
```

This is the clearest architectural idea in the repository: the semantic layer is a pure library over values, and the server is an effect shell around it. The excerpt elides two local bindings; the purity test in §7 is what keeps the shell from leaking back in.

### 3.2 The kernel interprets actions

`libs/kernel` is "the interpreter of actions, lowering, install §8 and object sets. Reads through Σ, writes as values." The interpreter is a multimethod over step operations (`:create`, `:update`, `:delete-object`, `:create-link`, `:compute-on`, `:notify`, `:invoke-webhook`, `:for-each`, …), with explicit modes:

- `:propose` — effect steps are collected into `:proposals`; a non-effect `:call` is refused with `:propose-refused`; ontology edits are refused with "Only Actions can be proposed for review".
- `:collect` — an effect step is evaluated and appended to `:collected`, and blocked if its params are invalid.
- otherwise — the step runs and its bindings flow to the next step.

Direct `sys/*` access is denied unless the interpreter is already inside an action (`action.clj`, `resolve-action!`). That is the semantic counterpart of the HTTP write door: even if a caller reaches the interpreter, it cannot bypass the action wrapper for system writes.

### 3.3 Commits, branches and revisions

`libs/commits` reads a commit as element changes, and `libs/commits/branches.clj` reconstructs the two branch states; `libs/service/merge.cljc` performs the one merge at the nodes their deltas touched. At the surface:

- branch-aware endpoints are marked `:branch? true` in the path table;
- a successful action response carries `result.commit.basis.rev`, which the SDK exposes as `revision`;
- `GET /api/at/:commit/objects/:type` reads objects as they stood at that commit, and `client.at(commit)` wraps it;
- branch merge proposals preview the schema merge, run checks, and land only when allowed (`branching/proposals.clj`).

The README's "last Tuesday's state is still there to read" is therefore supported: revisions are commits, and the read path accepts one. Whether the retention policy keeps every revision forever is a separate configuration question this note did not verify.

---

## 4. The write path

### 4.1 The object surface has no update endpoint

`libs/core/src/sutra/types/paths/objects.cljc` declares reads and query support: `/api/objects/:type` (GET), `/api/objects/:type/knn` (GET), `/api/object-counts`, `/api/instances`, `/api/links`, `/api/ontology/query`, `/api/ontology/series`. The endpoints that mutate anything are analysis scratch space — temporary object sets and explorations — not domain objects.

Object edits are in `paths/actions.cljc`:

```clojure
:actions {:url "/api/actions"
          :methods {:post {:effects writes :requires c-action}}
          :service :actions
          :branch? true
          :query {:dry-run :bool, :publish :bool, :stage :bool, :return-edits :bool}
          :request :sutra.foundation.contracts/action-wire
          :response :sutra.foundation.contracts/action-response}
```

That single endpoint plus its query flags is the "one door" for interactive object writes. The first three modes below are call modes; the fourth is the branch-level review path that wraps a series of changes.

### 4.2 Modes of a write

| Mode | Surface | What is written | Typical use |
|---|---|---|---|
| Validate only | `POST /api/actions?dry-run=true`; SDK `ActionConfig(mode=VALIDATE_ONLY)` | Nothing; returns validation and computed edits | "Would this work?", agent dry run |
| Staged | `?stage=true`; SDK `publish=False` | Rows on the caller's staging branch; `.staged` is true | Agent proposal, try-it-on-a-copy |
| Published | `?publish=true` | Action log entry plus edits, in one transaction | Normal UI/CLI/SDK call |
| Branch proposal | `POST /api/proposals` (branching) | A proposal over a source branch, with preview and checks; `land!` merges | Model and data changes that need review |

The staging path is not a flag on a write; it is a different store view. `src/sutra/actions/staged.clj` computes a per-user staging branch (`staging-branch-of branch user`), plans each call against the staging schema, and records rows with provenance. `plan-stage` produces the rows; `commit-staged!` and `publish!` apply them. `publish!` partitions the rows, skips objects that vanished, strips derived attributes before writing, and writes the action-log entry through the same index call — which is why an action log entry and its edits are atomic (the grammar states the same invariant around `:action-log`).

### 4.3 Two different "proposals"

The word is used for two distinct mechanisms, and the README's "deciding is not doing" depends on both:

1. **Agent/action proposals.** `src/sutra/actions/proposals.clj` stores a proposed action with a digest of its canonicalized arguments, the object versions it read, an optional reason and decision log, and a status. An identical proposal is not duplicated. The asker accepts or rejects it in the thread. This is what `:writes :propose` means.
2. **Branch proposals.** `src/sutra/branching/proposals.clj` opens a reviewable change from a source branch to a target branch, attaches a schema-merge preview, CI-style checks and approvals, and lands it. This is the model/data review path.

Both are called "proposals" in the UI and README; they have different tables, lifecycles and approvers. A research write-up that blurs them will misdescribe the system.

### 4.4 The qualification: ingestion and internal writes

The "actions are the only write path" claim is true for the interactive and agent surface, not for the whole system:

- Datasets and transforms write object types directly during builds; that is how data arrives.
- The effect-budget test allow-lists 37 files with direct control writes (locks, queues, run logs, agent threads, canvas graphs, proposals, health checks, and others), each with an exact count.
- `tests/grammar/control_commit_door_test.clj` forbids `ctl/plan!` outside `src/sutra/store`, forcing modules through `ctl/commit!`; it does not forbid every write, it forces one commit door.

The precise claim is: **every user-visible object mutation goes through a typed action; machine ingestion has its own declared path; internal state has one commit door and an audited budget.** That is a stronger and more honest formulation than "only write path".

---

## 5. Provenance, lineage and document intelligence

### 5.1 Per-property provenance

When an import materializes an object, `record-provenance!` (`src/sutra/builds/import/materialize.clj`) writes one row per non-nil property:

```clojure
{:entity-id entity-id :property prop :value v :source source :run-id run-id}
```

`src/sutra/projects/kompass.clj` stores these in `:entity-provenance`; `latest-provenance-by-source` groups by `[property source]` and keeps the highest sequence number, and a retraction writes the sentinel `ENTITY-PROVENANCE-RETRACTED` rather than deleting history. `consolidate-rows` in the importer chooses a surviving value per property according to a survivorship policy (`:order`, or `:by-property` source ranks).

This is object-property provenance: which source and run supplied the current value of an attribute. It is not text provenance, and it is not per-value history — the latest row per `[property source]` is what is read back.

### 5.2 Schema lineage

`src/sutra/lineage/facts.clj` builds a declared lineage graph from the installed schema: datasource → dataset → transform → object type, plus action/function/application references with relations `:input`, `:output`, `:reads`, `:edits`, `:calls`, `:uses`, `:shows`. `sutra lineage <node>` renders it. This is the graph the README's "every number has a past" points at.

It is **structural** lineage: it says that this object type is written by that transform from that dataset, not that this object's current `margin` came from row 42 of statement 7. The importer's provenance records add the missing per-property source attribution, but not a row or cell locator.

### 5.3 The action log and revisions

Every action can write an action log (`:action-log true`), and approved changes are kept as revisions. The log records the actor and the action; the revision stores the resulting state. For an audit question ("who changed this, through what action, in which commit?"), the machinery is present.

### 5.4 What is missing for a paper graph

Sutra has no first-class **EvidenceSpan** object in the ontology: no general attachment/page/offset locator, no stance (`supports|refutes|qualifies`), no distinction between a proposition and an attribution, and no negative-evidence object. A `Claim` could be modelled as an object type with `quote`, `page`, `start`, `end` attributes, and an action could enforce non-emptiness, but the platform would not, by itself, tie a claim to a validated text locator.

The gap is narrower than that sentence suggests. The entity extractor in §5.5 already requires a line id and a literal quote for every leaf value, locates the quote deterministically, and rejects a value that does not appear in it. Those are extraction records with page/block/box coordinates, not graph nodes with stance; they are the raw material for an `EvidenceSpan` object, not a substitute for one.

### 5.5 Document intelligence

Document Intelligence is a first-class app in the checkout, with its own endpoint family (`/api/doc-intel/...`) and a Python extraction runtime under `src/data-plane/python/`.

- **Page extraction.** `sutra_media.py` reads electronic PDFs with `pdfplumber`, runs Tesseract OCR (the `tesseract` binary, 300 DPI, language packs, 180 s timeout), builds layout blocks with normalized bounding boxes, reconstructs tables as Markdown or HTML, and renders page images for vision models. `sutra_extraction.py` exposes four page strategies: `raw-text`, `ocr`, `layout-ocr` and `vlm` (a vision model converts the page image to GitHub-flavored Markdown).
- **Quality scoring.** A judge prompt scores an extraction 1–5 on `text`, `tables`, `headings`, `lists`, `reading_order` and `completeness`, with one or two sentences naming the worst defects. The app shows the page, the extraction and those scores.
- **Entity extraction.** `sutra_entities.py` takes a typed entity schema (`string`, `integer`, `double`, `date`, `boolean`, `array`, `struct`), asks the model for a tool call whose every leaf is `{value, line, quote}`, rebuilds page lines from the layout blocks, and locates the quote. A value is `exact`, `normalized`, `relocated` or `unlocated`; `value_in_quote` checks numbers, dates and strings; a value not in its quote becomes `value_not_in_quote`; the model is re-asked once with the failed checks. The record carries page, block and bounding box.
- **Fingerprints and deployment.** A strategy is fingerprinted over strategy, mode, languages, model, preprocess, prompt and max tokens (entities add the schema). Page rows are keyed `media_reference#page` and carry the strategy id; an incremental run carries previous rows only when the fingerprint (and, for entities, the input hash) matches. `/api/doc-intel/extract` extracts one page and writes nothing; `/api/doc-intel/deploy` commits the strategy as a Python transform template onto a branch of a code repository; the transform writes a page-level dataset.
- **Evaluation.** `/api/doc-intel/evaluate` judges one page's Markdown against the page image; the entity strategy reads the deployed page dataset, grouping pages into documents with `documents_of`.

This is document intelligence, not a paper pipeline. It has no section model, no bibliography or citation-context parser, no claim/assertion distinction and no built-in library connector; the OCR default language is German (`deu`) and the entity schema is supplied per call. The quote-grounding mechanics are nevertheless directly relevant: they are a compact working implementation of the exact/normalized/unlocated discipline the paper-graph research doc specifies, including a re-ask loop and numeric/date-in-quote checks.

**Assessment.** The per-property provenance model is directly reusable: it is the right shape for recording which extraction arm, run, model and prompt produced each field of a claim. The entity extractor's `{value, line, quote}` records are a second reusable piece: a working deterministic locator with exact, normalized, relocated and unlocated outcomes and a re-ask loop. The schema lineage graph is useful for explaining the pipeline, but it is not evidence. What the paper graph still has to add is the graph-level `EvidenceSpan` (attachment/page/offset, stance, minimality) and the claim/assertion split; Sutra supplies the extraction and provenance mechanics around them, not the semantics.

---

## 6. Access and secrets

### 6.1 Markings and projects

Markings are mandatory-access-control labels on object types, datasets, transforms and sources. A project declares `allowed-markings` and `forbidden-markings`; a commit gate (`libs/world/src/sutra/world/gate/marking.cljc`) checks the marked resources a delta touches, and refuses a marking that is forbidden or not allowed. A second gate fact requires categories of markings on projects (`:required-categories`). Reads are scoped through `sutra.store.access`; markings are stored in a `_markings` column and projected out of ordinary row values.

The README's stronger claim — that the server enforces this on lists, filters, counts and totals, so one person's data cannot reveal what everyone else was offered — is a design claim backed by code and tests (`tests/platform/asvs_authz_test.clj`, `organizations_test.clj`, `read_path_parity.clj`, `grants_meet_test.clj`), but this note did not execute those tests.

### 6.2 Project grants

Access is not only markings: projects carry grants for read, act and invoke (README), and endpoints declare required capabilities in the path table (for example `:requires {:op :read, :on [:object-types :type]}`). The path table is also the source of the published OpenAPI document, so the permission model and the API contract are generated from the same declaration.

### 6.3 The egress agent

Credentials are held by a separate process. `bin/start.sh` starts `clojure -M:dev -m sutra.sources.egress` on its own port and points it at an RSA key file (`SUTRA_EGRESS_KEYFILE`); `src/sutra/foundation/egress.clj` calls it by URL with a per-agent token derived from `SUTRA_EGRESS_MASTER` or a token file. The server's transform and LLM code does not receive the external credential; it asks the agent. That is a clean pattern for a paper pipeline that must call an embedding or LLM endpoint: keep the key in a separate process and give the extraction code a capability, not a secret.

### 6.4 Deployment defaults

The plain server binds `0.0.0.0` by default (`src/sutra/serve/server.clj`: `(env/env "SUTRA_HTTP_BIND" "0.0.0.0")`), and `bin/start.sh` starts the development backend with `SUTRA_ALLOW_ANON=1`. The README warns about both. The solo distribution sets `SUTRA_HTTP_BIND=127.0.0.1`. This is a deployment footgun rather than a vulnerability claim, but it is the kind of default that matters if the platform is ever exposed.

---

## 7. Correctness machinery

This is the most transferable part of the repository, and the part least visible in the README.

### 7.1 Static guards

| Guard | File | What it enforces |
|---|---|---|
| Purity and layering | `tests/platform/purity_test.clj` | Parses every namespace under the declared roots with rewrite-clj; infers pure cores from `cores.edn`; forbids ambient time/env/random, file/console IO, global state, forbidden libraries (JDBC, shell, HTTP clients) in pure cores; validates layer order, module tiers and facade access; fails on stale allow-list entries |
| The world core holds no state | same file (`the-kern-holds-no-state`) | `libs/world/src` may not use atoms, volatiles, refs, agents, `set!`, `alter-var-root`, dynamic vars or resolution tricks |
| Effect budgets | `tests/grammar/edge_effects_budget_test.clj` | Exact per-file budgets for direct control writes (`ctl/upsert!`, `ctl/insert!`, `ctl/update!`, `ctl/swap!`, `ctl/delete!`), HTTP clients, `world/commit!`, and capability lookups; a second test fails if a budget is stale (spends fewer than recorded) |
| One commit door | `tests/grammar/control_commit_door_test.clj` | Modules outside `src/sutra/store` may not call `ctl/plan!`, `ctl/plan-as!` or `served`; they go through `ctl/commit!` |
| Grammar coverage | `tests/platform/template_coverage_test.clj` | Every closed `EntFile` grammar key appears in `grammar.ent`; the template passes the real read-ent door, lint, OOUX rules and ASCII API-name checks |
| Boundaries | `tests/grammar/boundaries_test.clj` (and `boundary_compat_test.clj`) | The register's sources, generated artifacts, versions and conformance pointers stay consistent |
| Scenario | `tests/scenario/` (27 files, 190 `deftest` forms) | An aircraft-maintenance world, with property-based state-machine tests and numbered requirements (`§10.4 a preview writes nothing`) |
| Conformance | `boundaries/conformance` | Seven executable kits against a running instance or a local process |

Two properties of these tests deserve emphasis. First, they are **static**: the purity and budget tests do not need the server to run, so they can fail a pull request without a database. Second, they are **self-policing**: the budget test fails on stale budgets, and the layer test fails on stale allow-list entries. A declaration that no longer matches the code is itself an error. This is a stronger form of architecture enforcement than a review checklist.

### 7.2 The boundary register

`resources/sutra/boundaries.edn` declares 14 boundaries, each with a number, title, class (external or component), owner, version, prose description, spec form (OpenAPI, JSON Schema, Malli, prose), source files, generated artifacts, standards, conformance test paths, covered URL prefixes or capabilities, a status map (`:spec`, `:versioned`, `:done`, `:conformance`), and a rights field.

The standards list is concrete: OpenAPI 3.1, RFC 9110, JSON Schema 2020-12, OAuth 2.0/2.1, OIDC, PKCE, JWK/JWT, RFC 7662/7009, TOTP, LDAP, SPNEGO, CloudEvents 1.0, OpenMetrics 1.0, SMTP/STARTTLS, Git smart HTTP v2, Parquet/Arrow IPC, and others. The register is not a diagram of aspirations; it names the standard each boundary claims to implement and the test that checks it.

### 7.3 The conformance results

`boundaries/conformance-results/` commits seven summaries and JUnit XML files. At this commit:

| Kit | Pass | Fail | Skip |
|---|---:|---:|---:|
| `http-api` | 7 | 0 | 0 |
| `identity` | 9 | 0 | 1 |
| `git` | 10 | 0 | 1 |
| `events` | 2 | 0 | 0 |
| `transform-runtime` | 8 | 0 | 0 |
| `platform-sdk` | 4 | 0 | 0 |
| `llm` | 3 | **1** | 0 |

The failing LLM check is `llm.structured-value`: the structured output arrived as a `final_answer` tool call, but the value was `nil` where an object was expected (`"value: : expected object, got nil"`). The failure is committed, not hidden. For a beta project, that is a good honesty signal; for a consumer, it means the "all boundaries conform" reading is false at this commit.

### 7.4 What the test suite does and does not establish

4,853 `deftest` forms is a large suite by any measure. The scenario suite is the most interesting: it installs an ontology, drives actions over HTTP, and checks numbered requirements, including property-based tests that compare the action implementation against a hand-written state machine. That is the right shape for a system whose central claim is "the rules are enforced by the server, the same way for every client."

But none of it was run here. There is no JVM on this host, the project targets Linux x86_64, and the solo runtime embeds a platform-specific PostgreSQL. The correct statement is: **the suite exists, is extensive, and is designed well; its passing status is not established by this note.**

---

## 8. Claims versus code

| README claim (line) | What the code supports | Qualification |
|---|---|---|
| "Actions are the only write path" (212) | No object-attribute update endpoint; object writes are `POST /api/actions` with modes; `sys/*` denied outside an action | True for the interactive/agent surface. Ingestion via datasets/transforms writes objects; 37 files have allow-listed direct control writes. The honest formulation is in §4.4. |
| "A test fails if the part that decides ever does anything but decide" (249) | `purity_test.clj` + `the-kern-holds-no-state` | Static, namespace-level. It forbids ambient IO/state in declared cores; it does not prove runtime purity of the whole decision path. |
| "Every number has a past" (258) | Per-property `entity-provenance` with source/run; schema lineage graph | No text-span evidence attached to provenance; no per-value history, no row/cell locator. "Past" means source attribution plus declared pipeline structure. |
| "AI agents only propose" (hero image, 27; default documented at 265–268) | Default `:writes :propose`; agent-proposals table with accept/reject | `:writes :apply` exists and runs under the asker's rights (`grammar.ent` 1340–1342). It is a default, not an invariant. |
| "Access control ... enforced ... on every read" (275–280) | Markings gate, project grants, `_markings` column, access scope, many tests | Code and tests present; read-path behavior not executed here. |
| "Model and data live on branches" (270) | Branch-aware endpoints, staging branches, merge proposals, `/api/at/:commit` | Structurally verified. Retention policy not verified. |
| "Credentials held by a separate egress agent" (286–287) | `sutra.sources.egress` as its own process; token/URL client in `foundation/egress.clj` | Structurally verified; credential isolation not audited. |
| "The model as the SDK" (289) | Generated Python SDK, endpoint table, typed actions, pyright-checked examples | SDK presence verified; generation and pyright run not performed. |
| "Nothing happens until you say so" (227) | dry-run/stage/publish; validate-only; staged rows; proposals | Structurally verified. |
| Beta status (15) | One commit, no tags, no CI, no docs beyond README, one failing conformance check | Consistent. |

---

## 9. Risks and limitations

### 9.1 The snapshot problem

The repository has exactly one commit (`ce8d4f7`, 2026-11-17). There are no tags, no releases, no changelog, no CI configuration and no other Markdown. The committed SBOM records version `791aa1001189065dc31c8e9b722f5c4d023055a1` (`boundaries/sbom/sutra.cdx.json`), which is not an object in this repository's history and dates to 2026-11-11, six days before the public commit; the conformance results carry no date at all. **Assessment:** the public repository is an import of a longer private history, not a development history a reviewer can audit. That does not make the code wrong, but it removes the usual signals — review trail, incremental tests, release notes — and makes the test suite the only quality evidence.

### 9.2 Bus factor and authorship

The README says "Sutra-OS is a one-person project so far" and "Maintainers wanted", and "Claude Code wrote all of the code" (543). The code is consistent in style and heavily commented, and the static tests are unusually disciplined. But at this scale, generated code is a review burden, and a single maintainer is a continuity risk. The checkout contains no issue history, and the GitHub API was rate-limited when this note was written, so defect-handling practice could not be judged.

### 9.3 Beta and API churn

The README says the model language and APIs may still change; boundary versions are 1.x/2.0, and several status maps say `:spec :partial` or `:conformance :partial`. A consumer should pin a commit and run the conformance kits against it, not assume compatibility.

### 9.4 Stack mismatch

Clojure 1.12, JDK 21, PostgreSQL, Linux x86_64, Node and Python 3 — against `paper-graph`'s Python 3.13, HawDB and macOS development. Adopting Sutra as a backend would be a rewrite of the storage and query layer. It has generic connectors (CSV, REST, PostgreSQL, MySQL, SQL Server, IMAP, SSH) and a transform runtime, but no Zotero or HawDB connector; feeding this library in would be new work, not configuration. The platform is not a library that can be imported.

### 9.5 License

AGPL-3.0-or-later with a section 7 exception for linking EPL libraries. Running a modified Sutra as a network service triggers the AGPL's source-offer obligation. The `boundaries/` specs are Apache-2.0, but the implementation is not. **Assessment:** reading the code for design is fine; copying code into `paper-graph` is not without a licensing decision. The patterns in §10 should be reimplemented, not transcribed.

### 9.6 Missing capabilities relevant here

- **Document extraction exists; a paper pipeline does not.** `src/sutra/llm/media.clj` reads PDF text or renders pages for an LLM attachment, and the Document Intelligence runtime (`sutra_media.py`, `sutra_extraction.py`, `sutra_entities.py`) goes further: pdfplumber for electronic text, Tesseract OCR, layout blocks and tables, VLM page Markdown, page-level datasets with strategy fingerprints, and typed entity extraction grounded in line and quote. What is absent is the paper-specific layer: section structure, bibliography and citation contexts, a claim/assertion model, and a Zotero/library connector.
- **No first-class EvidenceSpan in the ontology, no stance, no claim/assertion distinction, no negative evidence.** The entity extractor's line/quote records are extraction rows with page/block/box coordinates, not graph nodes that a claim must cite; the platform does not enforce a claim-level evidence link by construction.
- **Semantic search is exact scan over object vectors** (`objects/knn.clj`: cosine/dot/L2 over fetched pages), with the embedding call relayed through the egress agent; it is not an approximate index.
- **The bundled LLM source plugin is xAI-specific** (`plugins/llm/src/sutra/llm/xai.clj`), but the LLM wire layer has built-in OpenAI-chat (default) and Anthropic wires and a host allow-list including OpenAI, Anthropic, Mistral and OpenRouter (`src/sutra/llm/wire.clj`). The committed LLM conformance failure was run against a local endpoint on port 11434 (the Ollama default), so it is evidence about that endpoint, not about every provider.
- The UI is large (71k lines of frontend source plus a vendored Blueprint port); it is not a small dependency.

### 9.7 Security defaults

The plain server binds `0.0.0.0`; the dev script enables anonymous access. The README warns, and the solo distribution binds loopback. This is acceptable for a local-first beta but must not be mistaken for a hardened default.

---

## 10. What transfers to the paper graph

### 10.1 Mapping

| `paper-graph` need | Sutra mechanism | What to take | What to leave |
|---|---|---|---|
| Claim promotion and review | Action types; `:submission-criteria`; validate/stage/publish | Validate/stage/publish call model; criteria as data; one transaction for claim + evidence + log | The Clojure DSL |
| Multi-arm extraction | `entity-provenance`; survivorship per property | `{value, source, run, model, prompt}` per property; explicit per-property merge; keep all arms | Latest-only retraction as the sole history |
| Page and entity extraction | Document Intelligence; `sutra_entities.py` | Strategy fingerprints, page rows, `{value, line, quote}` with exact/normalized/relocated/unlocated, six-criterion judge | Page-level model, German OCR default, no paper structure |
| Experiment isolation | Branches, revisions, `/api/at/:commit` | One branch per arm; main untouched; read at a revision | Whole-world copy cost at larger scale |
| Published contract | Boundary register; conformance kits; committed results | Manifest with version, schema, standards, kit and results; commit failures too | OpenAPI/JVM tooling |
| Architecture discipline | Purity test; effect budgets; one commit door | Pure validators; budgeted LLM/DB effects; single write door | Rewriting the Clojure analyzer; use Python AST/import-linter |
| Annotation manual | `grammar.ent` + coverage test + linted template | Every key/relation documented; test coverage; lint examples | EDN |
| Typed client | Generated Python SDK from schema | Generated types for claims/evidence; pyright in CI | Not needed at 324 works |
| Privacy | Markings, projects, grants | Per-paper/source markings; read filtering in the query layer | Full ABAC |

### 10.2 Six borrowings, in order

**B1. Claim actions with validate, stage and publish modes.** Model claim promotion as a small action set: `propose_claim`, `attach_evidence`, `merge_concepts`, `retire_claim`. Each has typed parameters and data-driven submission criteria — for example, "a claim may be promoted only if it has at least one evidence span whose normalized quote matches the stored chunk text, and the span's paper is in the library." Expose `dry_run`, `stage` and `publish` in the CLI and API. Write the claim, its evidence links and the action log in one transaction. This is the highest-value governance transfer: it turns review from a convention into a server-enforced gate.

**B2. Property-level provenance and survivorship.** For the round-1 extraction arms, store one row per `(claim, property, arm/run/model)` rather than overwriting. Keep all arms for evaluation; apply an explicit per-property survivorship policy only when building the *reviewed* view. Sutra's `consolidate-rows` is a concrete precedent: the policy is data (`:order`, `:by-property`), not code. The paper-graph evaluation needs the unmerged arm outputs, so the review view must be a projection, not the storage.

**B3. Quote-grounded extraction with deterministic location.** Adopt the `sutra_entities.py` contract for claims and their evidence: every extracted leaf is asked for as `{value, line, quote}` and stored with a `line_id`; the validator normalizes (casefold, strip punctuation), locates the quote as exact, normalized or relocated, flags a value not present in its quote, and re-asks once with the failed checks. Its numeric and date checks catch 2.3 versus 1.3 and date-format drift, and its records carry page, block and box coordinates for the reviewer. The paper graph still needs its own `EvidenceSpan` node and minimality rule, but the locator and re-ask loop are already written in 371 Python lines and can be reimplemented directly.

**B4. A boundary register for the extraction contract.** Declare the chunk-extraction schema, the claim schema, the evidence-span schema, the action API and the SDK as numbered boundaries, each with a version, a source of truth, a generated artifact and a conformance test. Commit the conformance results, including failures. This directly addresses the "extraction contract" gap in the existing research doc: the schema is currently prose plus JSON Schema; a register would make it versioned and testable.

**B5. Static architecture guards.** The validators that matter (quote substring, page range, span minimality, domain/range, direction, stance coherence) should be pure functions over values, with no network or database access. Enforce that with a Python AST test (no `requests`, `httpx`, `psycopg`, `datetime.now` in the validators package), and give the effectful modules an explicit budget for LLM and DB calls. Add a single write-door test: only the claim-action module may insert into the claim/evidence tables. The Sutra tests show how to make the budget self-policing — a stale allowance is a failure.

**B6. A living annotation manual.** The grammar-coverage test is the best small idea in the repository. The paper-graph has an annotation manual and a JSON Schema in the research doc; a test should assert that every schema key, relation type and negative distinction in the manual is exercised by at least one example, and that the examples validate. This prevents the manual and the extractor from drifting apart, which is exactly the failure mode the DeepSeek critique warns about (prohibitions stated but not operationalized).

### 10.3 What not to adopt

- The Clojure/PostgreSQL platform, the embedded-Postgres solo distribution, the Linux-only packaging.
- The generated UI, the vendored Blueprint port, and the document-authoring stack.
- AGPL code, unless the project decides to take on the license.
- The full agent runtime. The *proposal* idea is useful; the agent execution loop, tool catalogue and prompt store are a separate product.
- The "everything is an object type" temptation. Claims need a graph-level text-span primitive and a stance model that Sutra does not have; modeling those as ordinary attributes would lose exactly the guarantees the research doc requires.

### 10.4 If one experiment is run next

A bounded, non-LLM spike, one development paper, five hand-authored claims:

1. Add a `claim_actions` module with `propose_claim` and `promote_claim`, validate-only and staged modes, and one submission criterion (evidence span required; normalized quote must match the chunk).
2. Add `extraction_provenance` rows for two fake arms and one conflict, with an explicit per-property policy.
3. Run the existing deterministic validators, including a `sutra_entities`-style quote locator, on the five claims and record which fail.
4. Measure reviewer actions and lines of code against the current manual path.

Gate: if the staged action API does not reduce reviewer ambiguity or catch at least one invalid claim that the manual path would have accepted, the machinery is not worth the complexity. If it does, wire it into the round-1 harness before the extraction arms run.

A second, optional spike belongs on a Linux host: encode six object types and eight relations of the v0 PL ontology as `.ent`, install it in a disposable instance, and record every grammar/lint refusal. That tests the *expressive* fit; it is not needed to justify B1–B6.

---

## 11. Unverified claims and non-claims

- **No execution.** The JVM, Clojure CLI and PostgreSQL runtime were not available on this host; the project targets Linux x86_64 and embeds a platform-specific PostgreSQL. Nothing here is a test result, a performance measurement or a security assessment.
- **No exhaustive code review.** The server has ~500 source files and ~94k lines, and the Python data plane adds the extraction and transform runtime; this note read the semantic core, the path tables, the action/staging path, provenance, lineage, access gates, the Document Intelligence runtime, the boundary register and representative tests. Other modules were listed, not audited.
- **Test counts are static.** 4,853 `deftest` forms were counted with `grep`; they were not executed, and some may be conditional or integration-only.
- **GitHub metadata was not read.** The unauthenticated API returned HTTP 403 (rate limit, 0 remaining), so stars, issues, creation date and default-branch metadata are not reported. The clone and commit metadata are authoritative here.
- **The SBOM/history inference is an inference.** The SBOM's `791aa100…` is not in the repository's object database; the conclusion that the public commit is an import is the most likely explanation, not a proven fact.
- **License analysis is not legal advice.** The AGPL section 7 exception and the Apache-2.0 boundary specs were read; their obligations for a specific deployment were not.
- **Feature absence is targeted, not exhaustive.** "No paper-specific citation parsing", "no first-class EvidenceSpan", "the bundled LLM source plugin is xAI-specific" are based on the modules and searches described; a feature could exist under a name this note did not search. The first pass of this note also missed the entire Document Intelligence subsystem; a later targeted search found it. That is the clearest evidence for why these are scoped as searches rather than proofs.
- **The extraction paths were read, not run.** OCR needs the `tesseract` binary and language packs, the VLM path needs a model, and `pdfplumber` must be installed in the transform sandbox. Their behavior is described from code and tests, not measured.
- **The local library probe is a BM25 probe, not a literature review.** It used the current 324-work index and five queries; it does not establish that no relevant work exists.

---

## 12. Sources

### Repository

- [`0nelight/sutra-os`](https://github.com/0nelight/sutra-os), commit `ce8d4f7819f0b6ba9eff7ab47d99c39e4e89f9bd`, 2026-11-17.
- `README.md` — motivation, model walkthrough, operational notes, license, authorship claim.
- `deps.edn`, `tests.edn`, `cores.edn` — build, test suites, architecture declaration.
- `libs/world/src/sutra/world/db.cljc`, `gate.cljc`, `gate/marking.cljc` — world as value, commit gates, markings.
- `libs/kernel/src/sutra/ref/kernel.clj`, `action.clj` — action interpreter, modes, `sys/*` denial.
- `libs/commits/src/sutra/commits/branches.clj`, `libs/service/src/sutra/service/merge.cljc` — branch state reconstruction and the three-way merge at touched nodes.
- `libs/core/src/sutra/types/paths/objects.cljc`, `actions.cljc` — endpoint tables and the write door.
- `src/sutra/actions/staged.clj`, `publish.clj`, `proposals.clj` — staging, publishing, agent proposals.
- `src/sutra/branching/proposals.clj` — branch proposals and landing.
- `src/sutra/builds/import/materialize.clj`, `src/sutra/projects/kompass.clj` — per-property provenance and survivorship.
- `src/sutra/lineage/facts.clj` — schema lineage.
- `src/sutra/llm/media.clj`, `src/sutra/llm/wire.clj`, `plugins/llm/src/sutra/llm/xai.clj` — PDF/attachment path, provider wires, bundled xAI source plugin.
- `src/data-plane/python/sutra_media.py`, `sutra_extraction.py`, `sutra_entities.py` — OCR/layout/page extraction, extraction strategies, quote-grounded entity extraction.
- `src/sutra/evals/extraction.clj`, `src/sutra/evals/http/extraction.clj`, `libs/core/src/sutra/types/paths/evals.cljc` — Document Intelligence API, deployment templates and page/evaluation endpoints.
- `clients/frontend/src/sutra_ui/apps/doc_intel/`, `tests/platform/doc_intel_deploy_test.clj` — Document Intelligence UI and deployment tests.
- `src/sutra/store/access/*.clj` — read scoping and the markings column.
- `src/sutra/foundation/egress.clj`, `bin/start.sh` — egress agent.
- `resources/sutra/boundaries.edn` — 14 boundaries; `boundaries/conformance-results/*.summary.json` — committed results.
- `tests/platform/purity_test.clj`, `tests/grammar/edge_effects_budget_test.clj`, `tests/grammar/control_commit_door_test.clj`, `tests/platform/template_coverage_test.clj` — static guards.
- `tests/scenario/aviation_gesetze_test.clj` — property-based state-machine checks against the installed ontology.
- `resources/presets/grammar.ent`, `template.ent`, `beispiele/farm.ent` — model language and examples.
- `boundaries/sbom/sutra.cdx.json` — SBOM and its commit reference.
- `LICENSE`, `boundaries/LICENSE` — AGPL-3.0-or-later with section 7 exception; Apache-2.0 boundary specs.

### Local

- `analysis/llm-document-graph-research.md` — extraction, schema, evidence and evaluation design.
- `analysis/deepseek-ontology-critique.md` — ontology semantics critique.
- `experiments/ontology-extraction/round-001-plan.md` — the three-arm ontology-specificity experiment.
- `../README.md` — the library and pipeline this note is written for.

### Library probe

- `uv run paper-graph search 'ontology' --mode bm25 --top-k 5` returned `[]`.
- `uv run paper-graph search 'provenance' ...` and `'knowledge graph' ...` returned *MemOS: A Memory OS for AI System* (itemID 4633) as the nearest hit; no ontology-oriented software work was found.
