# Slice 03 (Arc 06): schema versioning (ODD-0020)

> Plan-of-record for A6 slice03 — the operator's schema-versioning decision (ODD-0020),
> inserted **before** the self-host cutover (slice04) so the plan-set enters `nodes/` already
> stamped `v1.0`. A genuine model slice: the `schema` frontmatter field + per-type validity +
> migrate stamping + a backfill of the already-migrated nodes.

## Goal

Every odm-created node carries a per-type versioned schema marker — `schema: <type>/v1.0` —
so the file-metadata schema can evolve safely and readers can dispatch on it. Unversioned
frontmatter is understood as `v0.1` (legacy design docs only). This is ODD-0020 realized.

## Design of record: ODD-0020

Implements ODD-0020 §2 (per-type markers), §3 (the distinct version axis), §4 (the v0.1→v1.0
migrate path), §5 (forward-compat). Read it first.

## ODD-0020 open questions — resolved here (Duncan agreed all counts)

- **`schema:` required on every new node? → Yes.** `odm new`, `migrate`, and every
  odm-created write stamp `<type>/v1.0`. **Absent ⇒ `v0.1`** (a *computed* default on read,
  applying only to legacy `docs/design` ODDs, which migrate never mutates).
- **Validation severity? → Error.** A field not valid for a node's type is a **structural
  contract violation** (a `check` Error), not advisory — it means the file claims a schema it
  doesn't obey.
- **Shared struct vs per-type structs? → Shared-core + per-type validity.** One `Frontmatter`
  representation; per-type *field-validity* enforced by a `check` pass. Split into per-type
  structs only if divergence later earns it (YAGNI).

## Scope — in

1. **The `schema` frontmatter field** (`odm-core`): `schema: <type>/vMAJOR.MINOR`; additive
   round-trip; **absent ⇒ `v0.1`** on read (computed, not written); an odd/adr/note/project/
   arc/slice node stamps its own type's marker.
2. **New odm-created nodes stamp `<type>/v1.0`** — `odm new` + any create path + `migrate`
   (V-3). No new node is ever written unversioned.
3. **Per-type field-validity** (`odm-cli check` + `odm-core`): a field invalid for the node's
   type → a `check` **Error** finding. Known per-type sets (pin against the model): work nodes
   (project/arc/slice) may carry `desired_facts`/`deferred`/`part_of`; docs (odd) carry
   `supersedes`; decisions (adr) carry `affects`; etc. Reuses the slice05-A5 `check_*` +
   `violation_severity` pattern.
4. **migrate stamps `v1.0` / reads legacy as `v0.1`** — importing a legacy ODD (no `schema`)
   creates a node stamped `odd/v1.0`; the legacy source is understood as `v0.1`.
5. **Backfill the already-migrated nodes** — slice02 wrote 12 `odd` nodes **before** this
   field existed; they must carry `odd/v1.0` (odm-created nodes are never `v0.1`). Any
   `nodes/` file lacking a `schema` field is stamped `<type>/v1.0`. **Mechanism (flag):**
   fold into migrate as a **schema-upgrade** (an existing-but-unversioned node gets stamped —
   still idempotent once stamped) *or* a dedicated one-time stamp pass. Recommend the
   migrate-upgrade (keeps one entry point; re-run after stamping is a no-op).
6. **Forward-compat** — a node stamped with an **unknown newer** schema (e.g. `odd/v1.1` read
   by a v1.0 binary) is a **clear reported error**, never a silent misparse or dropped field
   (ODD-0020 §5).

## Scope — out (named, not dropped)

- **Self-host cutover** (plan-set → `nodes/`; `orient`/`rollup` on the self-hosted corpus) —
  **slice04**. slice03 makes it so that when the plan-set lands, it's stamped `v1.0`.
- **PM-skill / retire prose** — **slice05/06**.
- **Actually evolving a schema to v1.1/v2.0** — out; this slice establishes v1.0 + the
  machinery. The first real bump is a future change.
- **Output-projection markers** (`check/v1`…) — untouched; this is the file-metadata axis.

## Design notes / decisions to flag

- **The field is per-type but the struct is shared** — don't split `Frontmatter` into six
  structs; enforce the per-type contract with the validity `check`. (ODD-0020 §2.)
- **Legacy docs stay unversioned (v0.1)** — migrate never mutates `docs/design`; those ODDs
  are `v0.1` by absence. Only `nodes/` files are stamped.
- **The backfill is the one migration wrinkle** — the slice02 nodes predate the field. Flag
  the chosen mechanism (migrate-upgrade vs stamp-pass) in the closing-report; either way it's
  idempotent and never deletes.
- **Additive to every existing node/test** — adding `schema` must not break existing
  round-trips (absent ⇒ v0.1, re-emit stable); run the full suite.

## Verification approach

`odm-core` + `odm-migrate` + `odm-cli` tests:

- the `schema` field round-trips; absent ⇒ `v0.1`; a stamped node ⇒ its `<type>/v1.0`.
- `odm new <type>` (or the create path) → the node carries `<type>/v1.0`.
- per-type validity: a `desired_facts` on an `odd` (or `supersedes` on a `slice`) → a `check`
  **Error**; a valid node → no finding.
- migrate a legacy ODD → node stamped `odd/v1.0`; legacy file untouched (v0.1 by absence).
- backfill: the 12 slice02 `odd` nodes end up carrying `odd/v1.0` (via the chosen mechanism);
  re-run is idempotent.
- an `odd/v1.1` fixture read by this (v1.0) binary → a reported error, not a silent parse.
- clippy `-D warnings`; no `unsafe`; coverage ≥ 90% on new paths; **full workspace green**
  (additive field breaks nothing).

Cargo rows are CC-`attested`, reproduced via CI / local 1.85+.

## Exit criteria

Ledger V-1…V-6 reach a final status: the `schema` field exists (absent ⇒ v0.1; new nodes
stamp `<type>/v1.0`); per-type field-validity is a `check` Error; migrate stamps v1.0 / reads
legacy v0.1; the already-migrated nodes are backfilled to `odd/v1.0`; an unknown newer schema
is a reported error; gates + no regression. **On close, slice04 self-hosts the plan-set —
already stamped v1.0.**

> **Render/convention:** `writeln!` + `tabled` (no `oxur-cli`). **Git:** branch off
> `release/1.0.x`. Reuse the A5 `check_*` + `violation_severity` pattern for the validity
> finding; the migrate stamping rides the slice01 write path.
