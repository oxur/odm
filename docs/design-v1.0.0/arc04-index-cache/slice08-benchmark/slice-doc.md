# Slice 08 (Arc 04) — Benchmark harness (plan-of-record)

> Refs: ODD-0014 §4 (**"validate with a benchmark on a synthetic 100k-node corpus before
> declaring victory"**) + §1 (the `[P]` claims to promote to `[E]`), §2.1 (FIRST-full /
> SUBSEQUENT-incremental); arc04 `arc-plan.md` slice08 + Arc Ledger **A-13** (the
> compose row this slice satisfies). `depends_on:` slices 01–07 (the whole index engine
> + every wired consumer). *(Was slice06 → renumbered to 08, v1.10.)* **The arc
> capstone** — its close triggers the arc-close.
>
> **Why this slice exists:** ODD-0014's performance story is `[P]` (practitioner-lore):
> "single snapshot fine to tens of MB / sub-second load," "warm path scales with the
> delta, not the corpus." 0014 §4 is explicit that these stay `[P]` until measured on a
> synthetic 100k corpus. This slice builds that harness, **records the numbers**, and
> promotes the claims to `[E]`. It also **settles slice07's deferred question** — is the
> eager in-memory graph rebuild acceptable at 100k, or does it dominate?

## Goal

Measure the index engine at scale and promote 0014's `[P]` claims to `[E]`. **Done when**
a seeded synthetic-corpus harness measures cold-build, warm (no-change + small-delta),
snapshot load, and a consumer read at 1k/10k/100k; the numbers are recorded; and 0014's
performance claims are updated to `[E]` — asserting the **scaling** (warm ≪ cold;
delta-cost flat in corpus size; sub-second load at 100k) as the durable claim, with
absolute ms recorded as machine/toolchain context.

## Scope

**In:**

- **Seeded synthetic-corpus generator.** Reproducible (fixed seed) corpora at **1k / 10k
  / 100k** nodes with realistic frontmatter — a `part_of` forest, `depends_on`/other
  edges, gates+evidence, varied `origin`, some `decomposed` — so the benchmark exercises
  the real build/adapter/graph paths, not empty nodes.
- **Measurements** (recorded at each scale where relevant):
  1. **Cold build** — full walk + parse + hash; confirm ~`O(corpus)`.
  2. **Warm reconcile, no change** — the `lstat` sweep, **no re-parse**; confirm it is
     **dramatically faster than cold** (the incremental win, asserted as a ratio/scaling,
     not just an absolute).
  3. **Warm reconcile, small delta** (one changed file) — cost ≈ no-change + one
     re-parse; confirm it is **flat in corpus size** (delta-cost, not corpus-cost — §2.1).
  4. **Snapshot load** (decode) at 100k — validates "single snapshot file, sub-second
     load" (§1 `[P]` → `[E]`).
  5. **A consumer read** at 100k (`reconcile` → adapter → graph → e.g. `check`/`next`) —
     **settles slice07's open question**: is the eager in-memory rebuild acceptable at
     100k, or does it dominate? Record the verdict.
- **Record + promote.** Write the measured table (numbers + toolchain/machine context)
  durably, and **update ODD-0014's `[P]` tags to `[E]`** for the validated claims — the
  scaling as the durable `[E]`, absolutes as context.

**Out:** building deeper in-memory derived caching **even if** measurement (5) shows the
rebuild is slow — that is a *separate, post-benchmark* decision (a post-arc / A4-follow
optimization). This slice **measures and records the verdict**; it does not build caching
(0014 §2.4's eager-is-acceptable stands unless the data overturns it, which is then its
own scoped work).

## Design notes (settle here)

- **Scaling is the durable `[E]`; absolutes are context.** Machine/toolchain vary, so
  assert machine-independent properties (warm ≪ cold; delta-cost flat; load sub-second
  at 100k) and record the absolute ms alongside the environment. Don't pin a literal-ms
  claim as `[E]`.
- **Harness mechanism (CC's choice):** a reproducible release-mode harness — `benches/`
  (criterion), a feature-gated test, or a small `examples/` bin. Keep it simple and the
  numbers recorded; criterion is optional (we want end-to-end latencies at scale, not
  micro-bench statistics). Benches are excluded from coverage; the *generator* (if it's
  library code) is unit-tested.
- **Sandbox has no 1.85+ toolchain** — CC runs the harness locally (1.95.0) and records
  the numbers (attested); CI reproduces. The recorded absolutes are explicitly
  machine-tagged.

## Verification

The harness builds + runs (release) producing the table; the scaling assertions hold
(warm ≪ cold; small-delta flat across 1k→100k; load sub-second at 100k); 0014's `[P]`
claims are updated to `[E]`; clippy `-D warnings`; no `unsafe`; the generator (lib code)
coverage ≥ 90% (benches excluded). Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (the numbers reproduced via CI / local 1.85+). A-13 is
satisfied (the benchmark promoted `[P]`→`[E]`). **All 8 slices delivered — the arc-close
runs next:** the recomposition / silent-drop check across slices 01–08 + the class-(b)
compose rows (A-8…A-13) reproduced at arc scale, then **Arc 04 closes** (the index/cache
capability lands) and bubbles up to the project. Bubble up to `arc-plan.md` (A-8) at
slice close.
