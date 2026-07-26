---
id: 01KWWTTV124RVF2R82TKSRCCAW
number: 20
type: design
schema: design/v1.0
name: Versioned file-metadata schemas — per-type schema markers, v0.1 → v1.0
created: 2026-07-06
updated: 2026-07-06
tags:
- schema
- versioning
- frontmatter
- migrate
- metadata
component: All
origin: planned
reserved: false
status:
  accepted:
    reached: 2026-07-06
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-06
  draft:
    reached: 2026-07-06
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-06
  revised:
    reached: 2026-07-06
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-06
  under-review:
    reached: 2026-07-06
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-06
author: Duncan McGreggor
---

# Versioned file-metadata schemas — per-type schema markers, v0.1 → v1.0

> **Status:** Accepted (Duncan + CDC, 2026-07-06). Realized in A6 (a new schema-versioning
> slice before the self-host cutover; migrate stamps the version). This ODD records *why* and
> *how*; the A6 arc-plan carries the slice breakdown.

## 1. Context

odm already versions **three** schema layers, but not the fourth:

- **Output projections** — `check/v1`, `rollup/v1`, `orient/v1`, `reconcile/v1` (the `--json`
  contracts; ODD-0013 §7.1 + A3/A5).
- **The index binary format** — `FORMAT_VERSION` (odm-index; A4).
- **The drift snapshot** — `SNAPSHOT_VERSION` (odm-reconcile; ODD-0019 / A5 slice07).
- **The file/frontmatter *metadata* schema — UNVERSIONED.** This is the gap.

As odm **self-hosts** (A6 brings the plan into `nodes/`) and the frontmatter keeps evolving
(A5 added `desired_facts`/`deferred`; more will come), an on-disk node with no schema marker
can't be safely dispatched or evolved: a reader can't tell v-old from v-new, and an additive
change can't be distinguished from a breaking one. The migrate boundary (A6 slice01/02) is
*already* the v-old→v-new line — it just needs a name.

## 2. Decision

**Stamp each node's frontmatter with a per-type, versioned schema marker:**

```yaml
schema: <type>/vMAJOR.MINOR      # project/v1.0, arc/v1.0, slice/v1.0, odd/v1.0, adr/v1.0, note/v1.0
```

- **Per-type, independently versioned.** Each node type carries its own schema contract, so a
  change to `slice`'s fields bumps `slice/v1.0 → slice/v1.1` without touching `project`/`odd`.
  This **reuses the `check/v1` marker idiom** already in the codebase — one convention across
  output projections *and* file metadata.
- **The current schema is `v1.0`.** New odm-created nodes are stamped `<type>/v1.0`.
- **Unversioned frontmatter ⇒ `v0.1`.** A file with no `schema:` field is treated as `v0.1`
  — and in practice that is **only the legacy design docs** (the pre-v1.0 ODDs); the rebuilt
  corpus stamps `v1.0` from creation.
- **Implementation: shared-core + per-type validity, not necessarily N structs.** The common
  fields (id, number, title, type, created, updated, gates, `part_of`/edges) stay one
  shared representation; the type-specific fields (`desired_facts`/`deferred` for work nodes,
  `supersedes` for docs, `affects` for decisions) are validated **per type** — a
  wrong-type field is a `check` finding ("`desired_facts` is not valid on an `odd`"). Split
  into separate structs only if the divergence grows to warrant it.
- **`saga` gets no schema yet** — PROJECT-MANAGEMENT names it as a slot with "no operational
  weight"; add `saga/v1.0` if/when saga becomes a real node type.

## 3. Disambiguation — this is a distinct version axis

odm now carries several version numbers; **keep them from colliding**:

| Axis | Example | What it versions |
|------|---------|------------------|
| **File-metadata schema (this ODD)** | `schema: slice/v1.0` | the on-disk frontmatter contract, per type |
| Output projection markers | `check/v1`, `rollup/v1` | the `--json` output contracts |
| Index / drift binary formats | `FORMAT_VERSION`, `SNAPSHOT_VERSION` | the `.odm/` derived artifacts |
| Design-doc tree | `docs/design-v1.0.0/` | the *design*'s version (not a release) |
| Crate / release | `oxur-odm 1.0.0` | the shipped binary |
| Legacy ODD `version:` field | `version: 1.1` | a **document's content** version — **not** a schema version |

