# CC Prompt — Slice 05 (Arc 04): Index→graph adapter + wire graph readers & composed views

The slice04 continuation (seams b+c). Make the **graph commands and composed views** read
the index instead of full-scanning. The crux is the **index→graph adapter**: reconstruct
what `NodeGraph::build` / `Satisfaction::compute` consume from index records — no
frontmatter parse — then wire `next`/`blocked`/`path`/`check` + `rollup`/`orient`.

> **Start condition:** slice04 (seam a) CDC-verified / CI-green — the record carries
> per-gate evidence + `EdgeRef` qualifiers, and `reconcile` exists. If not in, hold.

## Read first
1. `slice05-graph-adapter-and-views/ledger.md` (7 rows).
2. `slice-doc.md` (same dir) and `../arc-plan.md` (slice05 + Arc Ledger A-5; the split
   history in v1.7/v1.8).
3. The slice04 `closing-report.md` — CC's adapter sketch (synthesize `Frontmatter`s).
4. Reuse points: `odm-core` `NodeGraph::build(&[Frontmatter])` + `Satisfaction::compute(&[Frontmatter], &GateSets, Evidence)`
   (both take `&[Frontmatter]`; `compute` needs evidence *levels*, which the record now
   carries); the `Rollup`/orient models; slice03 `reconcile`; the `odm-index` record
   (`gates: Vec<GateState>`, `EdgeRef` + qualifiers).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `02-api-design.md`, `05-type-design.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence at `attested`; write the
  **Bubble-up to the arc** section at close).

## Task
1. **Adapter:** reconstruct `Frontmatter`s from index records (id, number, type, name,
   edges from `EdgeRef`+qualifiers, status from `gates`+evidence) → feed the **existing**
   `NodeGraph::build` / `Satisfaction::compute` **unchanged**. (Recommended over adding
   index-native constructors — the smaller seam, zero odm-core change. Pick + record the
   shape on G-1.) The fidelity guard: adapter graph == frontmatter graph.
2. **Wire graph readers** (`reconcile`-then-read): `next`, `blocked`, `path`, `check`
   (incl. evidence-leveled satisfaction: min-prop, soft-sat, threshold).
3. **Wire composed views** (`reconcile`-then-read): `rollup`, `orient` — `orient` loads
   only the current project's `Document` for the vision body (one targeted load).
4. **Identical-to-baseline:** each wired consumer's output equals its `load_all` output
   (assert by test).

## Constraints (flag, don't silently change)
- **Amend, don't work around.** Raise an amendment if a row is wrong/impossible.
- **Reuse, not reimplement:** feed the existing graph/satisfaction; compose over the
  existing `Rollup`/orient models; freshen via `reconcile`. The adapter is the only
  substantial new code — do **not** re-derive graph or satisfaction logic.
- **Bodies stay out of the index** (§3.5): only `orient`'s vision triggers one targeted
  `store.load(project)`.
- No `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables
Green `cargo test -p odm-index -p odm-cli` + clippy + coverage; `ledger.md` evidence per
row (at `attested`); `closing-report.md` — per-row walk **plus the v2.0 Bubble-up to the
arc** (did slice05 deliver seams b+c; what it revealed; the silent-drop diff; note that
**A-4 now closes** and A-11 is satisfiable). Feature branch
(`arc04-slice05-graph-adapter-and-views`); not `main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration cap;
your `done` is *proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local
1.85+). On close, bubble up to `arc-plan.md` (A-5) per LEDGER-DISCIPLINE v2.0 §A.
