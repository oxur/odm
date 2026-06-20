# Arc 01 — Substrate & node CRUD (plan-of-record)

> Arc plan for the v1.0.0 rebuild. Source of truth for *what this arc delivers*;
> per-slice detail lives in each `sliceNN-<slug>/`. Refs: ODD-0013 (architecture),
> ODD-0015 (breakdown §3), ODD-0001 (post-mortem). `depends_on:` nothing — this is
> the foundation arc.

## Goal

Stand up the workspace and the node substrate: stable ULID identity, the node
model + frontmatter schema, the `nodes/YYYY/MM/<ULID>.md` git-native store, and
basic CRUD — proving "files are the source." No graph/edges yet (that is Arc 02).

## Exit criteria (arc acceptance)

- Create / list / show / rename / retire / supersede nodes of each type
  (`project`/`arc`/`slice` + `odd`/`adr`/`note`), persisted as markdown+frontmatter,
  git-tracked, with stable ULID ids that never move on retitle/reparent.
- `use`/`context` set the current project/arc.
- `check` v1 passes: frontmatter schema, link-integrity (no dangling
  `part_of`/`supersedes`/edge refs), supersession-chain integrity.
- `list` works by full filesystem scan (the `odm-index` accelerator is Arc 04).

## Slices (dependency-ordered — see ODD-0015 §3)

1. **slice01 — workspace scaffolding** ← *this slice; ready for CC*
2. slice02 — stable identity core (ULID, `Node`, `NodeType`)
3. slice03 — frontmatter schema + round-trip
4. slice04 — store layer (`nodes/YYYY/MM/<ULID>.md`, gix, odm.toml, atomic writes)
5. slice05 — node CRUD commands (`new`/`list`/`show`/`rename`/`retire`/`supersede`, `use`/`context`)
6. slice06 — `check` v1 + link-integrity

## Crate set introduced this arc

`oxur-odm` (umbrella → `odm` binary), `odm-cli`, `odm-core`, `odm-store`,
`odm-graph` (stub until Arc 02). `odm-index`/`odm-reconcile`/`odm-migrate` are
deferred to the arcs that need them (04/05/06-line), per ODD-0015 — we do not carry
empty crates for far-off arcs.

## Method

Ledger per slice; CC implements, CDC verifies every row independently before close;
five-iteration cap. Each slice's five-document set lives in its directory.
