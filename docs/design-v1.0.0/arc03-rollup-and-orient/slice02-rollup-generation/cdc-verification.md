# CDC Verification — Arc 03 / Slice 02: Rollup generation

> Independent verification of CC's closed ledger (impl `4a50ac2`; closed `58cd44e`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran in
> this session; **attested** = CC's evidence (incl. CC's local 1.95.0 run), not
> independently reproduced here.

## Environment constraint (disclosed)

Same as slice01: the Cowork sandbox has no 1.85+ toolchain, so cargo
build/test/clippy/coverage *executions* route to CI / a local 1.85+ run. CDC
reproduces structural rows by reading the branch `arc03-slice02-rollup-generation` at
`4a50ac2`. CC reproduced all cargo rows locally on rustc 1.95.0 — held as **attested
pending CI** (the independence gate is Duncan's push to green CI).

## Row dispositions

**Row count:** 11 opened, 11 addressed. No silent drops. ✔

**Reproduced by CDC (structural, at `4a50ac2`):**

- **R-1** — `Rollup::assemble(&[Frontmatter], &GateSets, threshold) -> Rollup`
  (`odm-core/src/rollup.rs`) is a pure assembly: builds nothing from disk, maps
  existing ops (`Recomposition`, `NodeGraph::next`/`blocked`, `active_tears`,
  `Satisfaction`, `origin()`) into owned view structs. No graph/recompose logic
  reimplemented. ✔ *(signature carries `threshold` — see ruling 1)*
- **R-3** — status vectors built from `GateSet::sequence()`; an unreached gate carries
  `evidence: None` (`GateStatus`); render shows `gate=evidence` / `gate=–`. Gate order,
  not alphabetical. ✔
- **R-4 + ruling 2** — `ready: Vec<ReadyNode>` where `ReadyNode { node, soft:
  Vec<SoftDep> }`; blocked is partitioned to exclude the ready frontier
  (`ready_ids`), so no node appears in both. The soft-satisfaction signal is **not
  lost**: `render_ready` prints `- soft: <dep> at evidence=<level>` on the ready node
  (0013 §4.4's "`next` lists it but flags it"), and `BlockReason::SoftSatisfied`
  renders `- low-evidence: <dep> at evidence=X (needs Y)` for nodes held by a *hard*
  dep that also carry a soft one. Blocked entries name unsatisfied edges. ✔
- **R-5** — active tears render with `because: <rationale>`, sourced via the shared
  `odm_core::graph::frontmatter_tears` (slice01's `TornEdge`). ✔
- **R-7 / R-8** — `Drift` renders "## Drift / Not yet tracked (A5)" unconditionally
  (Q-A3-2); `Deferred` slot defined but always empty, render emits nothing, no
  `deferred` status variant introduced (Q-A3-1). ✔
- **R-11 (no `unsafe`)** — `grep -RnE '\bunsafe\b' crates/odm-core/src crates/odm-cli/src`
  → no matches. ✔
- **Dedup (no partial adoption):** `frontmatter_tears` lives once in
  `odm-core/src/graph.rs:64` and is the single bridge used by both the rollup and the
  CLI `check` path (`commands.rs:1077`). ✔

**Attested by CC (local rustc 1.95.0), pending CI:** the cargo executions —
`cargo test --workspace` → 201/202 passed, 0 failed; clippy `--all-targets -- -D
warnings` → exit 0; `fmt --check` clean; line coverage **odm-core 98.69% / odm-cli
92.82%** (rollup.rs 98.62% / 98.17%), both ≥ 90%. → **PENDING CI.**

## Rulings on CC's flagged items

1. **`assemble` takes a `threshold` beyond the slice-doc's "corpus + GateSets"
   shorthand.** **Accepted — my shorthand was incomplete, not CC's scope creep.** The
   evidence threshold is required for soft-satisfaction in ready/blocked and is the
   same `odm.toml` value `check`/`next` read; the model stays pure (no I/O). I've
   corrected the slice-doc to name the threshold (doc-honesty).
2. **Blocked partitioned on the ready frontier** (soft-satisfied node is ready, not
   double-listed). **Accepted, and commended.** A real bug found and fixed before it
   shipped; verified it is the *correct* read of 0013 §4.4 (the soft signal travels
   with the ready node and renders), which my R-4 under-specified. **Carried forward:**
   slice03's `orient` inherits this — the soft-sat ⚠ surfaces on the ready frontier.
3. **Orphans appear in Provenance but not the tree.** **Accepted for slice02** — the
   tree is the resolved `Recomposition` forest *by design* (R-2), and an orphan is a
   `check` Error (slice01). Orphans are **not invisible** (the provenance view covers
   every node, incl. orphans). **But** the MVP DoD is "full situational awareness from
   `orient` alone," so an Error-level structural break must be unmissable. **Open item
   for slice03:** `orient` should surface `check` integrity findings (orphans,
   cycles-without-tears) inline alongside the rollup — the rollup is the structural
   view; `orient` is the actionable read. Not a slice02 defect; a slice03 design input.
4. **Two genuinely unreachable lines uncovered.** **Accepted** — named explicitly;
   coverage clears the floor with room.

## Verdict

**Arc 03 / Slice 02 CDC-verified on structure; all flags accepted (two improve on the
spec); cargo rows pending CI.** `odm rollup` generates the whole-plan view from a pure,
reusable model — the single source slice03 (`orient`) and slice04 (`--json`) build on.
On CI green (`attested → reproduced`), the slice closes.

**Threads carried forward:** (a) slice03 `orient` surfaces the soft-sat ⚠ from the
ready frontier (ruling 2) **and** `check` integrity findings inline so orphans are
unmissable (ruling 3); (b) slice04 pins both the rollup `--json` and the `check --json`
v2 envelope as canonical schemas (slice01 forward note).

CDC: planning thread, 2026-06-25. Iterations used: 1.
