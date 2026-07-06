# CDC Verification — Arc 05 / Slice 07: incremental drift — probes-as-rules + the `.odm/` drift snapshot

> Independent verification of CC's closed ledger (impl + close on
> `arc05-slice07-incremental-drift-snapshot`, commits `4cf2c82` + `67e58fa`), per
> LEDGER-DISCIPLINE v2.0 (slice scale, §A). CDC reproduces structural rows here; cargo rows
> route to CI / a local 1.85+ run.

## Environment constraint (disclosed)

CDC sandbox has no 1.85+ toolchain; CC built + ran on local 1.95.0 (attested → reproduced on
CI). Branch cut off `arc05-slice06-…` (01–06 unmerged) — rebase onto `main` when they merge.

## Row dispositions

**Row count:** 7 opened, 7 addressed (`done`). No silent drops. ✔ New modules:
`odm-reconcile/src/{snapshot.rs, incremental.rs}`.

**Reproduced by CDC (structural):**

- **K-1** — `ProbeSpec::Shell { …, inputs: Vec<String> }` (`desired.rs:68`); `ProbeClass
  {InputDerived, Volatile}` + `class()`/`inputs()` (`:87–114`): `file` → InputDerived,
  `shell`+inputs → InputDerived, `shell` w/o inputs → Volatile. `probe_inputs_round_trip` +
  `probe_class_from_inputs` present; additive (`skip_serializing_if`). ✔
- **K-2** — `capture_input(root, rel, captured_at)` (`incremental.rs:184`): `lstat` +
  conditional `hex_sha256` on the racy window (per-fingerprint `captured_at`).
  `input_fingerprint_catches_racy_same_size_edit` (same-size, same-second edit caught) +
  `input_fingerprint_trusts_stat_outside_racy_window` present. Reuses the file probe's
  `hex_sha256` — not the index warm-path. ✔
- **K-3** — `MAGIC = ODMDRIFT` (`snapshot.rs:51`), `SNAPSHOT_VERSION` (`:55`), checksum-signed
  (`signed[PREFIX_LEN..]`), body via `serde_json`; `Load::RebuildNeeded` on
  missing/garbage/flipped-byte. `drift_snapshot_round_trip` + `drift_snapshot_corrupt_rebuilds`
  present. Persist via `odm_store::atomic::write` (per ledger). ✔ (see ruling 1 on JSON)
- **K-4** — `incremental_skips_unchanged_reprobes_changed` uses a **counting `#!/bin/sh`
  probe** — unchanged input → run-count *stays* (no re-probe), edited → increments. The
  "count stays" assertion proves the cutoff, not just "same result" (mirrors A4 slice08's
  warm-skip proof). ✔
- **K-5** — `volatile_not_run_incrementally_stamps_last_checked`: full run checks + stamps;
  incremental leaves the counter (not run) and carries outcome + `last_checked`. ✔
- **K-6 (the invariant guard)** — `git diff --stat -- crates/odm-index/` → **empty**; `grep
  IndexRecord|index_frontmatters|FORMAT_VERSION crates/odm-reconcile/src` → **none** (version
  const named `SNAPSHOT_VERSION` to keep the guard a real tripwire). Separate `.odm/` artifact;
  input fingerprint reuses the ODD-0014 *pattern* + odm-reconcile's `hex_sha256`. **A4
  invariant untouched — A5's zero-index-change streak holds across all seven slices.** ✔
- **K-7 (no `unsafe`)** — grep empty. ✔

**Attested by CC (local 1.95.0), pending CI:** clippy `-D warnings` → 0; line coverage
**snapshot.rs 93.6% / incremental.rs 96.4%**; 43 suites green. → **PENDING CI.**

## Rulings on CC's flagged decisions

1. **JSON body, not postcard (vs the A4 index snapshot). Accepted — the right call, well
   reasoned.** `ProbeOutcome` is internally-tagged (`#[serde(tag="kind")]`) and `Id` is a
   string-newtype; **postcard (non-self-describing) can serialize but not round-trip
   internally-tagged enums** — JSON (self-describing) does. The perf rationale that drove
   postcard for the *100k-record* index does **not** apply to the drift snapshot (bounded by
   *declared facts* — far smaller), so JSON's larger/slower encoding is irrelevant, and the
   **same header/checksum/atomic discipline** is kept. Choosing the serializer to fit the
   data (rather than distorting `ProbeOutcome` to fit postcard) is correct. odm now has two
   snapshot encodings (index=postcard, drift=JSON) — a *justified* divergence by data shape,
   not drift; flagged in ledger/closing-report/arc-plan.
2. **Per-fingerprint `captured_at` (vs the index's single snapshot-wide stamp). Accepted, and
   a genuine refinement.** Each input fingerprint carrying its own racy reference means a
   carried-unchanged entry keeps *its* racy context, rather than being re-based against a
   fresh snapshot-wide stamp — more correct for a snapshot where entries are written at
   different times. Better than mirroring the index's model wholesale.
3. **`SNAPSHOT_VERSION` (not `FORMAT_VERSION`) naming** — deliberate, so the K-6 guard grep
   stays a real tripwire. Accepted (same discipline as slice02's identifier-hygiene for its
   guard).
4. **Adding `inputs` fired the not-`#[non_exhaustive]` compile-error net at every Shell
   site.** Accepted — the intended safety net (slice01/02 discipline) working again.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — A-7: the probe-model extension + racy input fingerprint +
  `.odm/` drift snapshot + incremental runner, as a library mechanism (slice08 wires it).
- **Silent-drop diff honest?** ✔ — 7/7; the JSON deviation, the per-fingerprint stamp, and
  the slice08 boundary are all disclosed.
- **Findings + arc-plan?** ✔ — CC propagated the bubble-up itself (A-7 + arc-plan v2.1), with
  a **concrete slice08 wiring plan**: point `reconcile_views` at the incremental path
  (dissolving the slice04 orient-runs-probes regression); project `DriftSnapshot` →
  `Drift`+`Deferred` (joining render-identity); rewire `odm reconcile` onto the full run;
  render "last checked Xm ago"; settle the deferred-`next` decision + the `ROLLUP.md`
  early-cutoff seam; add a gitignored `.odm/drift` default path. The slice06 re-entry
  predicate **folds in for free** (it's a snapshot fact). All correct — slice08 is now a
  wiring + loose-ends-settle slice.

## Verdict

**Arc 05 / Slice 07 CDC-verified on structure; all four flags ruled; cargo rows pending CI.**
The freshness core is landed: probes are input-tracked rules, the input fingerprint is
racy-correct (a same-tick same-size edit is caught — the C2 failure it would otherwise hide),
the `.odm/` drift snapshot persists/self-heals on the A4 discipline, and the incremental
reconcile re-probes *only* what changed (proven by run-count, not result-equality) while
volatile facts are stamped-not-run. Zero index coupling — A5's clean streak holds. A-7
attested-on-close; flips `done` on CI green. **A5 at 7/8.**

**Only slice08 remains** — the wiring + honest-staleness rendering — and CC has handed it a
concrete plan. After slice08: the **A5 arc-close** (composition check across A-1…A-13, the
class-(b) rows reproduced at arc scale) and the third arc lands.

CDC: planning thread, 2026-07-02. Iterations used: 1.
