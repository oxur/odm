---
id: 01KYP5GZH4EY8294KG0X3BCJQK
number: 534994400
type: artifact
schema: artifact/v1.1
name: 'Slice 08 (Arc 05): freshness on every command + honest staleness'
created: 2026-07-06
updated: 2026-07-06
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice08-freshness-wiring/ledger.md
  class: ledger
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKT5WD5R931YMT2FNE
---
# Slice 08 (Arc 05): freshness on every command + honest staleness

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. **Arc capstone** — wires
> slice07's freshness mechanism into every command + settles the two open loose ends; the
> arc-close runs next.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| L-1 | `reconcile_views` runs the **incremental** path (slice07) — orient/rollup get fresh input-derived drift at near-zero cost; **bare `odm`/orient runs ZERO volatile probes** (the slice04 regression dissolved); an input-derived fact whose input changed *is* re-probed | `cargo test -p odm-cli orient_runs_zero_volatile_probes` + `orient_reprobes_changed_input` → ok (a counting volatile probe stays at 0 on bare orient) | serious | ODD-0019 §3.2 / slice04 finding | done | attested — `reconcile_views` → `reconcile_incremental(store, default_drift_path)`; counting-probe tests: bare orient runs the volatile probe **0×** (twice), and the input-derived probe runs 1× → carried → 2× only on an input edit | The headline: freshness for free, regression gone. `reconcile_views` → `reconcile_incremental` (not full `run_corpus`). |
| L-2 | `odm reconcile` (explicit) runs the **full** path — re-probes all input-derived **and volatile** facts, refreshing + re-stamping the snapshot | `cargo test -p odm-cli reconcile_command_runs_full_refreshes_volatile` → ok (volatile counting probe increments; `last_checked` re-stamped) | serious | ODD-0019 §3.4 (explicit refresh) | done | attested — `reconcile()` → `reconcile_full(store, default_drift_path)`; counting probe 1→2 across two reconciles; `reconcile --json` carries a numeric `last_checked` on the volatile fact | The sanctioned "refresh volatile now" verb. `odm reconcile` → `reconcile_full`. |
| L-3 | **Honest-staleness rendering**: rollup/orient render volatile facts as **"last checked Xm ago"** (from `last_checked`), input-derived as fresh; a never-checked volatile fact reads "not yet checked — run `odm reconcile`" (no fabricated freshness); `--json` carries staleness **additively** (no schema bump) | `cargo test -p odm-cli {rollup,orient}_render_volatile_staleness` + `orient_never_checked_volatile` + `{rollup,orient}_json_staleness_additive` → ok AND `grep -rnE "rollup/v1\|orient/v1" crates/odm-cli/src` unchanged | serious | ODD-0019 §3.4 | done | attested — `staleness_suffix`/`humanize_age` render "last checked Xs/Xm ago"; never-checked → "not yet checked — run `odm reconcile`" (asserts NOT "last checked"); JSON `freshness:{kind,at}` + `unchecked[]` added, `rollup/v1`·`orient/v1` markers unchanged | The honest half of the model — say "last checked", never pretend fresh. Additive like slice04/06. |
| L-4 | The drift snapshot lives at a **`.odm/drift` default path** and `.odm/` is **gitignored** (never truth, never committed) | `cargo test -p odm-cli drift_snapshot_default_path` → ok AND `grep -nE "^\.odm/\|/.odm/" .gitignore` | serious | ODD-0019 §3.3 / 0013 (`.odm/` gitignored) | done | attested — `default_drift_path(root) = <root>/.odm/drift`; test asserts the file exists after `reconcile`; `.gitignore:28` = `.odm/` | Mirrors the index's `.odm/` home; the snapshot is derived, never committed. |
| L-5 | **`ROLLUP.md` early-cutoff is drift-aware** (settles slice04 finding #2): a **drift change with no corpus change** → `ROLLUP.md` **regenerates**; no drift change + no corpus change → **skipped** (byte-identical). A persisted rollup can no longer hide stale drift | `cargo test -p odm-cli rollup_regenerates_on_drift_change` + `rollup_skips_when_drift_and_corpus_unchanged` → ok | serious | slice04 finding #2 / A4 slice07 cutoff | done | attested — cutoff keys on `content_fingerprint` = sha256(`RollupJson`) (corpus+drift+deferred, absolute stamps); toggling a file-probe flag (no corpus change) regenerates `ROLLUP.md`; an unchanged corpus+drift is byte-identical (skipped) | The cutoff now keys on the drift projection too (incremental drift is cheap — slice07). Closes the "cutoff hides stale drift" gap. |
| L-6 | **Deferred-`next` decision settled — `next` stays graph-pure**: `next` answers graph-readiness (deps+gates) and is **unaffected** by a `deferred` marker; deferred is surfaced only in the reconcile-aware views (rollup/orient). Documented as a deliberate layering boundary (closes the slice06 limitation as **decided-not-withheld**) | `cargo test -p odm-cli next_unaffected_by_deferred_marker` → ok AND the rationale is in the slice06/08 docs + a code comment on the `next` reader | correctness | slice06 limitation / ODD-0019 | done | attested — a deferred, dep-free slice still appears in `odm next` (not withheld, exit 0); layering-boundary rationale is a doc-comment on `commands::next` + this slice's docs | Withholding would need indexing the marker (rejected invariant/`FORMAT_VERSION`) or `next` running a reconcile (re-introduces the orient regression). Graph-ready ≠ parked; both surfaces are correct. |
| L-7 | **No `odm-index` change**; clippy `-D warnings`; no `unsafe`; coverage ≥ 90% (line) for the touched `odm-cli`/`odm-reconcile` paths | `git diff --stat -- crates/odm-index/` shows **no** change AND `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-cli/src crates/odm-reconcile/src` AND `cargo llvm-cov --summary-only -p odm-cli -p odm-reconcile` → **line** ≥ 90% | serious | A4 invariant / CLAUDE.md | done | attested — `git diff crates/odm-index/` empty; clippy `-D warnings` exit 0; no `unsafe`; line cov: json.rs 100%, rollup.rs 96.96%, orient.rs 94.60%, cli reconcile.rs 93.48%, incremental.rs 98.21%, snapshot.rs 93.65% (all ≥ 90) | Wiring slice — the freshness home is the drift snapshot, not the index. A5's zero-index-change streak closes the arc intact. |

## What Worked

- **One seam flipped the whole arc.** The slice04/06 projectors already read a
  `(Drift, Deferred)` pair from `reconcile_views`; swapping its body from the full
  `run_corpus` to `reconcile_incremental` + a snapshot/corpus join dissolved the
  slice04 regression without touching any caller. The projection is the stable
  interface; the compute behind it was free to change.
- **The counting probe *proves* the invariant.** "orient runs zero volatile
  probes" is not "it's faster" — a `#!/bin/sh` script that appends one line per
  run lets the test assert the run count is exactly 0 (and exactly 1→2 for the
  explicit reconcile). Reused straight from slice07's playbook.
- **Absolute stamps in, relative render out.** The snapshot stores an absolute
  `last_checked`; only the human render computes "Xm ago". So `content_fingerprint`
  (sha256 over the `RollupJson` projection, which carries the absolute stamp) is
  stable across wall-clock ticks — the drift-aware cutoff regenerates on a real
  drift change but stays byte-identical when nothing moved.
- **Never-checked is a first-class state.** Modeling volatile-never-run as
  `UncheckedFact` (no outcome) — distinct from holds/drifted/errored — made honest
  staleness fall out of the type: there is no code path that can print "fresh" for
  something never probed.
- **Additive JSON held again.** `freshness:{kind,at}` on drift entries + an
  `unchecked[]` array, no `rollup/v1`·`orient/v1`·`reconcile/v1` bump — the same
  discipline as slice04/06, verified by unchanged schema markers + shape-lock tests.

## Deviations / decisions flagged

1. **Two reads per bare command** (`reconcile_incremental` internally
   `load_all`s the corpus, then `reconcile_views` `load_all`s again for render
   identity — number/name/describe, which the lean snapshot deliberately omits).
   Both are cheap store reads, **not probes**; the L-1 invariant (zero volatile
   probes) is untouched. Kept the snapshot lean (slice07's recommendation) rather
   than duplicate render identity into it. Flagged as an accepted read-cost
   tradeoff, not a probe-cost one.
2. **Pre-slice08 deferred tests now `reconcile`-first.** slice06's
   ready/waiting tests used a *volatile* re-entry fact and relied on `odm rollup`
   probing it live. Under L-1, bare rollup no longer runs volatile probes, so a
   never-checked re-entry fact honestly reads "waiting (not yet checked)". The
   tests were updated to run `odm reconcile` first (see the Bubble-up) — a genuine
   slice06→08 behavior change, disclosed, not a silent test-massage.
3. **No amendment to ODD-0019** was needed — the incremental wiring, the
   `Freshness`/`UncheckedFact` model, and the drift-aware cutoff all fit the design
   of record as written.

## Closure

Closed at commit `f4b5ac7` on 2026-07-06. Verified by: CC (proposed-done, attested);
CDC to reproduce. Rows: 7. Done: 7. Deferred: 0. No-op: 0.
On close → **the A5 arc-close** (composition check A-1…A-13; class-(b) rows reproduced at arc scale).
