# Closing Report — Slice 02 (Arc 02): Cycle detection + tears

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain) before slice 03 opens.

- **Implementation commit:** `b7514cf`.
- **Branch:** `arc02-slice02-cycles` (based on `arc02-slice01-graph`; not pushed;
  not merged to `main`).
- **Scope delivered:** Kahn-based cycle detection over the ordering relation,
  precise member naming, the explicit `Tear` mechanism (rationale required) that
  breaks a cycle, the typed `Cycle` error for an un-torn cycle, and enumerable
  active tears. All in `odm-graph`, domain-agnostic.
- **Result:** 8 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.
- **Aggregate gates:** `cargo test -p odm-graph` → 16 pass (10 cycle/tear + 6
  prior); clippy `-D warnings` → exit 0; no `unsafe`; H-6 domain-agnostic grep
  clean; coverage TOTAL line 93.58% / region 93.91%. Workspace builds.

## Per-row walk

| ID | Status | Evidence (re-runnable at `b7514cf`) |
|----|--------|-------------------------------------|
| H-1 | done | `cargo test -p odm-graph detect_cycle` → 2 passed; members named, downstream excluded. |
| H-2 | done | `cargo test -p odm-graph acyclic_no_cycle` → 1 passed; chain → `None`. |
| H-3 | done | `cargo test -p odm-graph tear_breaks_cycle` → 1 passed; torn edge → acyclic. |
| H-4 | done | `cargo test -p odm-graph tear_requires_rationale` → 1 passed; empty/whitespace → `Err`. |
| H-5 | done | `cargo test -p odm-graph cycle_without_tear_errors` → 1 passed; `Cycle` is a typed `Error` naming members. |
| H-6 | done | `cargo test -p odm-graph list_active_tears` → 1 passed; tear of a real edge listed, phantom excluded. |
| H-7 | done | clippy exit 0; `! grep '\bunsafe\b' crates/odm-graph/src` → no match. |
| H-8 | done | `cargo llvm-cov -p odm-graph … --ignore-filename-regex '(odm-core\|odm-store\|odm-cli)/'` → line 93.58%. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **Kahn detects; a DFS back-edge walk names the members.** §4.2 says "Kahn's
   algorithm yields cycle detection for free." Kahn's *leftover* set, however,
   includes nodes merely *downstream* of a cycle (a node whose in-degree never
   reaches zero because a cycle blocks it), so it over-names. I therefore use
   Kahn for the yes/no verdict and a DFS back-edge walk over the un-emitted nodes
   to name the **precise** cycle. The `detect_cycle_excludes_innocent_downstream`
   test pins that distinction. Flagging because "names its members" is
   implemented as a second pass, not directly from Kahn's output.

2. **`detect_cycle` returns `Option<Cycle>`, and `Cycle` *is* the typed error.**
   The probe returns `None` (acyclic) or `Some(Cycle)`; `Cycle<N>` implements
   `std::error::Error` (with `N: Debug + Display`), so slice06's `check` can
   either branch on the `Option` or propagate the `Cycle` as a hard failure.
   This satisfies both H-1 ("names members") and H-5 ("hard error") with one
   type. Flagging the shape in case CDC expected a `Result`-returning API.

3. **Tears are passed in per call, not stored on the graph.** `detect_cycle` and
   `active_tears` take `tears: &[Tear<N>]`. The graph stays a pure structure;
   tears (which live in node frontmatter in the domain) are supplied by the
   caller (odm-core, a later wiring slice). This keeps odm-graph from needing a
   mutable "torn" state and matches "the caller owns the ordering relation."

4. **"Ordering kinds" are a caller parameter.** odm-graph does not know which
   edge kinds are "ordering" (that is `depends_on ∪ consumes` in odm-core). The
   caller passes `ordering_kinds: &[E]`. This is what keeps the engine
   domain-agnostic (H-6) and lets the same primitive power `next`/`blocked` in
   slice04 with the same kind set.

## Uncertainties named

- **Coverage residue (~9% of `cycle.rs`).** The uncovered lines are the
  `extract_cycle` empty-`Vec` fallback (logically unreachable once Kahn has
  proven a cycle exists) and the `emitted.insert` duplicate-queue guard (a
  defensive belt-and-braces check). Both are guards that cannot fire on a
  well-formed run; I judged contriving inputs to hit them not worth it. TOTAL
  line 93.58% clears the bar; flagging the specific defensive lines.
- **Cycle attribution when multiple disjoint cycles exist.** `detect_cycle`
  returns *one* concrete cycle (the first the DFS reaches, in node-index order),
  not all of them. That is deterministic and sufficient to fail `check`; a
  "report every cycle" variant is a possible later refinement. Noted.
- **No multigraph dedup.** If the same `(from, to)` ordering edge is added twice,
  both contribute to in-degree; a single tear of that pair removes both (the
  torn set is keyed by the pair, not by edge identity). Fine for the domain
  (frontmatter lists a target at most once per kind in practice); noted.
- **Sandbox has no Rust toolchain**, so all cargo evidence was produced on the
  local dev host; CDC should reproduce on CI / a local 1.85+ run, from a clean
  `target/llvm-cov-target`.
