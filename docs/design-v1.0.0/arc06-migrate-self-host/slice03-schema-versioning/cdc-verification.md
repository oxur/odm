# CDC Verification — Slice 03 (Arc 06): schema versioning (ODD-0020)

> Independent CDC reproduction of CC's proposed-done (`attested`) report.
> Structural rows reproduced by code inspection against `release/1.0.x`; cargo
> rows (test pass, clippy, coverage %, workspace green) are **attested-pending-CI**
> — the CDC sandbox carries no 1.85+ toolchain (edition 2024), so executable
> evidence flips to `reproduced` on CI green. Commits `5b5dd43` (work) + `fc4ccbc`
> (close SHA) on `arc06-slice03-schema-versioning`, branched off the slice02 tip.

## Verdict

**PASS-WITH-NOTES.** All six ledger rows reproduce structurally; the design of
record (ODD-0020 §2/§3/§4/§5) is faithfully realized. Two calibration notes below
(one worth a future validity check, one a naming precision) — neither blocks the
slice. Cargo-executable rows reproduce on CI.

## Row-by-row

| Row | Reproduced | Evidence (CDC-observed) |
|-----|-----------|-------------------------|
| **V-1** schema field + absent⇒v0.1 | ✅ structural | `crates/odm-core/src/schema.rs`: `SchemaVersion { major, minor }` with `CURRENT=v1.0`/`LEGACY=v0.1`, `Ord`-based `is_newer_than_current`; `SchemaMarker { node_type, version }` ⇄ `<type>/vMAJOR.MINOR` (Display/FromStr, custom Serialize/Deserialize), malformed rejected. `frontmatter.rs`: `schema: Option<SchemaMarker>` (`skip_serializing_if` → legacy round-trips byte-identically); `schema()`, `schema_version()` = `map_or(LEGACY, …)` (**absent⇒v0.1, computed not written**), `stamp_schema()`/`with_schema()`. |
| **V-2** new nodes stamp `<type>/v1.0` | ✅ structural | `odm-cli/src/commands.rs:220` — `fm.stamp_schema()` in the `new` path (stamps the node's **own** type). Migrate create path also stamps (V-4). |
| **V-3** per-type field-validity = check Error | ✅ structural | `odm-core/src/check.rs`: `content_validity()` → `check_field_validity` gated on `NodeType::is_document()` / `is_work()`; work-only `{desired_facts, deferred}` flagged on document nodes, doc-only `{supersedes, affects}` flagged on work nodes → `Violation::FieldNotValidForType`. Severity: `violation_severity` maps `StaleDoc | DanglingReenterWhen => Warning`, `_ => Error` — so this is an **Error** (code `wrong-type-field`). Folded into check entries at `commands.rs:1160`. **Reads the store** (full corpus), not the index — correct: the index omits `desired_facts`/`deferred`/`schema`. |
| **V-4** migrate stamps v1.0 / legacy untouched | ✅ structural | `odm-migrate/src/mapping.rs:167` — `fm.stamp_schema()` in `build_node`. Legacy `docs/design` ODDs carry **no** frontmatter `schema` field (verified: the only `schema:` in the tree is a **body code-example** in the ODD-0020 doc, lines 42; frontmatter lines 1–13 have none) → legacy stays `v0.1` by absence. |
| **V-5** backfill folded into migrate; 13 nodes `odd/v1.0` | ✅ structural + on-disk | `odm-migrate/src/lib.rs:321` `backfill_schema(store, mode)`: idempotent (`schema().is_some() → continue`), dry-run-safe (`persist` skipped under `is_dry_run`), never-deletes (stamp+persist only); folded into `migrate` at `lib.rs:236`, runs **before** import. On disk: all **13** `nodes/2026/07/*.md` carry `schema: odd/v1.0`, none missing (12 slice02 upgraded + ODD-0020 imported). |
| **V-6** forward-compat + gates + no regression | ✅ structural / ⏳ cargo pending-CI | `check.rs:230` `check_schema_version` → `Violation::UnsupportedSchema` when `marker.version.is_newer_than_current()` (a newer **minor** *or* major — a `v1.0` reader reports `v1.1`, never misparses); severity Error (code `unsupported-schema`); marker fully parsed. clippy `-D warnings` / no `unsafe` / line-cov (schema 96.55%, check 95.34%, frontmatter 99.52%, migrate lib 99.21%) / `cargo test --workspace` (50 suites) — **attested by CC, reproduced on CI**. |

## Bubble-up (checked)

Propagated to `arc-plan.md` correctly: **A-3** `attested` (cargo pending-CI, → `done` on reproduce); **A-12** compose row `mechanism-complete` (to be reproduced at arc-scale at arc-close, never inherited); **v1.6** version entry with the slice04 hand-off note. Stable-ID discipline honored — when schema-versioning was inserted as slice03, A-11 (retire) and A-12 (schema compose) were **appended**, not renumbered.

## Calibration notes (accepted, watched — not blockers)

1. **"Per-type" is implemented as "per-class" (work vs document).** The validity
   buckets partition on `is_work()` / `is_document()`, so a strictly per-*subtype*
   collision within a class is **not** caught: an `odd` carrying `affects` (adr's
   field) or an `adr` carrying `supersedes` (odd's field) would pass, since both
   are document types and the doc-only bucket permits both. ODD-0020 §2 says
   "per-type"; the mechanism is per-class. For the fields actually in the model
   this is harmless (supersedes/affects are both document-level lineage and don't
   corrupt each other), and it's additively tightenable. Named here so the ledger's
   "per-type" language isn't read as stricter than the code. Candidate future
   refinement (cheap), not an amendment now — flag if a real odd/adr field
   collision needs distinguishing.

2. **`schema`-type vs `type:` mismatch is not yet a check** (CC deviation #4). The
   marker redundantly carries the node type; `stamp_schema` always uses the node's
   own type, so odm-created nodes never diverge — but a hand-edited/wrong-type
   import could carry `type: odd` + `schema: slice/v1.0` and pass `check`. Correctly
   scoped out. A one-line future validity check (`marker.node_type == fm.node_type()`)
   would close it. Watch, don't block.

3. **Double store-read** (CC deviation #2): `content_validity` runs over
   `store.load_all()` alongside the index reconcile — the same A5-flagged read-cost
   tradeoff (not a probe cost), consistent with the accepted "reconcile reads the
   store" pattern. `check_reenter_when` shares the index-blindness; noted out of
   scope. No action.

## Reproduced

Structural rows reproduced by CDC (code + on-disk state) against `release/1.0.x`.
Cargo rows flip `attested → reproduced` on CI green. No amendment to ODD-0020 was
required — the marker shape, per-type (per-class) validity, and forward-compat all
fit the design of record. Slice03 delivers **A-3** and the **A-12** mechanism;
the `nodes/` corpus is now uniformly `odd/v1.0`, so slice04's plan-set lands
already stamped.
