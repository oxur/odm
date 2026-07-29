---
id: 01KYP5GYQE85EM1QHEBJRTAY5G
number: 536534500
type: artifact
schema: artifact/v1.1
name: 'Closing report — Slice 07 (Arc 05): incremental drift — probes-as-rules + the `.odm/` drift snapshot'
created: 2026-07-02
updated: 2026-07-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice07-incremental-drift-snapshot/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKJ818NYB2A2W2FRFY
---
# Closing report — Slice 07 (Arc 05): incremental drift — probes-as-rules + the `.odm/` drift snapshot

> Per LEDGER-DISCIPLINE v2.0 §A. CC's `done` is **proposed-done** — evidence at
> `attested` (built + run locally on a 1.95 toolchain). CDC reproduces (CI /
> local 1.85+) to elevate to `reproduced`.
>
> **Branch:** `arc05-slice07-incremental-drift-snapshot`, branched off
> `arc05-slice06-deferred-surfacing` (slices 01–06 unmerged; the prompt's named
> fallback). Rebase onto `main` once 01–06 merge.

## Per-row walk

**K-1 — probe `inputs` + class — done (attested).** `ProbeSpec::Shell` gained an
additive `inputs: Vec<String>` (skip-if-empty); `ProbeSpec::class()`/`inputs()`
classify: `file` → `InputDerived` (path is the input), `shell`+inputs →
`InputDerived`, `shell` w/o inputs → `Volatile` (ODD-0019 default). Adding the
field surfaced a compile error at every `ProbeSpec::Shell` site (the
not-`#[non_exhaustive]` net) — each handled. `probe_inputs_round_trip` +
`probe_class_from_inputs` → ok.