The new field is `schema:` (a per-type marker), deliberately **not** overloading the legacy
`version:` (which stays the document's own content revision).

## 4. The v0.1 → v1.0 upgrade path (A6 / migrate)

This makes **migrate the schema-upgrade path**, which is exactly what A6 already is:

- **Read:** a legacy ODD with no `schema:` field is `odd/v0.1`.
- **Write:** the imported node is stamped `odd/v1.0` (and every new odm-created node stamps
  `<type>/v1.0`).
- The self-hosted plan-set (A6 self-host slice) is stamped `project/v1.0`, `arc/v1.0`,
  `slice/v1.0` as it enters `nodes/`.

So the schema marker + the v0.1→v1.0 stamp is an **A6 concern**: a schema-versioning slice
lands **before** the self-host cutover, so the plan-set is self-hosted already at v1.0.

## 5. Consequences

- **Safe evolution.** Additive changes bump the minor (`v1.0 → v1.1`, e.g. a new optional
  field); a breaking change bumps the major (`v2.0`) and a reader can refuse/upgrade
  explicitly. Same discipline the output markers and binary formats already use.
- **Readers can dispatch.** `check`/`rollup`/`orient`/migrate can branch on the schema
  version rather than guessing from field presence.
- **Forward-compat is honest.** A node stamped with an *unknown newer* schema (e.g. a v1.1
  read by a v1.0 binary) is a **clear, reported** condition — never a silent misparse or
  dropped field.
- **Per-type independence.** The node types evolve on their own cadence; the marker records
  which contract each file speaks.

## 6. Alternatives considered

- **One global metadata version** (`schema: v1.0` for all types). Simpler (one number), but a
  change to any one type bumps the schema for *all* types — coupling their evolution. Rejected
  in favor of per-type markers (independent evolution; matches `check/v1`). *(Duncan's call,
  2026-07-06.)*
- **No versioning** (keep frontmatter unmarked). Rejected: can't evolve safely, can't
  dispatch, can't tell additive from breaking — the exact problems self-host will multiply.
- **N separate per-type structs from the start.** Deferred to an implementation call: the
  *contract* is per-type + versioned regardless; the *code* starts shared-core + per-type
  validation and splits only if divergence earns it (YAGNI).

## 7. Open questions (settle in the schema-versioning slice)

- The exact **per-type field sets** (which fields are valid on which type) — pin against the
  real model when the slice is drawn.
- **Shared-core struct vs per-type structs** — the implementation call above.
- **Is `schema:` required on every new node?** (Recommend yes; migrate + `new` stamp it;
  absence ⇒ v0.1 legacy-only.)
- **Validation severity** — a wrong-type field: `check` Error or Warning? (Lean Error — it's a
  structural contract violation, not advisory.)

## 8. Impact on the plan

- **A6 gains a schema-versioning slice** (a new slice03), sequenced **before** the self-host
  cutover, so the self-hosted plan-set enters `nodes/` already stamped `v1.0`. self-host,
  PM-skill, and retire shift down one.
- **odm-core**: the `schema` field + per-type field-validity (a `check` extension).
- **odm-migrate**: stamp `<type>/v1.0` on import; read unversioned legacy as `v0.1`.
- The output-projection markers (`check/v1` …) are **unaffected** — this is the file-metadata
  axis, a sibling not a replacement.

## 9. References

- ODD-0013 §7.1 (the output schema-marker convention) + the node model / frontmatter.
- ODD-0014 (`FORMAT_VERSION`) + ODD-0019 (`SNAPSHOT_VERSION`) — the sibling versioned formats.
- `docs/design-v1.0.0/arc06-migrate-self-host/arc-plan.md` (the slice that realizes this).
- A5 slice02/04/06 — the additive-evolution discipline (`skip_serializing_if`, no schema bump
  for additive fields) this ODD generalizes into an explicit version marker.
