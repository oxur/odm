# Closing Report — Slice 01 (Arc 02): Graph construction + reverse edges

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain) before slice 02 opens.

- **Implementation commit:** `f2f74f0`.
- **Branch:** `arc02-slice01-graph` (based on arc01-complete `main`; not pushed;
  not merged to `main`).
- **Scope delivered:** the in-memory graph — `odm-graph` (generic engine,
  forward + derived-reverse adjacency by kind) and `odm-core::graph` (translation
  + the ordering-DAG / `part_of`-tree split). No cycles, gates, or
  `next`/`blocked` (later slices).
- **Result:** 8 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.
- **Aggregate gates:** `cargo test -p odm-graph` → 7 pass; `cargo test -p
  odm-core` → all pass (incl. 6 graph tests); clippy (both) `-D warnings` → exit
  0; no `unsafe`; H-6 grep clean; coverage TOTAL line 97.67% / region 97.48%.
  Workspace clippy/fmt clean.

## Per-row walk

| ID | Status | Evidence (re-runnable at `f2f74f0`) |
|----|--------|-------------------------------------|
| H-1 | done | `cargo test -p odm-core graph_build` → 2 passed; 3 nodes → 3 indices; idempotent add. |
| H-2 | done | `cargo test -p odm-graph reverse_edges` → 1 passed (200-case transpose proptest). |
| H-3 | done | `cargo test -p odm-core ordering_dag_membership` → 1 passed; ordering = {depends_on, consumes} only. |
| H-4 | done | `cargo test -p odm-core part_of_tree` → 1 passed; single parent, derived-reverse children, not in ordering DAG. |
| H-5 | done | `cargo test -p odm-graph adjacency_by_kind` → 1 passed; forward/reverse/outgoing filtered by kind. |
| H-6 | done | `! grep -REiq 'project\|arc\|slice\|odd\|adr\|gate' crates/odm-graph/src` → no match. |
| H-7 | done | clippy (odm-graph+odm-core) exit 0; `! grep '\bunsafe\b' crates/odm-graph/src` → no match. |
| H-8 | done | `cargo llvm-cov … --ignore-filename-regex '(odm-store\|odm-cli)/'` → line 97.67% / region 97.48%. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **`odm-graph` is generic (`Graph<N, E>`), and `EdgeKind` lives in odm-core.**
   The cc-prompt says "over abstract `(NodeId, EdgeKind)`". I read that as: the
   engine is parameterized over the id and edge-kind *types*, with the concrete
   odm `EdgeKind` (PartOf/DependsOn/…) defined in odm-core. This makes H-6
   structural rather than cosmetic — there is no domain vocabulary in the engine
   to grep for. (Note the H-6 grep is an aggressive case-insensitive *substring*
   match — `arc` matches "search"/"hierarchy", `gate` matches "aggregate" — so I
   also kept the engine's prose/identifiers clear of those incidental hits, e.g.
   no `std::sync::Arc`, "traverse" not "search", "tree" not "hierarchy". Flagging
   the substring sharpness in case CDC wants the grep scoped to whole words.)

2. **Dangling edges are skipped at build, not errored.** An edge whose target is
   not in the node set cannot be a graph edge, so `build` drops it (returning
   `false` from `add_edge`). Link-integrity is arc01's `check` (it flags exactly
   these), so the graph stays buildable on an imperfect corpus and the two
   concerns stay separate. Flagging because `build` is therefore total (no
   `Result`) — a deliberate choice given `check` owns ref-resolution.

3. **`part_of` single-parent is enforced by the schema, surfaced by `parent()`.**
   The frontmatter `part_of` field is already `Option<Id>` (one parent max), so
   `parent()` returns `Option<Id>` directly. The engine itself would permit
   multiple `PartOf` edges out of a node; the single-parent guarantee comes from
   the source schema, and `parent()` takes the first if any. A `check`-level
   assertion that no node has >1 part_of is not needed (the schema makes it
   unrepresentable) — noted so CDC doesn't expect a graph-level guard.

## Uncertainties named

- **No cycle/acyclicity guarantee yet.** This slice builds the graph and the
  views; it does **not** detect or reject cycles (that is slice 02, with tears).
  `ordering_successors`/`children` are plain adjacency, not transitive closure,
  so a cyclic corpus builds fine here and is caught later. Intentional per scope.
- **`EdgeKind` is `Copy` and not yet `serde`.** It is an in-memory translation
  artifact (the on-disk form is the slice03 frontmatter edges), so it carries no
  serde derives. If a later slice needs to persist a derived graph view, that is
  a separate decision.
- **Coverage clean-state caveat (carried from arc01).** `cargo llvm-cov` must run
  from a clean `target/llvm-cov-target`; the 97.67% figure is from a fresh
  rebuild.
- **Sandbox has no Rust toolchain**, so all cargo evidence was produced on the
  local dev host; CDC should reproduce on CI / a local 1.85+ run.
