# 0002 — Docs precedence and one statement per fact

- **Status:** accepted
- **Date:** 2026-11-19
- **Owner:** emliunix

## Context

The W1 dependency graph in `worklog/2026-11-19/todo.md` assigns documentation work in parallel. Without explicit authority, authors can produce conflicting definitions, status claims, and procedures. `paper-graph/knowledge-system/CONSTITUTION.md` establishes precedence and one owner per subject, treating duplicated facts as defects. Duplication also affects claim integrity: L08 limits conclusions from pre-verdict artifacts, and L13 rejects uncontrolled promotion of process narrative. A summary helps navigation, but a second normative copy obscures which statement governs and which needs correction.

## Decision

We will enforce this authority order: `CONSTITUTION.md`, then `method.md`, then the owning project document, then `README.md`. The owning document contains the sole normative statement of each fact or rule. Other documents link to it and may summarize orientation or status without redefining it. `README.md` never overrides an owner. When documents conflict, we will apply the higher-authority statement, correct the lower document, and remove duplicated rules rather than maintain parallel copies. Each structurally testable rule will have one check identifier mapped through [docs/owner-map.md](../docs/owner-map.md); semantic conflicts remain explicit human-judgment gates. ADRs record consequential decisions and point to owners instead of becoming alternate owners of operational detail.

## Alternatives rejected

- Let the newest document win — rejected: chronology does not establish authority and rewards accidental rewrites.
- Repeat key rules wherever readers need them — rejected: repetition creates edit surfaces that silently diverge.
- Make `README.md` the normative dashboard — rejected: its orientation role would conflict with detailed ownership and verification.
- Preserve apparently equivalent duplicates — rejected: future edits still lack deterministic resolution.

## Consequences

- Reviewers can resolve conflicts deterministically, and readers can navigate from summaries to the authoritative statement.
- Authors must locate the owner before editing; reorganization requires link and ownership updates.
- Status summaries cannot manufacture “done” from prose; they derive from checks and the gate ledger required by W1.
- Verification: explicit docs-lint script steps check authority links and structural duplication; semantic duplication and final conflict resolution remain `judgment (human)`.
