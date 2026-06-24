# Slice 01 (Arc 02): Graph construction + reverse edges

> Per LEDGER_DISCIPLINE. Final status + evidence before advancing; cargo rows
> reproduced in CI / local 1.85+ (sandbox has none). Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | Graph builds from a node set; every node maps to exactly one graph index | `cargo test -p odm-core graph_build` → ok | serious | 0013 §4 | done | `f2f74f0`: `graph_build` → 2 passed (`..._maps_every_node` + `..._skips_dangling_edges`). 3 nodes → `node_count==3`, each `contains`; idempotent `add_node` ⇒ one index per id. | |
| H-2 | Reverse edges derived, not stored: backlinks match forward edges | `cargo test -p odm-graph reverse_edges` (proptest: reverse adjacency == transpose of forward) → ok | serious | 0013 §3 | done | `f2f74f0`: `reverse_edges_is_transpose` → 1 passed (200-case proptest: `b ∈ successors(a,k)` iff `a ∈ predecessors(b,k)`). Reverse uses petgraph `Incoming`; nothing stored. | |
| H-3 | Ordering DAG = `depends_on ∪ consumes` only (excludes part_of/verifies/supersedes/affects) | `cargo test -p odm-core ordering_dag_membership` → ok | serious | 0013 §3 | done | `f2f74f0`: `ordering_dag_membership` → 1 passed. A node with one edge of every kind: `ordering_successors` = {depends_on, consumes} targets only; part_of/blocked_by/verifies/affects/supersedes/tears excluded (but reachable via their own accessors). | |
| H-4 | `part_of` exposed as a separate single-parent tree (not in the ordering DAG) | `cargo test -p odm-core part_of_tree` → ok | correctness | 0013 §3 | done | `f2f74f0`: `part_of_tree` → 1 passed. `parent()` single (root → None); `children()` = derived reverse `part_of` (total recomposition); `part_of` absent from `ordering_successors`. | |
| H-5 | Forward/reverse accessors by edge kind (children/parents) | `cargo test -p odm-graph adjacency_by_kind` → ok | correctness | 0013 §4 | done | `f2f74f0`: `adjacency_by_kind` → 1 passed. `successors`/`predecessors`/`outgoing` filter correctly by kind. | |
| H-6 | `odm-graph` is domain-agnostic: no node-type/gate names in its source | `! grep -REiq 'project\|arc\|slice\|odd\|adr\|gate' crates/odm-graph/src` | correctness | 0013 §8 | done | `f2f74f0`: grep → no match. Engine is generic `Graph<N, E>`; `EdgeKind` lives in odm-core. Vocabulary kept neutral (no `Arc` type, "search", "hierarchy", "aggregate") — see decision (1). | |
| H-7 | Clippy clean (`-D warnings`); no `unsafe` | `cargo clippy -p odm-graph -p odm-core --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-graph/src` | serious | CLAUDE.md | done | `f2f74f0`: clippy (both) → exit 0; unsafe grep → no match. | |
| H-8 | Coverage ≥ 90% (target 95%) | `cargo llvm-cov -p odm-graph -p odm-core --summary-only --ignore-filename-regex '(odm-store|odm-cli)/'` → **line** ≥ 90% (target 95%) | correctness | CLAUDE.md | done | `f2f74f0`: TOTAL **line 97.67%**, region 97.48% (odm-graph/lib.rs 90.16% line, odm-core/graph.rs 94.03% line). | Run from a clean `target/llvm-cov-target` (stale-state caveat carried from arc01). |

## What Worked

- **Generic engine = trivially domain-agnostic.** Making `odm-graph` a generic
  `Graph<N, E>` (not a graph of odm ids and odm edge kinds) meant H-6 passed by
  construction: there is *no* place for a domain name because the engine never
  names a node type or edge kind — `EdgeKind` is defined in odm-core and only
  ever appears as the type parameter `E`. The H-6 grep is satisfied as a side
  effect of the right layering, not by scrubbing.
- **Reverse = the same edges read backward.** petgraph's directed graph answers
  both `Outgoing` and `Incoming` from one stored edge set, so "derived reverse"
  is just a `Direction` argument — H-2's transpose property holds by
  construction, and there is genuinely one place to edit (the source's forward
  edge).
- **Views as filters, not separate graphs.** The ordering DAG and the `part_of`
  tree are the *same* graph queried with different edge-kind filters
  (`{DependsOn, Consumes}` vs `{PartOf}`), so they cannot drift out of sync and
  H-3/H-4's "separate but consistent" falls out for free.
- **Build skips dangling edges** rather than erroring: link-integrity is
  arc01's `check`, so the graph stays buildable on an imperfect corpus and the
  two concerns don't entangle.

## Closure

Closed at `f2f74f0` on 2026-06-24. CDC: _pending_ (cargo rows via CI / local
1.85+; sandbox has no toolchain). Total rows: 8. Done: 8. Deferred: 0. No-op: 0.
