---
id: 01KYP5H1FKCJRC2V2WSGENG7ZS
number: 581644600
type: artifact
schema: artifact/v1.1
name: 'Slice 03 (Arc 06): schema versioning (ODD-0020)'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc06-migrate-self-host/slice03-schema-versioning/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKDGTBTSSVNYV89CE7
---
# Slice 03 (Arc 06): schema versioning (ODD-0020)

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+). Five-iteration cap. Realizes **ODD-0020**; sequenced **before** self-host so the
> plan-set (slice04) enters `nodes/` already at `v1.0`.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| V-1 | The **`schema` frontmatter field** exists: `schema: <type>/vMAJOR.MINOR`; additive round-trip; **absent ⇒ `v0.1`** on read (computed, not written); a stamped node reads its `<type>/vN.N` | `cargo test -p odm-core schema_marker_round_trip` + `schema_absent_is_v0_1` → ok | serious | ODD-0020 §2/§4 | done | attested — new `odm_core::schema` (`SchemaMarker { node_type, version }` ⇄ `<type>/vN.N`; `SchemaVersion` `CURRENT`=v1.0 / `LEGACY`=v0.1); `Frontmatter.schema: Option<SchemaMarker>` (skip-if-none → legacy round-trips byte-identically); `schema()`/`schema_version()` (absent⇒v0.1)/`stamp_schema()`. `schema_marker_round_trip` + `schema_absent_is_v0_1` → ok. | Per-type marker (reuses the `check/v1` idiom). Absent-⇒-v0.1 applies only to legacy `docs/design` (migrate never mutates them). |
| V-2 | **New odm-created nodes stamp `<type>/v1.0`** — `odm new <type>` (and every create/write path) writes the current schema marker; no new node is ever written unversioned | `cargo test -p odm-cli new_node_stamps_schema_v1` → ok (a freshly-created node of each type carries `<type>/v1.0`) | serious | ODD-0020 §2 | done | attested — `commands::new` calls `stamp_schema()`; `new_node_stamps_schema_v1` → ok (project/odd/slice each carry `<type>/v1.0`). Create paths = `odm new` + `migrate` build (V-4); mutating commands preserve the existing marker. | "Required on every new node" (resolved). |
| V-3 | **Per-type field-validity** is a `check` **Error**: a field invalid for the node's type (e.g. `desired_facts` on an `odd`, `supersedes` on a `slice`) is flagged; a valid node produces no finding | `cargo test -p odm-core check_flags_wrong_type_field` + `cargo test -p odm-cli check_wrong_type_field_is_error` → ok | serious | ODD-0020 §2 (per-type contract) / A5 `check_*` pattern | done | attested — `check::content_validity` + `Violation::FieldNotValidForType`; pinned buckets: work-only `{desired_facts, deferred}`, document-only `{supersedes, affects}`. `check_flags_wrong_type_field` + `check_wrong_type_field_is_error` (exit 1, code `wrong-type-field`, Error in `--json`) → ok. **Runs over store-loaded frontmatters** (the index omits `desired_facts`/`deferred`/`schema` — the A5 store-read pattern). | Reuses slice05-A5 `check_*` + `violation_severity` (Error, not Warning). Pin the per-type field sets against the model. |
| V-4 | **migrate stamps `v1.0` / reads legacy as `v0.1`** — importing a legacy ODD (no `schema`) creates a node stamped `odd/v1.0`; the legacy source is understood as `v0.1`; legacy file untouched | `cargo test -p odm-migrate migrate_stamps_schema_v1` → ok (imported node carries `odd/v1.0`; legacy byte-unchanged) | serious | ODD-0020 §4 | done | attested — `mapping::build_node` calls `stamp_schema()`; `migrate_stamps_schema_v1` → ok (every imported node `odd/v1.0`; legacy byte-snapshot unchanged). | The v0.1→v1.0 upgrade path. Rides the slice01 write path. |
| V-5 | **Backfill the already-migrated nodes** — the 12 slice02 `odd` nodes (written before this field) + any `nodes/` file lacking a `schema` are stamped `<type>/v1.0`; the mechanism is **idempotent** and **never deletes** | `cargo test -p odm-migrate backfill_stamps_unversioned_nodes` → ok (a schema-less node → `<type>/v1.0`; re-run is a no-op) AND after slice03, the 12 committed `odd` nodes carry `odd/v1.0` | serious | ODD-0020 §4 / slice02 (12 nodes predate the field) | done | attested — **mechanism: `backfill_schema(store, mode)` folded into `migrate`** (the recommended single entry point); idempotent (stamped→skip), dry-run-safe, never-deletes. `backfill_stamps_unversioned_nodes` + `migrate_folds_in_the_backfill` → ok. **Real backfill committed**: `odm migrate docs/design` → 12 upgraded + ODD-0020 created = **13 `nodes/` files all `odd/v1.0`**; re-run 0/0/13. | odm-created nodes are **never** `v0.1`. Never-mutate applies to legacy `docs/design`, not to `nodes/` (odm owns those). |
| V-6 | **Forward-compat** — a node stamped with an **unknown newer** schema (e.g. `odd/v1.1` read by a `v1.0` binary) is a **clear reported error**, never a silent misparse / dropped field; clippy `-D warnings`; no `unsafe`; coverage ≥ 90% on new paths; **full workspace green** (the additive field breaks nothing) | `cargo test -p odm-core unknown_newer_schema_is_reported_error` → ok AND `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-migrate/src` AND `cargo llvm-cov --summary-only -p odm-core -p odm-migrate` → **line** ≥ 90% AND `cargo test --workspace` green | serious | ODD-0020 §5 / CLAUDE.md | done | attested — `SchemaVersion::is_newer_than_current` + `Violation::UnsupportedSchema` (a newer minor *or* major → reported); marker fully parsed (no misparse). `unknown_newer_schema_is_reported_error` → ok. clippy `-D warnings` exit 0; no `unsafe`; line cov: schema.rs 96.55%, check.rs 95.34%, frontmatter.rs 99.52%, odm-migrate lib.rs 99.21% (all ≥ 90); `cargo test --workspace` green (50 suites). | Additive to every existing node/test (absent ⇒ v0.1, re-emit stable). |

