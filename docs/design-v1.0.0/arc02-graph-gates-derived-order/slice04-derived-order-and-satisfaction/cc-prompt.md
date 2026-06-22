# CC Prompt — Slice 04 (Arc 02): Derived order & satisfaction

You are implementing **Arc 02 / Slice 04**: derived-order queries and
**evidence-leveled satisfaction**. This is where `next`/`blocked`/`path` come from,
and where a dependency satisfied only on low-confidence evidence gets *surfaced*
instead of silently treated as green.

> **Start condition:** arc02 slices 01 (typed edges), 02 (DAG + tears), and 03
> (multi-gate status + evidence levels) must be CDC-closed first. If not, hold.

## Read first (in this order)

1. `docs/design-v1.0.0/arc02-graph-gates-derived-order/slice04-derived-order-and-satisfaction/ledger.md` — the 14 acceptance rows. Read before coding.
2. `slice-doc.md` (same dir) — the plan + the design recap.
3. `docs/design/01-draft/0013-odm-architecture-design.md` §4.1 and **§4.4** — the
   satisfaction + evidence-leveled-satisfaction spec (the authority).
4. `docs/design/01-draft/0017-...md` §3.3 — the cross-team counterpart (same idea,
   across a team boundary); and ODD-0001 F2/G3 — the failure this guards against.

## Load these skills

- **rust-guidelines** — `11-anti-patterns.md`, then `06-traits.md`,
  `05-type-design.md`, `02-api-design.md`. Graph algorithms over `odm-graph`'s
  abstract ids; keep the engine domain-agnostic (translation lives in `odm-core`).
- **collaboration-framework → LEDGER_DISCIPLINE** — work against the ledger;
  evidence per commit; per-row closing report; name uncertainty.

## Task

Per `slice-doc.md` / ODD-0013 §4.4:

- Topo order (Kahn) over `depends_on ∪ consumes`; `next` (ready frontier: deps
  satisfied, no active `blocked_by`, not complete); `blocked X` (reasons);
  `path X [Y]` (chain / critical path).
- **Satisfaction** predicate: edge satisfied iff target reached `satisfied_at`
  (default terminal) gate.
- **Evidence-leveled satisfaction** — the core of this slice:
  - Evidence ordering `asserted < attested < reproduced < reconciled` (a total
    order; recorded per gate transition in slice03).
  - **Min-propagation**: a node's effective evidence = the minimum across its
    transitive dependency path.
  - A configurable **threshold** (default `reproduced`, from `odm.toml`); below it,
    a satisfied dependency is **soft-satisfied**.
  - Soft-satisfied surfacing: `next` lists the node but flags it
    (`⚠ dep X satisfied at evidence=attested`); `blocked X` names the low-evidence
    dep and how to raise it. **It must NOT block** — visibility, not gating.
- Staleness guard: advancing a node with an unsatisfied `depends_on` warns.
- `--json` for each query, carrying the evidence level.

## Constraints (honor exactly; flag, don't silently change)

- Threshold default is `reproduced`; configurable, not hard-coded.
- Soft-satisfied is *non-blocking* (H-10) — surfacing only. Do not gate `next` on it.
- Keep `odm-graph` domain-agnostic; node-type/gate semantics translate in `odm-core`.
- No `unsafe`; typed errors; no panics on public paths; proptest the min-propagation
  and threshold behaviors. Coverage ≥ 90% (target 95%).

## Deliverables

- Green: `cargo test -p odm-graph -p odm-core`, `cargo clippy … -D warnings`,
  `cargo llvm-cov` ≥ 90%; `--json` snapshot tests.
- `ledger.md` with evidence per row; `closing-report.md` (per-row walk for all 14,
  "What Worked", uncertainties named).
- Feature branch (`arc02-slice04-derived-order-and-satisfaction`); not on `main`.

## Working agreement

- Ledger row wrong/impossible? Raise an amendment — don't work around it.
- Five-iteration cap. Your `done` is *proposed done*; CDC re-runs every Verify
  (compile/test rows in CI or a local 1.85+ toolchain) before slice05 opens.
