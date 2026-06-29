# CC Prompt — Slice 08 (Arc 04): Benchmark harness

Arc 04's capstone. Measure the index engine at 1k/10k/100k, record the numbers, and
promote ODD-0014's `[P]` performance claims to `[E]`. Also **settle slice07's deferred
question**: is the eager in-memory graph rebuild acceptable at 100k?

> **Start condition:** slices 01–07 CDC-verified / CI-green (the whole index engine +
> every wired consumer). If not in, hold.

## Read first
1. `slice08-benchmark/ledger.md` (8 rows).
2. `slice-doc.md` (same dir) and `../arc-plan.md` (slice08 + Arc Ledger A-13).
3. **ODD-0014 §4** ("validate on a synthetic 100k corpus before declaring victory") +
   **§1** (the `[P]` claims to promote) + **§2.1** (FIRST-full / SUBSEQUENT-incremental).
4. Reuse points: `odm-index` `build`, `reconcile`, `Snapshot::{persist,load}`, the
   adapter; `odm-store` for writing the synthetic corpus; the consumers (`check`/`next`).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `13-benchmarking` (criterion/`testing::Bencher` if used), `02-api-design.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence at `attested`; write the
  **Bubble-up to the arc** section at close).

## Task
1. **Seeded corpus generator:** reproducible (fixed seed) corpora at 1k/10k/100k with
   realistic frontmatter (`part_of` forest, edges, gates+evidence, varied `origin`, some
   `decomposed`). Same seed → identical corpus. Exercise the *real* paths.
2. **Measure + record** (a table): cold build (O(corpus)); warm no-change (≪ cold, no
   re-parse); warm small-delta (flat across 1k→100k); snapshot load + file size at 100k
   (sub-second); a consumer read at 100k (`reconcile`→adapter→graph→`check`/`next`).
3. **Settle slice07's question:** from the consumer-read measurement, record a **verdict**
   — is eager in-memory rebuild acceptable at 100k, or does it dominate (a flag for a
   post-arc optimization)? Measure + write the verdict; **do not build caching here.**
4. **Promote `[P]`→`[E]`:** update ODD-0014's `[P]` performance tags to `[E]` for the
   validated claims, asserting the **scaling** (warm ≪ cold; delta flat; load sub-second)
   as the durable claim, with absolute ms + the toolchain/machine recorded as context.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** Raise an amendment if a row is wrong/impossible.
- **Scaling is the durable `[E]`, not absolute ms** (machine-specific) — assert
  machine-independent properties; record absolutes + environment as context.
- **Measure, don't build:** if the consumer-read at 100k shows the eager rebuild
  dominates, **flag it** (a post-arc / A4-follow optimization) — do **not** build
  persistent in-memory caching in this slice (0014 §2.4 stands unless the data overturns
  it, which is its own scoped work).
- Reuse the existing engine (`build`/`reconcile`/`Snapshot`/adapter/consumers); the
  generator + harness are the new code. No `unsafe`; the generator (lib) ≥ 90% coverage;
  benches excluded from the gate.

## Deliverables
The harness runs (release) + the recorded table; the 0014 `[P]`→`[E]` diff; `ledger.md`
evidence per row (at `attested`); `closing-report.md` — per-row walk **plus the v2.0
Bubble-up to the arc** (note **all 8 slices delivered; the arc-close is next**). Feature
branch (`arc04-slice08-benchmark`); not `main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration cap;
your `done` is *proposed-done* (`attested`) → CDC reproduces (the numbers via CI / local
1.85+). On close, bubble up to `arc-plan.md` (A-8) per LEDGER-DISCIPLINE v2.0 §A — and the
**arc-close** (recomposition + class-(b) compose rows reproduced at arc scale) follows.