## What Worked

- **Shared-core + a per-type *validity* pass, exactly as ODD-0020 §2 prescribed.**
  No `Frontmatter` split into six structs — one representation, one additive
  `schema` field, and a `check::content_validity` pass enforces the per-type
  contract. The two field buckets (work-only `{desired_facts, deferred}`,
  document-only `{supersedes, affects}`) made the rule a four-line matrix.
- **The index gap forced the right seam.** `odm check` is index-backed, and the
  index deliberately omits `desired_facts`/`deferred`/`schema` — so the
  content-validity pass reads the **store** (the same store-read A5 used for
  drift/deferred). Discovering this early kept V-3/V-6 from silently no-op-ing on
  index-reconstructed frontmatters.
- **Backfill folded into migrate = one entry point.** `backfill_schema(store,
  mode)` stamps any unversioned `nodes/` file to `<type>/v1.0`, idempotent and
  dry-run-safe; `migrate` runs it first. So `odm migrate docs/design` both imports
  new legacy ODDs *and* upgrades the pre-schema slice02 nodes — a single verb.
- **Additive held, byte-for-byte.** `schema: Option<SchemaMarker>` with
  skip-if-none means a legacy (unversioned) node round-trips byte-identically and
  the whole 50-suite workspace stayed green — the additive-evolution discipline
  ODD-0020 generalizes, applied to itself.
- **The real run self-hosted ODD-0020 too.** The backfill run also imported the
  freshly-added ODD-0020, so the schema-versioning decision is itself a v1.0 node.

## Deviations / decisions flagged

1. **Backfill mechanism = migrate-folded `backfill_schema`** (the recommended
   option). A store-wide stamp pass folded into `migrate` (runs before import),
   covering *any* unversioned node type — more general than an odd-only
   idempotence-upgrade, still one entry point. Reported via a new `upgraded`
   report section.
2. **Content-validity reads the store, not the index.** The index omits the
   type-specific fields + `schema`, so the new checks run over `store.load_all()`
   in `aggregate` (a second read alongside the index reconcile — the A5-flagged
   read-cost tradeoff, not a probe cost). Pre-existing `check_reenter_when` has the
   same index-blindness; not fixed here (out of scope), noted for a future pass.
3. **Per-type field buckets pinned** (ODD-0020 §7 open Q): work-only
   `{desired_facts, deferred}`; document-only `{supersedes, affects}`. `part_of`
   and the ordering edges are common (not restricted). A candidate amendment if a
   real type needs a field outside its bucket.
4. **`schema` marker carries the type** (`odd/v1.0`) redundantly with `type:`, per
   ODD-0020 §2. A schema-type≠node-type mismatch is *not* yet a check (out of
   scope); the stamp always uses the node's own type, so odm-created nodes never
   diverge. Flagged as a possible future validity check.
5. **No amendment to ODD-0020** was needed — the marker shape, per-type validity,
   and forward-compat all fit the design of record.

## Closure

Closed at commit `5b5dd43` on 2026-07-06. Verified by: CC (proposed-done, attested);
CDC to reproduce. Rows: 6. Done: 6. Deferred: 0. No-op: 0.
On close → bubble up to `arc-plan.md` (A-3) per LEDGER-DISCIPLINE v2.0 §A;
slice04 self-hosts the plan-set — which now enters `nodes/` already stamped `v1.0`.
