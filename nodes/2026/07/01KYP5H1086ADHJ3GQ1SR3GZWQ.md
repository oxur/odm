---
id: 01KYP5H1086ADHJ3GQ1SR3GZWQ
number: 564792800
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 03 (Arc 06): schema versioning (ODD-0020)'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc06-migrate-self-host/slice03-schema-versioning/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKDGTBTSSVNYV89CE7
---
# CC Prompt — Slice 03 (Arc 06): schema versioning (ODD-0020)

The operator's schema-versioning decision, realized. Add a per-type versioned schema marker to
node frontmatter — `schema: <type>/v1.0` — with per-type field-validity, migrate stamping, and
a backfill of the already-migrated nodes. Sequenced **before** the self-host cutover (slice04)
so the plan-set enters `nodes/` already at `v1.0`. This is a genuine model slice
(`odm-core` + `odm-migrate` + `odm-cli`).

> **Start condition:** slices 01–02 CDC-verified. Branch off **`release/1.0.x`**:
> **`arc06-slice03-schema-versioning`** (not `main`). If 01–02 unmerged, branch off slice02 +
> flag for rebase.

## Read first
1. **ODD-0020** (`docs/design/04-accepted/0020-versioned-file-metadata-schemas.md`) — the
   design of record (§2 per-type markers, §3 the distinct axis, §4 migrate path, §5
   forward-compat).
2. `slice03-schema-versioning/ledger.md` (6 rows) + `slice-doc.md` (same dir) — the resolved
   ODD-0020 open questions (schema required on new nodes; validity = Error; shared-core +
   per-type validity).
3. `../arc-plan.md` — A6 Arc Ledger (this slice closes **A-3**, mechanism for compose **A-12**),
   v1.5 (the fold-in).
4. Reuse: `crates/odm-core/src/frontmatter.rs` (add the field; the `insert_extra`/serde
   discipline from A5), `crates/odm-core/src/check.rs` + `odm-cli/src/commands.rs`
   (`check_*` + `violation_severity` from A5 slice05 — the per-type validity finding rides
   this), `crates/odm-migrate` (stamping + the backfill), `crates/odm-store` (node write).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then API design + serde evolution + error handling.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence `attested`; per-row walk +
  **Bubble-up to the arc**; **propagate it into `arc-plan.md` (A-3 + version entry)**).

## Task
1. **`schema` field** (V-1): `schema: <type>/vMAJOR.MINOR`; additive round-trip; **absent ⇒
   `v0.1`** on read (computed, not written).
2. **New nodes stamp `<type>/v1.0`** (V-2): `odm new` + every create/write path.
3. **Per-type field-validity** (V-3): a field invalid for the node's type → a `check`
   **Error** (reuse `check_*` + `violation_severity`). Pin the per-type field sets against the
   model (work nodes: `desired_facts`/`deferred`/`part_of`; odd: `supersedes`; adr: `affects`;
   …).
4. **migrate stamps v1.0 / reads legacy v0.1** (V-4): imported node → `odd/v1.0`; legacy
   untouched.
5. **Backfill** (V-5): the 12 slice02 `odd` nodes (+ any `nodes/` file lacking `schema`) →
   `<type>/v1.0`. **Recommended mechanism: fold into migrate as a schema-upgrade** (an
   existing-but-unversioned node gets stamped; still idempotent once stamped). Never delete;
   never-mutate applies to legacy `docs/design`, **not** to `nodes/` (odm owns those). Flag
   the mechanism you choose.
6. **Forward-compat** (V-6): an unknown *newer* schema (`odd/v1.1` read by v1.0) → a **clear
   reported error**, never a silent misparse.
7. **Gates + no regression** (V-6): clippy `-D warnings`; no `unsafe`; ≥ 90% new-path line;
   `cargo test --workspace` green (the additive field must break nothing).

## Constraints (flag, don't silently change)
- **Amend, don't work around.** If the per-type validity or the marker shape needs to differ
  from ODD-0020, raise an amendment **against ODD-0020** (it's the design of record).
- **Absent ⇒ v0.1 is legacy-only.** Only `docs/design` ODDs are unversioned; every `nodes/`
  file must end up stamped (via create-stamp or backfill). Don't leave odm-created nodes at
  v0.1.
- **Shared-core, not N structs** — enforce per-type via the `check` validity pass.
- **Additive:** adding `schema` must not break existing round-trips or the workspace suite.
- **Never delete; never mutate legacy `docs/design`** — the backfill touches `nodes/` only.
- **Render** `writeln!`+`tabled`; **branch off `release/1.0.x`**.

## Deliverables
The `schema` field + per-type validity + migrate stamping + backfill + forward-compat, with
`ledger.md` evidence per row (`attested`); a `closing-report.md` — per-row walk **plus the
Bubble-up to the arc** (did slice03 deliver A-3 + the A-12 mechanism; what it reveals for
slice04's self-host — esp. the plan-set stamping; the backfill mechanism chosen; the
silent-drop diff) — **and propagate it into `arc-plan.md` (A-3 + a version entry)**. Feature
branch `arc06-slice03-schema-versioning`; not `main`/`release/1.0.x` directly.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-3) per LEDGER-DISCIPLINE v2.0 §A.
