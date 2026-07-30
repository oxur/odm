---
id: 01KWXMBBTJCJ30F3TE4BJJV2CB
number: 1000
type: project
schema: project/v1.1
name: Vision
created: 2026-06-26
updated: 2026-07-30
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/project-plan.md
  class: vision
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-30
  synthesis: editorial-merge
  attestation: 'odm-migrate on 2026-07-30: distills project-plan.md''s Definition-of-done section verbatim'
edges:
  supersedes:
  - node: 01KYSX4RGCZT9H1N83TJFX8XFB
    kind: updates
---
`odm` is a markdown/git-native, dependency-ordered planning + documentation substrate
that **mechanically actualizes** the collaboration framework: stable-identity nodes +
an explicit dependency DAG + order *derived* by topological sort + per-edge
staleness/reconciliation + one *complete* graph as the source of truth. Success test:
a fresh session reaches full situational awareness from `odm orient` alone.

The architecture is fixed in **ODD-0013** (this file is the plan, not the design).

**In scope (v1.0.0):** one unified node graph (work nodes project/arc/slice +
document nodes odd/adr/note); stable ULID identity; typed edges → petgraph DAG; cycle
detection + explicit tears; derived-order queries (`next`/`blocked`/`path`); multi-gate
status vectors with evidence levels + evidence-leveled satisfaction; mechanical
integrity checking (`check`); generated rollup + `orient`; incremental index;
desired-vs-actual reconciliation; a legacy `migrate` importer + self-hosting; LLM
ergonomics (`--json` everywhere, question-named commands, errors-as-affordances,
idempotent describe-or-create, `--dry-run`/`--yes`, bare `odm` orients).

**Non-goals (ODD-0012):** no ticketing system / server / database (files are the
source, `odm` is the build); not a scheduler/optimizer (dependency order is *correct*,
not *fastest*; priority is an optional advisory layer, not in the MVP); no attempt at
*complete* traceability (dependency + verification edges, not everything); no
preservation of the legacy on-disk truth-encoding (number-as-identity,
state-in-directory, dustbin) — migrated *into* the new model, not carried forward.