**K-2 — racy-correct input fingerprint — done (attested).** `capture_input` does
`lstat` + a content hash (the file probe's `hex_sha256`) when the file exists;
`input_changed` uses the cheap size/mtime signal, and re-hashes on the racy
window (`mtime_secs >= captured_at`) where the **hash is the authority**.
`input_fingerprint_catches_racy_same_size_edit` proves a same-size, same-second
edit is caught (a stat-only shortcut would miss it — the C2 failure);
`input_fingerprint_trusts_stat_outside_racy_window` covers the non-racy + delete
cases.

**K-3 — `.odm/` drift snapshot — done (attested).** `DriftSnapshot` with its own
`MAGIC` `ODMDRIFT` + `SNAPSHOT_VERSION` + trailing SHA-256, atomic-written via
`odm_store::atomic::write`; per fact: last `ProbeOutcome` + (`InputFingerprint`s
| `last_checked`). `drift_snapshot_round_trip` (encode/decode + persist/load) and
`drift_snapshot_corrupt_rebuilds` (missing/garbage/flipped-byte → self-heal, no
bad parse) → ok. **Body is JSON, not postcard** — see *Deviations*.

**K-4 — incremental reconcile — done (attested).** An input-derived fact whose
inputs are unchanged reuses its cached outcome (no re-probe); changed inputs →
re-probe; persisted atomically each run. Proven with a **counting shell probe**:
`incremental_skips_unchanged_reprobes_changed` shows the run count staying put
across unchanged runs and incrementing only on an input edit.

**K-5 — volatile skip + stamp — done (attested).** Volatile facts are not run on
the incremental path — their last outcome + `last_checked` are carried verbatim;
a `Full` run checks them and stamps `last_checked = now`.
`volatile_not_run_incrementally_stamps_last_checked` → ok (counter stays 1 across
the incremental run).

**K-6 — no index coupling — done (attested).** `git diff --stat --
crates/odm-index/` is empty; no `IndexRecord`/`index_frontmatters`/`FORMAT_VERSION`
in `crates/odm-reconcile/src` (the snapshot's version const is `SNAPSHOT_VERSION`,
keeping the guard a real tripwire). The fingerprint reuses the ODD-0014 *pattern*
+ the crate's own `hex_sha256`, not the index warm-path.

**K-7 — gates — done (attested).** clippy `-D warnings` → 0; no `unsafe`;
coverage (line) `snapshot.rs` 93.65%, `incremental.rs` 96.36%, `desired.rs` 100%,
`file.rs`/`runner.rs` ≥ 97% — all ≥ 90. Full workspace green (43 suites).

Rows: 7. Done: 7. Deferred: 0. No-op: 0.

## Silent-drop diff (scope-as-specified vs. scope-as-delivered)

None. Every "in" item shipped: the `inputs`/class probe-model extension, the
racy-correct input fingerprint, the persisted `.odm/` drift snapshot
(round-trip + self-heal), and the incremental reconcile (cutoff + re-probe +
volatile-skip-and-stamp), all as **library API**. Every "out" item stayed out:
**no** wiring into `reconcile_views`/commands, **no** staleness rendering (both
slice08), **no** `next`-withholding, **no** index change.

## Deviations / decisions flagged

1. **Snapshot body is JSON, not postcard** (the index's format). `ProbeOutcome`
   is an internally-tagged enum and `Id` is a string-newtype — both require a
   *self-describing* format; postcard serializes them but **cannot deserialize**
   them (it's not self-describing). So the drift snapshot uses `serde_json` for
   the body under the *same* header/checksum/atomic discipline. Not a divergence
   from ODD-0019 (which mandates the discipline, not the encoding); flagged
   because it differs from the index and is worth a reviewer's eye. A binary body
   would need postcard-specific DTOs (externally-tagged outcome, `Id` as a
   `String` field) — not worth it for a small, derived, rebuildable cache.
2. **Per-fingerprint `captured_at`** instead of the index's single snapshot-wide
   `index_timestamp` as the racy reference. Necessary: carried (unchanged) entries
   must keep their own racy context; a shared stamp advanced on write would
   de-racify them. A faithful *extension* of the ODD-0014 discipline, not a
   departure.
3. **`SNAPSHOT_VERSION`** (not `FORMAT_VERSION`) names the drift snapshot's
   version, so the K-6 guard grep for the index's `FORMAT_VERSION` stays a real
   tripwire (same technique as the slice02 G-5 / slice03 prose fixes).
4. **No amendment to ODD-0019** was needed — the model/class/snapshot shapes fit.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A)

**1. Did slice07 deliver A-7?** Yes — the freshness core (ODD-0019 §3) is a
tested library mechanism: probe classes, racy input fingerprint, persisted
self-healing snapshot, incremental cutoff + volatile skip. A-7 stays `attested`
until CDC reproduces.

**2. What it reveals for slice08 (the wiring — the meaty next step).**
  - **`reconcile_views` should call the incremental path, not `run_corpus`.**
    Today (slice04/06) `reconcile_views(store)` runs the full `run_corpus` and
    projects `Drift` + `Deferred`. slice08 should instead: run
    `reconcile_incremental(store, drift_snapshot_path)` (cheap, fresh) and project
    the resulting `DriftSnapshot` → `odm_core::rollup::{Drift, Deferred}`. This
    dissolves the slice04 "orient runs every probe" regression: bare `odm` re-hashes
    only changed inputs and runs **zero** volatile probes.
  - **Two projections from one snapshot.** The `DriftSnapshot`'s `FactEntry`s carry
    everything both projections need: `outcome` → `Drift` (drifted/errored/holds),
    and — joined with the corpus's `deferred` markers — the re-entry status →
    `Deferred`. slice08 writes `project_drift_from_snapshot` /
    `project_deferred_from_snapshot` paralleling slice04/06's `run_corpus`
    projectors. **Note:** the snapshot has node/fact ids + outcomes but **not**
    node number/name/`describe` (render identity) — slice08 joins those from the
    corpus docs (as slice03 did before slice04's enrichment), or enriches the
    snapshot. Recommend the join (keep the snapshot lean).
  - **The re-entry predicate (slice06) folds in for free.** A deferred node's
    `reenter_when` references a `desired_fact`; that fact is in the snapshot with
    a fresh (incremental) outcome. If the re-entry fact is **input-derived**, its
    readiness is now always-fresh at near-zero cost; if **volatile**, it carries
    honest staleness. slice08's deferred projection reads the fact's snapshot
    outcome instead of a live `run_corpus`.
  - **`odm reconcile` (slice03) becomes `reconcile_full`.** The explicit command
    should call `reconcile_full` (checks volatile too) and persist — the one place
    volatile probes run. slice08 rewires the command onto it.
  - **Staleness rendering + `next`-withholding.** "last checked Xm ago" for
    volatile facts and the deferred-`next` decision (ODD-0019 §6) are slice08's;
    the snapshot already carries `last_checked`, so rendering is a formatting step.
  - **Snapshot path convention.** slice07 takes the path as a parameter; slice08
    should settle a `.odm/drift.bin` (or `.json`) default alongside the index's
    `default_index_path`, gitignored (derived).

**3. The `ROLLUP.md` early-cutoff interaction (slice04 finding 2 / ODD-0019 §6).**
Now resolvable: `rollup` reads snapshot drift (fresh, cheap) rather than running
probes to skip a write. slice08 settles the exact seam.

**4. Reusable finding.** The counting-`#!/bin/sh`-probe pattern is the way to
assert "did/did-not run" for the exec-directly shell probe — reuse it for any
future cutoff proof.

Per the PM Part IV slice-close step, this bubble-up is propagated into
`arc-plan.md` (the A-7 row evidence + a v2.1 version-history entry), not only here.
