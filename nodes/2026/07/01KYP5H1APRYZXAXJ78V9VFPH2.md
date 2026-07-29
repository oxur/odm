---
id: 01KYP5H1APRYZXAXJ78V9VFPH2
number: 586594500
type: artifact
schema: artifact/v1.1
name: 'Closing report — Slice 03 (Arc 06): schema versioning (ODD-0020)'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc06-migrate-self-host/slice03-schema-versioning/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKDGTBTSSVNYV89CE7
---
# Closing report — Slice 03 (Arc 06): schema versioning (ODD-0020)

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> `attested` (built + run locally on a 1.95 toolchain). CDC reproduces (CI /
> local 1.85+) to elevate to `reproduced`.
>
> **Branch:** `arc06-slice03-schema-versioning`, branched off
> `arc06-slice02-migrate-odm-docs` (slices 01–02 unmerged; the prompt's fallback).
> Rebase onto `release/1.0.x` once 01–02 merge.
>
> **The operator's schema-versioning decision (ODD-0020), realized.** A per-type
> `schema: <type>/v1.0` marker + per-type field-validity + migrate stamping + a
> backfill of the slice02 nodes — sequenced **before** the self-host cutover
> (slice04) so the plan-set enters `nodes/` already at `v1.0`.

## Per-row walk

**V-1 — the `schema` field — done (attested).** New `odm_core::schema` module:
`SchemaMarker { node_type, version }` serializing as `<type>/vN.N`, and
`SchemaVersion` with `CURRENT` (v1.0) / `LEGACY` (v0.1). `Frontmatter` gains
`schema: Option<SchemaMarker>` (skip-if-none, so a legacy node round-trips
byte-identically) with `schema()` / `schema_version()` (absent ⇒ v0.1, a computed
default) / `stamp_schema()`. `schema_marker_round_trip` + `schema_absent_is_v0_1`
→ ok.

**V-2 — new nodes stamp `<type>/v1.0` — done (attested).** `commands::new` calls
`stamp_schema()`; `new_node_stamps_schema_v1` → ok (project/odd/slice each carry
`<type>/v1.0`). The create paths are `odm new` + `migrate`'s build (V-4); the
mutating commands (rename/retire/…) load-modify-persist and preserve the existing
marker, so no node is ever written unversioned.

**V-3 — per-type field-validity is a `check` Error — done (attested).**
`check::content_validity` + `Violation::FieldNotValidForType`, with two pinned
buckets: **work-only** `{desired_facts, deferred}` and **document-only**
`{supersedes, affects}`. A field from the wrong bucket for the node's type is an
Error. `check_flags_wrong_type_field` (odm-core) and `check_wrong_type_field_is_error`
(odm-cli: exit 1, code `wrong-type-field`, `error` severity in `--json`) → ok; a
valid corpus stays green (`check_green_when_fields_valid_for_type`). **It runs over
store-loaded frontmatters** — the index omits the type-specific fields (see
Deviation 2).

**V-4 — migrate stamps `v1.0` / reads legacy `v0.1` — done (attested).**
`mapping::build_node` calls `stamp_schema()` on every imported node.
`migrate_stamps_schema_v1` → ok: every imported node carries `odd/v1.0`, and a
byte-snapshot of the legacy source is unchanged (the legacy ODD is understood as
`v0.1` by absence — never mutated).

**V-5 — backfill the already-migrated nodes — done (attested).** Mechanism:
`backfill_schema(store, mode)` — stamp any unversioned `nodes/` file to
`<type>/v1.0` — **folded into `migrate`** (runs before import) as the single
schema-upgrade entry point; idempotent (stamped → skip), dry-run-safe, never
deletes. `backfill_stamps_unversioned_nodes` (+ dry-run) and
`migrate_folds_in_the_backfill` → ok. **Real backfill committed on the branch:**
`odm migrate docs/design` reported 12 upgraded + 1 created (ODD-0020, added after
slice02) = **13 `nodes/` files, all `odd/v1.0`**; a re-run is 0 created / 0
upgraded / 13 skipped.

**V-6 — forward-compat + gates — done (attested).**
`SchemaVersion::is_newer_than_current` + `Violation::UnsupportedSchema`: a node
stamped newer than the binary (a newer minor *or* major) is a reported error, and
the marker is fully parsed (never a silent misparse or dropped field).
`unknown_newer_schema_is_reported_error` → ok. clippy `--all-targets
--all-features -D warnings` exits 0; no `unsafe` in odm-core/odm-migrate; line
coverage schema.rs 96.55% / check.rs 95.34% / frontmatter.rs 99.52% /
odm-migrate lib.rs 99.21% (all ≥ 90); `cargo test --workspace` green (50 suites).
`odm check` on the real 13-node corpus → "ok (13 node(s), no problems)", exit 0.

Rows: 6. Done: 6. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item shipped: the `schema` field (absent ⇒ v0.1), new-node
stamping, per-type field-validity as a `check` Error, migrate stamping, the
backfill of the slice02 nodes (real + committed), and the forward-compat report.
Every "out" item stayed out: **no** self-host cutover (slice04), **no** PM-skill
(slice05/06), **no** actual schema bump to v1.1/v2.0 (this establishes v1.0 + the
machinery), **no** output-projection-marker change (this is the file-metadata
axis). **No `odm-index` change.**

Two decisions are disclosed, not dropped: the backfill mechanism (migrate-folded
`backfill_schema` — Deviation 1) and the store-read for content-validity
(Deviation 2).

## Deviations / decisions flagged

See the ledger's **Deviations / decisions flagged**. In brief: backfill is a
migrate-folded `backfill_schema` (the recommended single entry point);
content-validity reads the store (the index omits the type-specific fields +
`schema` — the A5 store-read pattern; pre-existing `check_reenter_when` shares the
index-blindness, not fixed here); the per-type buckets are pinned
(work-only `{desired_facts, deferred}`, document-only `{supersedes, affects}`);
the `schema` marker carries the type redundantly per ODD-0020 §2 (a type-mismatch
check is a possible future addition). **No amendment to ODD-0020.**

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice03 deliver A-3 + the A-12 mechanism?** Yes. The `schema` field +
per-type validity + migrate stamping + backfill are tested, and the real `nodes/`
corpus is now uniformly `odd/v1.0`. A-3 (slice03 closed) is `attested`; A-12
(compose: nodes carry the per-type marker, migrate stamps v1.0 / reads legacy
v0.1, a wrong-type field is a `check` finding) is **mechanism-complete**, to be
reproduced at arc scale at arc-close.

**2. What slice03 reveals for slice04 (the self-host cutover).**
  - **The plan-set enters already stamped — that was the whole point of the
    sequence.** slice04 brings the `design-v1.0.0` plan set (project/arc/slice
    nodes) into `nodes/`. Its create/import path must call `stamp_schema()` (as
    `migrate`'s build and `odm new` do), so those nodes are `project/v1.0` /
    `arc/v1.0` / `slice/v1.0` from creation — no second backfill needed. If slice04
    writes them via a path other than `migrate`/`new`, it must stamp explicitly.
  - **Per-type validity will meet *work* nodes for the first time.** The 13
    self-hosted nodes are all `odd`; slice04 adds work nodes with `desired_facts` /
    `deferred` / `part_of` (all valid on work) — and the mixed corpus must stay
    `check`-green. The bucket rule (work-only vs document-only) is exactly what
    keeps an odd's `supersedes` and a slice's `desired_facts` both valid; slice04
    should confirm green on the mixed corpus (as slice02 confirmed it for pure-odd).
  - **The content-validity store-read now runs on every `check`.** With work nodes
    added, `odm check` still does one index reconcile + one `store.load_all()` (for
    field-validity + schema). Fine at plan-set scale; flagged so the double-read is
    a known, accepted cost (the A5 tradeoff), not a surprise.
  - **The reflexive-import ordering (from the slice02 bubble-up) still holds for
    slice04.** Import the plan describing the migration *last*; lean on `--dry-run`
    + a git checkpoint. slice03 adds no new reflexivity, but the plan-set now
    includes ODD-0020 + this slice's docs — all already self-hosted as `odd/v1.0`.

**3. Silent-drop diff at the arc altitude.** Nothing the arc-plan scoped for A-3 /
A-12 is missing. The backfill-mechanism and store-read decisions are disclosed
here so the arc-plan records them rather than a reader discovering them in slice04.

**4. Reusable finding.** *Version the metadata, then let the version gate the
reader.* The `schema` marker + `is_newer_than_current` is the general recipe for
safe on-disk evolution — the same shape as the output markers (`check/v1`) and the
binary formats (`FORMAT_VERSION`/`SNAPSHOT_VERSION`), now unified across all four
axes. Future frontmatter changes bump the minor (additive) or major (breaking) and
a reader can refuse/upgrade explicitly instead of guessing from field presence.

Per the PM Part IV slice-close step, this bubble-up is propagated into
`arc-plan.md` (the A-3 row evidence + A-12 mechanism-complete + a v1.6
version-history entry), not only here.
