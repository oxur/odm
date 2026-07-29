---
id: 01KYP5FW0GRCDNW5A4KHK1461V
number: 548682500
type: artifact
schema: artifact/v1.1
name: Slice 05 closing report — Source-based identity
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice05-source-identity/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6S0CAYR4Q2WJ2X4XAS
---
# Slice 05 closing report — Source-based identity

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 05 · **Feeds:** MF-2, MF-3, MF-5
> **Realizes:** ODD-0025 §2.0/§2.2 (`source` as the identity axis), §2.8 (repair extended, not
> replaced), §5 (coverage — exact set-difference on `source`) · **Assignment:** `cc-prompt.md`
> **Ledger:** `ledger.md` (F-1…F-12) · **Implemented by:** CC · **Date:** 2026-07-28
> **Branch:** `release/1.0.x` (per the arc's established fast-forward pattern — no per-slice branch)
> **Evidence class:** attested-by-CC (local 1.85+); cargo rows reproduce on CI/CDC.

## What shipped

Retiring `number` as a **correctness key** — fixture-proven, no live mutation:

1. **`self_host` idempotence keys on `source.paths`, not `(type, number)`.** Existing work nodes
   are indexed by `by_source` (primary — every path in every node's `source.paths`) and, only for a
   node that carries no `source` yet, `by_coordinate` (the one-time transition fallback). A re-run
   finds an already-imported node by its stable source path regardless of what number this run's
   `discover()` computes for it.
2. **The coordinate→source transition backfills `source` in place.** A matched pre-`source` node is
   never re-created; its frontmatter is cloned, `source`/`updated`/schema are set on the delta, and
   it is persisted via `Store::persist` overwrite — the same "clone + mutate the delta" pattern s04's
   `repair` established, reused here for a second call site.
3. **The s04 v1.8 named-arc re-run-duplicate hazard is now structurally impossible**, not just less
   likely: source-matching alone would have prevented the duplicate even under a number shift, and
   the name-derived handle (below) additionally removes the shift itself.
4. **`named_arc_number` is name-derived, not position-based.** Replaces
   `named_arc_number(index)` with `named_arc_number(slug, taken)` — an FNV-1a hash of the arc's own
   directory slug into the `≥ 1900` band, collision-handled against the handles already assigned
   earlier in the same `discover()` pass. Recomputable from the slug alone; unchanged when another
   named arc is added or removed.
5. **`repair` is generalized past the stub filter (extended, not forked).** A stub's body is still
   replaced outright (unchanged s04 behavior); a **faithful non-stub** node lacking `source` keeps its
   existing body, which is verified against the freshly-read source body through the same hard
   body-hash gate — a match adds `source` as a pure content no-op (F-4); a mismatch surfaces
   `MigrateError::BodyHashMismatch` and the node is left untouched, not silently backfilled over
   (F-5). The **project node stays excluded** — its body is a synthesis (`replan.rs::vision_from_plan`),
   not a 1:1 migration (ODD-0025 §2.3), so it can never pass the gate (F-6).
6. **`coverage.rs`'s `doc_coverage` gains an exact `source.paths` match as the primary check**,
   ahead of the structural/number heuristics. This directly closes the named-arc doc-coverage gap
   CC's s04 bubble-up disclosed (a source-bearing named-arc node now resolves without needing any
   named-arc-aware structural matching — the model already carried the answer). The heuristics remain
   as the pre-`source` fallback for legacy nodes.

Entirely fixture-verified (`test-data/` + `TempDir` stores throughout). The live `.worktrees/odm`
corpus is untouched — see F-10.

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | Source-keyed idempotence | **done** | `by_source` checked before `by_coordinate`; a hand-numbered, source-bearing node is matched and not duplicated. |
| F-2 | Coordinate→source transition | **done** | First run: `SkipReason::SourcePopulated`, source backfilled. Second run: `SkipReason::AlreadyExists`, matched by source. |
| F-3 | Re-run duplicate impossible | **done** | Adding a named arc that sorts earlier and re-running mints no duplicate; the pre-existing arc's id survives. |
| F-4 | Backfill faithful non-stub | **done** | Body byte-identical, `source` added, idempotent on re-run. |
| F-5 | Backfill surfaces drift | **done** | A drifted non-stub body returns `BodyHashMismatch`; node left with no `source`. |
| F-6 | Project node excluded | **done** | `repaired_count() == 0` on a project-only fixture; not an error. |
| F-7 | Source-based coverage matching | **done** | A named arc's `arc-plan.md` resolves with basis `"source.paths (exact match)"`. |
| F-8 | Name-derived stable handle | **done** | Recomputable from the slug alone (unit test); unchanged across two independent stores, one with a second named arc present (integration test). |
| F-9 | `number` keys nothing for correctness | **done** | Grep: `by_source` is the sole primary key in both `self_host` and `doc_coverage`; every remaining `(type, number)` use is either the one-time transition fallback or per-run parent-wiring. |
| F-10 | No live-store mutation | **done** | `.worktrees/odm` is not even checked out in this session; every test opens a `Store` over `TempDir`. |
| F-11 | Clippy/unsafe/coverage | **done** | Clean; 0 `unsafe`; all changed modules ≥ 93.9% line coverage (`selfhost.rs` clears the 95% target). |
| F-12 | No decided-model drift | **done** | Cross-read confirms `source` is the identity key, `repair` extended not forked, project/synthesis excluded, no stored hash. |

**Rows: 12. Done: 12. Deferred: 0. No-op: 0.** No silent drops — the slice-doc's "Out" items (the
live run, `artifact` minting + check-wiring, the project node's synthesis treatment, re-pointing the
live `context.json`) were not touched; verified below and by grep (no `context.json` write in
`selfhost.rs`, no `NodeType::Artifact` reference anywhere, `replan.rs` untouched by this slice's diff).

## Verification

| Check | Result |
|-------|--------|
| `cargo test --workspace --all-features` | all green, no failures (every crate's suite, incl. the 9 new `tests/source_identity.rs` cases) |
| `cargo test -p odm-migrate` | 47 (lib) + 6 (`coverage`) + 9 (`fidelity`) + 10 (`migrate`) + 3 (`real_docs`) + 5 (`selfhost`) + 9 (`source_identity`), all green |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean (only an unrelated future-incompat notice for `proc-macro-error2`, a transitive dep) |
| `cargo fmt --check --all` | clean |
| `unsafe` in changed files | none (`grep -RnE '\bunsafe\b'` on every file this slice touched) |
| `cargo llvm-cov -p odm-migrate --summary-only` | `selfhost.rs` 95.27%, `coverage.rs` 93.98%, `lib.rs` 94.53%, `fidelity.rs` 100% (unchanged) — all ≥ 90%; the shortfall against the 95% target on `coverage.rs`/`lib.rs` is entirely pre-existing/unrelated lines, confirmed via `--show-missing-lines` against this slice's diff |
| Live store (`.worktrees/odm`) | not present in this working tree (`git -C .worktrees/odm status` → no such directory) — cannot have been touched |
| `grep -n "node_type(), fm.number())\|node.node_type, node.number\|(NodeType, u32)" selfhost.rs coverage.rs` | every hit is the one-time transition fallback or per-run wiring, never a cross-run identity key |

## Deviations (flagged, per the working agreement)

None. The slice landed exactly as scoped in `slice-doc.md`/`cc-prompt.md`: source-keyed idempotence
+ the coordinate transition (self_host), the generalized backfill (repair), source-based coverage
matching, and the name-derived handle — all four pieces, fixture-verified, in one pass (the
five-iteration cap's split-escape for coverage-matching was not needed; it landed with the rest).

### Note: the `repair()` drift-detection path is exercised end-to-end, not just at the primitive

Unlike s03/s04's note about `verify_body_hash`'s fail-case being proven only at the primitive
(because those callers always constructed `node_body` from the same read as `source_body`), this
slice's F-5 test constructs a **genuinely independent** pre-existing body and a **genuinely
independent** source-file body, so `repair()`'s drift path is exercised as a real integration case,
not an invariant-check. Worth naming as the pattern for verifying a hard-gate's fail branch generally:
prefer two independently-authored inputs over one input re-derived from the other.

### Note: `body_source_path`'s catch-all branch remains provably unreachable (carried from s04)

Same accepted shape as s04's closing report recorded — `_ => node.source.clone()` for node types
`discover()` never produces. Untouched this slice; still a safety net, not dead code worth removing
without a `#[non_exhaustive]`-safe justification this slice doesn't need to make.

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

**Did s05 retire `number`-as-key, and does it resolve the v1.8 pre-mint requirement?** Yes, at the
fixture-verified level the arc-plan (v1.9) scoped this slice to. `number` — numbered or named — now
keys nothing for correctness anywhere in `odm-migrate`: idempotence, the coordinate transition, the
backfill match, and coverage all key on `source.paths`, with `(type, number)` surviving only as the
documented one-time pre-`source` fallback and per-run containment wiring. The v1.8 finding (position-
based named-arc handles causing a re-run duplicate) is closed at its root, not patched: even without
the name-derived handle, source-matching alone would prevent the duplicate; the name-derived handle
additionally removes the number *shift* itself. F12 (design-notes) is **done**.

**What implementing it revealed the arc-plan didn't anticipate:**

1. **The stub-vs-non-stub branch in `repair()` collapses into one code path, not two.** The original
   plan implicitly suggested "extend repair to also handle non-stub nodes" might need a second
   branch with its own error handling. In practice, computing `new_body` up front (source text for a
   stub, the existing body otherwise) and always gating `new_body` against `source_body` through the
   *same* `verify_body_hash` call means the stub-repair and non-stub-backfill behaviors are one
   function with one gate, not two policies bolted together. Worth naming as a general shape for
   "generalize an operation past a narrower filter": look for the invariant that makes both cases the
   same check, rather than branching the check itself.
2. **Source-path coverage matching required no changes to the *model*, only to the *matcher*.**
   Every node `self_host`/`migrate` create already carried `source.paths` since s03 — the named-arc
   coverage gap was purely a matcher limitation (`arc_coordinate` cannot derive a name-derived
   handle from a directory name alone). This means the "split-escape to s07" the slice-doc reserved
   in case s05 ran heavy was never needed: F-7 turned out to be a small, low-risk addition once F-1
   was in place, not independent heavy work. Worth noting for future slice-sizing: a "coverage
   matching" line item that depends on a field another in-scope task already populates is often
   cheaper than it looks in isolation.
3. **A concrete edge case worth a name for s06+: number churn under a genuine hash collision.**
   `named_arc_number`'s collision-bump is deterministic *given a fixed set of slugs processed in a
   fixed order*, but adding a slug that collides with an *already-assigned* slot could, in the
   vanishingly unlikely case, still shift a later-processed arc's number if insertion order changes
   which slug claims the natural slot first. This is not a live risk (FNV-1a over a million slots for
   a handful of real arc names), and source-matching would absorb it even if it happened — but it is
   the one place "stable under adding arcs" is a strong practical guarantee rather than a
   mathematical one. Not worth engineering around further; worth a one-line mention if s06's live run
   ever needs to explain why two runs disagreed on a named arc's cosmetic number.

**The slice-scale silent-drop diff:** scope-as-specified (`slice-doc.md`'s "In"/"Out") vs.
scope-as-delivered — no drops. All four "Out" items (the live run, `artifact` minting +
check-wiring, the project node's synthesis treatment, re-pointing the live `context.json`) are
confirmed absent — verified by `.worktrees/odm` not existing in this working tree at all, and by
grep for their would-be-visible traces (no `context.json` write path in `selfhost.rs`, no
`NodeType::Artifact` reference, `replan.rs` outside this slice's diff).

**Recommended arc-ledger update (MF-2, MF-3, MF-5):** all three remain **planned**. The identity
model that makes a live re-run safe and re-runnable now exists and is fixture-proven in full — the
live-corpus outcome each MF row actually asserts (zero stubs, every node source-bearing, all arcs
represented) is still s06's job. Recorded as pointers from MF-2/MF-3/MF-5 to this closing report,
alongside s04's, as baseline evidence, per LEDGER-DISCIPLINE v2.0 §B (composition rows reproduce at
arc scale, at arc close — never inherited from a slice's fixture-only attestation).
