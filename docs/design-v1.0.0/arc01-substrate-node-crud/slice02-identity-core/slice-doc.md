# Slice 02 — Stable identity core (plan-of-record)

> Per-slice implementation plan. Refs: ODD-0013 §2.1–§2.2, ODD-0015 §3 (A1.2).
> `depends_on:` slice01 (the workspace must exist + be CI-green first).

## Goal

Implement `odm-core`'s identity primitives — the value types every later slice
builds on: `Id` (ULID), `NodeType`, `Origin`, and a **minimal** `Node` skeleton —
as pure, well-tested types. **Done when the types + their invariant proptests are
green, clippy is clean, and `odm-core` coverage clears the bar.**

No persistence, no serde, no frontmatter parsing (slice 03), no edges/status/gates
(Arc 02), no number→id resolution (needs a collection — slice 04/05).

## Decisions

- **`Id` wraps a ULID** (the `ulid` crate). Opaque, immutable, **never reused**:
  `Id::new()` always mints a fresh ULID; we never recycle one (the legacy
  number-reuse model is gone). ULID is lexicographically time-ordered, so id
  ordering = creation ordering — but order is never *derived from* the id in the
  planning sense (that's the dependency graph's job).
- **`number: u32` is a separate human handle**, not identity, and **not
  interconvertible** with `Id` (no `From<u32> for Id`). Human-number *allocation*
  (next unused number) is deferred to slice 04, where the node set lives.
- **`NodeType` = `Project | Arc | Slice | Odd | Adr | Note`** — **no `Step`**
  (ODD-0013 §2.2). Work nodes: project/arc/slice; document nodes: odd/adr/note.
- **`Origin` = `Planned | Discovered | Amendment`** (the `provenance` rename;
  `reserved: bool` is the separate future-placeholder flag).
- **serde derives deferred to slice 03** — the *serialization format* (how a ULID,
  enum, or date appears in frontmatter) is a schema decision owned by slice 03.
  Slice 02 stays pure types + logic.

## Scope

**In:** `Id` (new/parse/Display/Ord, ULID-backed); `NodeType` (variants,
parse/as_str, `is_work`/`is_document`, `valid_child_types` for the
project→arc→slice containment rule used later by `check`); `Origin`
(variants, parse/Display); a minimal `Node { id, number, node_type, name, origin,
reserved }`; invariant proptests; full rustdoc on public items.

**Out:** serde/frontmatter (slice 03), persistence/store (slice 04), CRUD/CLI
(slice 05), edges/status/gates/graph (Arc 02), number→id resolution & number
allocation (slice 04/05), dates `created`/`updated` (frontmatter — slice 03).

## Steps (→ ledger rows)

1. `Id` newtype over `ulid::Ulid`: `new`, `FromStr`, `Display`, `Ord`/`Eq`/`Hash`.
2. `NodeType` + `Origin` enums: variants, parse/as_str round-trip, classification
   + `valid_child_types`.
3. `Node` minimal skeleton + constructor; identity-stability behavior (rename/number
   change must not touch `id`).
4. Invariant proptests (uniqueness, round-trip, ordering, identity stability).
5. rustdoc on every public item; `#![deny(missing_docs)]` in `odm-core`.

## Verification

`cargo test -p odm-core` green (incl. proptests); `cargo clippy -p odm-core --
-D warnings` clean; `cargo llvm-cov -p odm-core` ≥ 90% (target 95%; justify any
gap); no `unsafe`; no `unwrap`/`expect` outside tests. Full grep/test-verifiable
rows in `ledger.md`.

## Exit

`ledger.md` closed (every row `done`/`deferred`/`no-op` with evidence), CDC
verified (compile/test rows via CI or a local 1.85+ run). Then slice 03
(frontmatter schema + round-trip) opens.
