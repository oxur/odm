---
id: 01KYP5GY5SA7BGFHYK03MX4NJ7
number: 557558400
type: artifact
schema: artifact/v1.1
name: 'CC Prompt — Slice 07 (Arc 05): incremental drift — probes-as-rules + the `.odm/` drift snapshot'
created: 2026-07-02
updated: 2026-07-02
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice07-incremental-drift-snapshot/cc-prompt.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKJ818NYB2A2W2FRFY
---
# CC Prompt — Slice 07 (Arc 05): incremental drift — probes-as-rules + the `.odm/` drift snapshot

The **freshness core** (ODD-0019) — the heart of the v1.8 redirection. Make drift **cheap +
incremental**: fingerprint each input-derived fact's inputs, re-probe **only** what changed,
carry the rest from a persisted `.odm/` drift snapshot, leave volatile facts untouched
(stamped "last checked"). This slice is the **library mechanism**; **slice08 wires it into
every command** + renders honest staleness. This is the meatiest slice of A5 — read carefully.

> **Start condition:** slices 01–06 CDC-verified. Branch off `main`:
> **`arc05-slice07-incremental-drift-snapshot`** (not `main`). If 01–06 unmerged, branch off
> slice06's branch and flag for rebase.

## Read first
1. **ODD-0019** (`docs/design/04-accepted/0019-incremental-drift-probes-as-rules-honest-staleness.md`)
   — the design of record. This slice implements §3 (two classes, incremental reconcile,
   snapshot) + §4 (lineage: verifying traces / Salsa / Ninja `restat` / Buck2 dep-files / git
   racy-correctness).
2. `slice07-incremental-drift-snapshot/ledger.md` (7 rows) + `slice-doc.md` (same dir).
3. `../arc-plan.md` — A5 capability + `## Freshness model`, Arc Ledger (this slice closes
   **A-7**), v1.8/v2.0.
4. Reuse points — **do not reinvent, do not couple to `odm-index`:**
   - `crates/odm-reconcile/src/file.rs` — `hex_sha256` (your content hash for input files).
   - `crates/odm-store` — `atomic::write` (your snapshot's atomic persist).
   - `crates/odm-index/src/snapshot.rs` — the **header discipline** to *mirror* (own `MAGIC`
     `ODMDRIFT`, format-version, checksum) — mirror the pattern, do **not** import the index's
     record-coupled `Snapshot`.
   - `crates/odm-index/src/warm.rs` — the racy-correct algorithm to *mirror* (size + mtime +
     conditional content-hash on `mtime >= stamp`); apply it to probe inputs.
   - slice01 `desired_facts` (`odm-core/desired.rs`) — where `inputs` is added.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then API design, error handling, serde evolution.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence `attested`; per-row walk +
  **Bubble-up to the arc** at close; **propagate it into `arc-plan.md` (A-7 + version entry)
  yourself**, as in slices 02–06).

## Task
1. **Probe model + class** (K-1): add optional `inputs` to a `desired_fact`; classify —
   `file` → input-derived (path is the input); `shell`+`inputs` → input-derived; `shell`
   w/o `inputs` → **volatile**. Additive round-trip.
2. **Racy-correct input fingerprint** (K-2): size + mtime + conditional `hex_sha256` on the
   racy window; a same-tick same-size input edit must be caught (mirror A4/slice03's racy
   test). **Never stat-only** — a missed input change is the C2 silent-staleness failure.
3. **`.odm/` drift snapshot** (K-3): own `MAGIC` `ODMDRIFT` + version + checksum; atomic
   write via `odm_store::atomic::write`; per fact: last `ProbeOutcome` + (fingerprint |
   last-checked); round-trips; corrupt/missing → rebuild.
4. **Incremental reconcile** (K-4): unchanged inputs → cached outcome (no re-probe — prove it
   with a side-effecting/counting probe); changed → re-probe; persist atomically.
5. **Volatile skip + stamp** (K-5): volatile facts not run incrementally; carry last outcome
   + stamp last-checked.
6. **No index coupling** (K-6): drift snapshot is a separate artifact; **no change under
   `crates/odm-index/`**; reuse the *pattern* + your own `hex_sha256`, not the index's record
   machinery.
7. **Gates** (K-7): clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** If the model/class shape or the snapshot format needs to
  differ from ODD-0019, raise an amendment against the ODD (don't quietly diverge from the
  design of record).
- **Racy-correctness is non-negotiable** (K-2) — stat-only would silently miss input changes,
  the exact failure A5 exists to kill. Test the same-tick same-size case.
- **No `odm-index` change.** Mirror the pattern; don't import/extend the index's `Snapshot`
  or `IndexRecord`. (If you find yourself wanting to, stop and flag — it's the rejected
  design.)
- **Stay in scope:** **do not wire into `reconcile_views`/commands and do not render
  staleness** — that's slice08. Deliver + test the incremental runner + snapshot as library
  API. (If `next`-withholding falls out trivially, flag it; otherwise leave it for slice08.)
- **Volatile default:** shell w/o inputs = volatile (honest). Don't invent an always-run
  default.

## Deliverables
The probe-model extension + racy input fingerprint + `.odm/` drift snapshot + incremental
runner, with `ledger.md` evidence per row (`attested`); a `closing-report.md` — per-row walk
**plus the Bubble-up to the arc** (did slice07 deliver A-7; what it reveals for slice08's
wiring — especially how `reconcile_views` should call the incremental path and how the
re-entry predicate (slice06) folds in; the silent-drop diff) — **and propagate that into
`arc-plan.md` (A-7 + a version entry)**. Feature branch
`arc05-slice07-incremental-drift-snapshot`; not `main`.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-7) per LEDGER-DISCIPLINE v2.0 §A.
