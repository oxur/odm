# Slice 04 (Arc 02) — Derived order & satisfaction (plan-of-record)

> Per-slice plan. Refs: ODD-0013 §4.1 & §4.4 (satisfaction + **evidence-leveled
> satisfaction**, v1.5), ODD-0015 (A2), ODD-0017 §3.3 (the cross-team counterpart),
> ODD-0001 F2/G3 (the failure this guards). `depends_on:` arc02 slice01 (edges),
> slice02 (DAG), slice03 (gates + evidence levels).

## Goal

Derive order and **confidence** from the graph: `next`/`blocked`/`path`/topo,
edge satisfaction, the staleness guard, and — the headline of this slice —
**evidence-leveled satisfaction**: a dependency satisfied only at low evidence is
*soft-satisfied*, surfaced rather than silently green, so we never build
load-bearing work on a relayed belief (the prod-DB 503).

## Design recap (from ODD-0013 §4.4)

- Evidence ordering: `asserted < attested < reproduced < reconciled` (0001-D3),
  recorded per gate transition in slice03.
- An edge `A depends_on B` is **satisfied** when `B` reached the edge's
  `satisfied_at` gate (default: `B`'s type's terminal gate).
- **Min-propagation:** a node's *effective* evidence is the minimum over its
  transitive dependency path — a chain is only as verified as its weakest link.
- **Threshold** (default `reproduced`, configurable in `odm.toml`): satisfaction
  below it is **soft-satisfied**.
- Soft-satisfied ⇒ `next` lists but flags it; `blocked X` explains it and how to
  raise it; `check` warns (strict/CI: fails). Never *blocks* — only makes low
  confidence visible.

## Scope

**In:** **wire the typed `Status` (slice03) into `Frontmatter`** (replace the
preserved-unknown-key passthrough with the typed field — satisfaction must *read*
status; this flips arc01's `unknown_keys_preserved` 2→1, an expected/disclosed
change, status moving unknown→known); topo order (Kahn) over `depends_on ∪
consumes`; `next` (ready frontier, no active `blocked_by`, not complete);
`blocked X` (unsatisfied + soft-satisfied reasons); `path X [Y]` (chain / critical
path); satisfaction predicate; evidence min-propagation; the configurable
threshold; soft-satisfied surfacing in `next`/`blocked`; the staleness guard;
`--json` for each query. **Decide + encode** (slice03 uncertainty): whether
reaching gate N requires gates < N, and whether evidence may regress.

**Out:** `check` v2 wiring of the strict-fail (slice06; this slice exposes the
warning + the predicate it uses); the gate-recording mechanics (slice03);
recomposition (slice05); reconciler probes / external nodes (Arc A5 / ODD-0017).

## Verification

`cargo test -p odm-graph -p odm-core` green (incl. proptests for min-propagation
and threshold behavior); clippy `-D warnings`; coverage ≥ 90% (target 95%);
`--json` schema snapshot-tested. Full grep/test rows in `ledger.md`.

## Exit

`ledger.md` closed (evidence per row; compile/test rows via CI or local 1.85+),
CDC verified. Then slice05 (recomposition integrity) opens.
