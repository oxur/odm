# Slice 07 (Arc 05): incremental drift — probes-as-rules + the `.odm/` drift snapshot

> Plan-of-record for A5 slice07 — the **freshness core** (ODD-0019). The heart of the v1.8
> redirection: drift stops waiting on a scheduled/manual `reconcile` and starts riding the
> A4 stat-cache. This slice delivers the **mechanism** (the probe-model extension + the
> persisted snapshot + the incremental runner) as library API; **slice08 wires it into every
> command** + renders honest staleness.

## Goal

Make a drift check **cheap and incremental**: fingerprint each input-derived fact's declared
inputs, re-probe **only** what changed, carry the rest from a persisted `.odm/` drift
snapshot, and leave volatile facts untouched (stamped with "last checked"). After this slice,
`odm-reconcile` can answer "what drifted?" in cost proportional to the change — the
substrate every command will lean on in slice08.

## Design of record: ODD-0019

Read ODD-0019 (`docs/design/04-accepted/0019-…`) — this slice implements §3 (the two classes,
the incremental reconcile, the snapshot) and §4 (the lineage: verifying traces / Salsa
backdating / Ninja `restat` / Buck2 dep-files / git racy-correctness).

## Scope — in

1. **Probe-model extension** (`odm-core`, additive): a `desired_fact` gains optional
   **`inputs`** (paths/globs). Classification:
   - a **`file`** probe is input-derived — its `path` **is** its input (auto; no `inputs`
     needed);
   - a **`shell`** probe **with** `inputs` is input-derived (those files are its inputs);
   - a **`shell`** probe **without** `inputs` is **volatile** (no fs signal — ODD-0019's
     default: honest staleness, never false-fresh).
   Additive round-trip (slice01 discipline: internally-tagged, `skip_serializing_if` for the
   absent default, proptest `modeled` extended).
2. **Racy-correct input fingerprint** (`odm-reconcile`): an input's fingerprint is
   **size + mtime + a conditional content-hash** on the racy `mtime >= snapshot-timestamp`
   window (the ODD-0014 discipline — **never stat-only**), reusing the file probe's
   `hex_sha256` (slice02). A same-tick, same-size edit to an input **must** invalidate the
   cached outcome (mirrors A4/slice03's racy test — it would be missed under a stat-only
   shortcut, which is the exact C2 silent-staleness failure).
3. **Persisted `.odm/` drift snapshot** (`odm-reconcile`): a versioned artifact — own
   `MAGIC` (e.g. `ODMDRIFT`) + format-version + checksum, **atomic write via
   `odm_store::atomic::write`** (reused), following the A4 header discipline. Per fact it
   records: the last `ProbeOutcome`, plus the **input fingerprint** (input-derived) or a
   **last-checked timestamp** (volatile). Round-trips; **corrupt/missing → rebuild** (self-heal,
   like the index).
4. **Incremental reconcile** (`odm-reconcile`): given the snapshot —
   - input-derived fact whose inputs are **unchanged** → the **cached outcome stands** (early
     cutoff, no re-probe);
   - inputs **changed** → **re-probe**, update the entry;
   - **volatile** fact → **not run**; carry its last outcome + stamp last-checked;
   - persist the updated snapshot (atomic). Cost is proportional to the change, not the corpus
     of probes.

## Scope — out (named, not dropped) → slice08

- **Wiring into the command read-path.** `reconcile_views` (slice06) still calls the full
  `run_corpus` this slice; **slice08 flips it to the incremental path** and runs it before
  every command. slice07 delivers + tests the incremental runner as **library API**.
- **Honest-staleness rendering** ("last checked Xm ago"; bare `odm` runs zero volatile
  probes) — **slice08**.
- **`next`-withholding of deferred nodes** — the deferred slice06 follow; ODD-0019 routes it
  to where the snapshot's home is decided, i.e. here/slice08. *Not built this slice* unless it
  falls out for free; flag if so.

## Boundaries / decisions to flag

- **No `odm-index` coupling.** The drift snapshot is a **separate `.odm/` artifact**; the
  input fingerprint reuses the ODD-0014 *pattern* + odm-reconcile's own `hex_sha256`, **not**
  odm-index's record-coupled warm-path. **No `IndexRecord`/adapter/`FORMAT_VERSION` change** —
  the A4 adapter-fidelity invariant is untouched (a row guards it). *(This keeps A5's
  zero-index-change streak.)*
- **DRY note (flagged, not acted on):** odm now has two racy-fingerprint users — the index
  warm-path (record-coupled) and this input-fingerprint (fact-coupled). Both are thin; a
  shared primitive isn't worth extracting for two users (YAGNI). Revisit if a third appears.
- **Volatile default.** A shell fact without `inputs` is volatile — the conservative,
  honest choice (ODD-0019). Flag if an author clearly wants "always fresh" for such a fact
  (that's the slice08 `--strict`/explicit-refresh story, not a slice07 default).

## Verification approach

`odm-core` + `odm-reconcile` library tests (no command wiring yet):

- `inputs` round-trips; classification (file → input-derived; shell+inputs → input-derived;
  shell w/o inputs → volatile).
- input fingerprint is racy-correct: a same-size, same-mtime input edit is caught (the
  A4/slice03 racy pattern, applied to a probe input).
- the drift snapshot round-trips; corrupt/missing → rebuild.
- incremental: unchanged inputs → cached outcome, **no re-probe** (assert the probe did not
  run, e.g. a counting/side-effecting probe); changed inputs → re-probe; volatile → not run,
  last-checked stamped.
- **no** change under `crates/odm-index/`.
- clippy `-D warnings`; no `unsafe`; coverage ≥ 90% line.

Cargo rows are CC-`attested`, reproduced via CI / local 1.85+.

## Exit criteria

Ledger K-1…K-7 reach a final status: the probe model carries `inputs`/volatility; the input
fingerprint is racy-correct; the `.odm/` drift snapshot persists/loads/self-heals; the
incremental reconcile re-probes only changed inputs and skips volatile (stamping
last-checked); no index coupling; gates pass. The mechanism is ready for slice08 to wire into
every command.

> **Render/convention:** `writeln!` + `tabled` where any output is involved (none this slice —
> library only). Reuse `odm_store::atomic::write`, `hex_sha256` (slice02), the A4 snapshot
> header discipline.
