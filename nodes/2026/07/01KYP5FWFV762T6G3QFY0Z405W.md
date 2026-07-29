---
id: 01KYP5FWFV762T6G3QFY0Z405W
number: 589833300
type: artifact
schema: artifact/v1.1
name: Slice 06 closing report — Live-run capability
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice06-live-run-capability/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6S6MH11W8QX4XWGQWB
---
# Slice 06 closing report — Live-run capability

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 06 · **Feeds:** MF-2, MF-3, MF-5
> **Realizes:** ODD-0025 §2.1/§2.3/§2.8 (the hard gate, the synthesis exclusion, update-in-place) ·
> **Resolves:** arc-plan v2.1 (CDC's two-path finding), v2.2 (the missing entry point) ·
> **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (F-1…F-10) · **Implemented by:** CC
> **Date:** 2026-07-28 · **Branch:** `release/1.0.x` (the arc's fast-forward pattern — no per-slice branch)
> **Evidence class:** attested-by-CC (local 1.85+); cargo rows reproduce on CI/CDC.

## What shipped

Two pieces, both fixture-verified, no live mutation:

1. **The two-path `source`-backfill fix (CDC v2.1).** A new `reconcile_source()` in `selfhost.rs` is
   now the **one** gated, project-excluding `source`-population policy. `self_host`'s
   coordinate→source transition (`to_populate`) — which previously stamped `source` **ungated** and
   **without the project exclusion** — now routes through it, exactly like `repair()` already did.
   **Self-identified extension beyond the literal finding:** while unifying the two paths, the same
   review surfaced that neither path guarded against a **retired** node (a tombstone) sharing a
   coordinate with a live plan directory — `repair()`'s loop happened to filter retired nodes as a
   side effect of its own pre-check, but `self_host`'s `by_coordinate` map did not. `reconcile_source`
   now excludes retired nodes too (a historical record, never live work — `replan.rs`'s established
   principle, extended here), and the guard now lives in exactly one place for both callers.
2. **`repair()` wired into `odm migrate`'s self-host path (CDC v2.2).** `repair()`'s only callers
   were tests — the reconcile-before-import flow had no way to actually run. `self_host_inner`
   (`odm-cli/src/migrate.rs`) now calls `selfhost::repair` **first** (reconcile existing: repair
   stubs + gated faithful-backfill), **then** `selfhost::self_host` (import missing arcs/slices), both
   `Mode`/`--dry-run`-aware, with the repair count folded into the rendered report and the status
   line. **No new command or flag** — the command inventory documents no `--repair`/`--reconcile` opt-in
   for `migrate`, and both steps are idempotent + gated + `--dry-run`-safe, so this landed default-on
   (flagged explicitly, per the working agreement, rather than silently decided).

Entirely fixture-verified (`TempDir` stores + synthetic fixtures throughout). The live `.worktrees/odm`
corpus is untouched — see F-8.

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | Two-path fix | **done** | `reconcile_source()` is the one policy; gated + project-excluded + retired-excluded (the last one a self-identified extension). |
| F-2 | `odm migrate` runs the full flow | **done** | `self_host_inner` calls `repair()` then `self_host()`; default-on, no new flag (decision flagged below). |
| F-3 | Correct order | **done** | Implemented as specified; see the Deviations section for a disclosed nuance about what "load-bearing" actually means here. |
| F-4 | End-to-end fixture flow | **done** | One fixture, one `odm migrate` call, every outcome (stub/faithful/project/missing-arc) asserted. |
| F-5 | Project excluded by every path | **done** | Proven via the transition path specifically, and re-confirmed through the full CLI flow. |
| F-6 | `--dry-run` mutates nothing | **done** | Byte-identical store snapshot before/after; the gate still runs (drift would still be caught). |
| F-7 | Re-run idempotent | **done** | Second run: 0 reconciled, 0 created, identical node/id set. |
| F-8 | No live-store mutation | **done** | `.worktrees/odm` isn't even checked out in this working tree. |
| F-9 | Clippy/unsafe/coverage | **done** | Clean; 0 `unsafe`; `selfhost.rs` 96.26%; `migrate.rs`'s new code has zero missing lines (the file's low aggregate is entirely pre-existing, untouched code). |
| F-10 | No decided-model drift | **done** | Cross-read confirms no stored hash, project/synthesis exclusion, update-in-place preserved. |

**Rows: 10. Done: 10. Deferred: 0. No-op: 0.** No silent drops — the slice-doc's "Out" items (the live
run, `artifact` minting + check-wiring, the project node's synthesis re-cast, the arc-close reconcile
demo) were not touched; verified by `.worktrees/odm` not existing in this working tree at all, and by
grep (no `NodeType::Artifact` reference anywhere, no `context.json` write path added, `replan.rs`
outside this slice's diff).

## Verification

| Check | Result |
|-------|--------|
| `cargo test --workspace --all-features` | all green, no failures (every crate's suite, incl. 3 new `odm-cli` tests and 5 new `odm-migrate` tests) |
| `cargo test -p odm-migrate` | 51 (lib, incl. 4 new `reconcile_source` unit tests) + existing integration suites + 10 (`source_identity`, incl. 1 new), all green |
| `cargo test -p odm-cli` | existing suites + 3 (`migrate_reconcile`, new), all green |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean (only the pre-existing, unrelated `proc-macro-error2` future-incompat notice) |
| `cargo fmt --check --all` | clean |
| `unsafe` in changed files | none |
| `cargo llvm-cov --workspace --summary-only` | `odm-migrate/src/selfhost.rs` 96.26% line; `odm-cli/src/migrate.rs` new code fully covered (verified via `--show-missing-lines`, none of the added/changed lines appear) |
| Live store (`.worktrees/odm`) | not present in this working tree (`git -C .worktrees/odm status` → no such directory) — cannot have been touched |

## Deviations (flagged, per the working agreement)

### Finding: given the "Preferred" unification, the repair→self_host order is not load-bearing for per-node correctness in this implementation — only for report shape

The cc-prompt's Task 2/F-3 frame the call order as load-bearing: "reconcile existing... **before**
importing missing arcs... so the import's `by_source` key finds the already-reconciled nodes." Having
implemented the **Preferred** unification (Task 1) — routing `self_host`'s own coordinate→source
transition through the *same* `reconcile_source` gate `repair` uses, rather than removing that
transition's backfill and making `repair` the *sole* place `source` is ever populated — I traced
through both orderings and found they converge on an **identical final store state** for every
coordinate-matchable node:

- **`repair()` first:** reconciles the node, persists it with `source`. `self_host()` then scans the
  store, finds it via `by_source`, reports `SkipReason::AlreadyExists`.
- **`self_host()` alone** (no `repair()` call): finds the same node via `by_coordinate` (no `source`
  yet), calls the *same* `reconcile_source`, persists the *same* resulting document, reports
  `SkipReason::SourcePopulated`.

Both paths use the identical `discover(plan_root)` call and the identical existing-corpus scan, so
there is no node either mechanism can reconcile that the other cannot. The order changes **which**
function performs the write and what the report attributes it to — not the resulting node.

This is implemented **exactly as specified** regardless (`repair()` first, `self_host()` second) — it
is safe, matches the plan, and is the more explicit/composable shape (repair() now has a real,
independent caller, which was the actual, stated problem in the v2.2 finding: "repair()'s only callers
are tests"). The finding is disclosed here rather than either (a) silently implementing something
different because the stated rationale didn't hold up, or (b) writing a test comment asserting a
correctness-load-bearing claim I had specifically falsified by tracing the code. The order **would**
become genuinely load-bearing for correctness under the "Or remove the transition's own backfill
entirely" alternative the cc-prompt also offered — that alternative was not chosen (see the F-1
disposition), so this note names the actual shape of the dependency rather than the one the plan
initially assumed.

### Note: the retired-node guard is a self-identified addition, not requested by the literal CDC finding

CDC's v2.1 finding named the project-node gap specifically. While implementing the unification, the
same review pattern (does the unified function honor every precedent invariant this codebase already
established?) surfaced that `self_host`'s `by_coordinate` map carried no guard against a **retired**
node at all — `repair()`'s old loop excluded retired nodes as a side effect of a pre-check that never
existed on the transition side. Given the live corpus documents a real tombstone node
(`design-notes.md` §1), this is not a hypothetical: fixed alongside F-1 as the same class of gap, not
scoped separately, and disclosed here rather than folded in silently.

### Note: `context.json` re-pointing is out of scope here, despite one mention in `slice-doc.md`'s prose

`slice-doc.md`'s Goal paragraph lists "`v1.1` + `source`/`author`/`version` + `context.json`" among
what "done" means, but the operative `cc-prompt.md` Task list (the four numbered tasks) never mentions
`context.json`, and the arc-plan's own slice-breakdown table assigns "re-point `context.json`"
explicitly to **s07** (the live run). Structurally `context.json` is written only by `odm use`/`store
init` — an operator statement of current focus, not a derived artifact of migration — so re-pointing
it doesn't fit "fixture-only, no live mutation" at all; it only makes sense against a real store.
Followed `cc-prompt.md` (the assignment I was given) over `slice-doc.md`'s summary prose on this one
point; no `context.json` code was added or touched. Flagged here as a minor spec inconsistency between
the two documents rather than silently picking one without saying so.

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

**Did s06 make the flow invocable and one-policy?** Yes, at the fixture-verified level the arc-plan
(v2.2/v2.3) scoped this slice to. `source` now has exactly one population policy
(`reconcile_source`), reachable from exactly two call sites (`repair`, `self_host`'s transition), both
of which agree byte-for-byte on outcome. `odm migrate` on a self-hosted plan tree now runs the full
reconcile-then-import flow with no new command surface, resolving the v2.2 "repair() has no caller"
gap at its root. Both v2.1 and v2.2 findings are closed.

**What implementing it revealed the arc-plan didn't anticipate:**

1. **The "Preferred" unification choice (route the transition through the shared gate, rather than
   removing it) makes the two call sites functionally redundant for per-node correctness, not just
   textually unified.** This is a *good* property — it means `self_host()` remains fully correct when
   called standalone (as many existing tests, and potentially future callers, do) without requiring
   `repair()` to run first — but it does mean the plan's "order is load-bearing" framing describes the
   report/attribution shape more precisely than it describes a correctness dependency. Worth naming
   for s07: the live run's safety does **not** hinge on `repair()` running before `self_host()` in the
   wired CLI path specifically — it hinges on `reconcile_source` being correct, which is now exercised
   both ways.
2. **A "two paths converged on one bug" pattern is worth a standing review step.** The retired-node gap
   (F-1's self-identified extension) was found by asking "what does the *other*, already-correct path
   guard against that this one doesn't?" — the same question that would have caught the original
   project-node gap, had it been asked proactively before CDC found it in review. Recommend this
   question become a standing step whenever a slice's job is explicitly "unify two implementations of
   the same operation."
3. **`repair()`'s CLI wiring needed no new report type** — folding `RepairReport` into the existing
   render pipeline (`render_repair` alongside `render_self_host`) and a three-count status line
   (`reconciled`/`created`/`skipped`) was a same-shape extension of the existing `SelfHostReport`
   rendering, not a new rendering concept. Worth noting for s07/s08: the report-shape risk the slice-doc
   worried about ("a report-shape or ordering wrinkle") did not materialize — the existing
   `Themed`/`OxurTable` pattern absorbed a second table with no friction.

**The slice-scale silent-drop diff:** scope-as-specified (`slice-doc.md`'s "In"/"Out") vs.
scope-as-delivered — no drops, one disclosed addition (the retired-node guard, beyond the literal CDC
finding but within the same "unify the source-backfill policy" scope). All four "Out" items (the live
run, `artifact` minting + check-wiring, the project node's synthesis re-cast, the arc-close reconcile
demo) are confirmed absent — verified by `.worktrees/odm` not existing in this working tree, and by
grep for their would-be-visible traces.

**Recommended arc-ledger update (MF-2, MF-3, MF-5):** all three remain **planned**. The flow that makes
a live run safe, correct, and invocable now exists, is CDC-review-informed (both v2.1 and v2.2
findings closed), and is fixture-proven end-to-end — the live-corpus outcome each MF row actually
asserts (zero stubs, every non-project node source-bearing, all arcs represented) is still **s07**'s
job. Recorded as pointers from MF-2/MF-3/MF-5 to this closing report, alongside s04's and s05's, as
baseline evidence, per LEDGER-DISCIPLINE v2.0 §B.
