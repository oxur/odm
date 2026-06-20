# CC Prompt — Slice 02: Stable identity core

You are implementing **Slice 02** of the `odm` v1.0.0 rebuild: the identity
primitives in `odm-core`. Pure value types + invariant tests — **no persistence,
no serde, no frontmatter, no edges/status, no CLI.** Done when the types and their
proptests are green, clippy is clean, and coverage clears the bar.

> **Start condition:** slice 01 must be CI-green first (the workspace must build).
> If it isn't yet, hold.

## Read first (in this order)

1. `docs/design-v1.0.0/arc01-substrate-node-crud/slice02-identity-core/ledger.md`
   — the acceptance criteria. Read before writing code.
2. `slice-doc.md` (same dir) — the plan, scope, and decisions.
3. `docs/design/01-draft/0013-odm-architecture-design.md` §2.1–§2.2 — identity,
   node types, the no-`step` rule, `origin`/`reserved`.

## Load these skills

- **rust-guidelines** — `11-anti-patterns.md` first, then `02-api-design.md`,
  `05-type-design.md`, `03-error-handling.md`, `13-documentation.md`. This is
  public-API + type-design work: newtypes for invariants, derive the standard
  traits, `thiserror` for parse errors, `# Errors` docs, no panics on public paths.
- **collaboration-framework → LEDGER_DISCIPLINE** — work against the ledger; fill
  Evidence per commit; per-row closing report; name uncertainty.

## Task

Implement in `odm-core` (per `slice-doc.md`):

- **`Id`** — newtype over `ulid::Ulid`. `new()` (fresh ULID), `FromStr`,
  `Display`, `Ord`/`PartialOrd`/`Eq`/`Hash`, `Debug`. **No `From<u32>` or any
  numeric constructor** (identity must not be confusable with the human number).
- **`NodeType`** — `Project | Arc | Slice | Odd | Adr | Note` (**no `Step`**).
  `as_str`/`FromStr`, `is_work()`/`is_document()`, and `valid_child_types()`
  encoding project→arc→slice containment (used later by `check`).
- **`Origin`** — `Planned | Discovered | Amendment`, parse/Display.
- **`Node`** (minimal) — `{ id: Id, number: u32, node_type: NodeType, name:
  String, origin: Origin, reserved: bool }` + a constructor. Dates, edges, status,
  tags arrive in later slices; do not add them now.
- **Invariant proptests** — id uniqueness, string round-trip, creation-ordering,
  enum round-trips, and identity-stability (mutating `name`/`number` leaves `id`
  unchanged).
- **rustdoc** on every public item; `#![deny(missing_docs)]` in `odm-core`.

## Constraints (honor exactly; flag, don't silently change)

- ULID via the `ulid` crate (already in `[workspace.dependencies]`).
- **No serde** in this slice — serialization is slice 03's decision. Do not add
  `#[derive(Serialize/Deserialize)]` or pin a frontmatter lib.
- **No `Step`** anywhere. No persistence, store, CLI, edges, gates, or resolution.
- No `unsafe`; no `unwrap`/`expect` outside tests/doctests; parse errors are typed
  (`thiserror`).
- `odm-core` coverage ≥ 90% (target 95%); justify any uncovered line.

## Deliverables

- The types + tests green: `cargo test -p odm-core`, `cargo clippy -p odm-core --
  -D warnings`, `cargo doc -p odm-core --no-deps` all clean; `cargo llvm-cov -p
  odm-core` ≥ 90%.
- `ledger.md` updated with Evidence (commit SHA + Verify output) per row.
- `closing-report.md` in this dir: a per-row walk for all 13 rows, a "What Worked"
  note, uncertainties named.
- Work on a feature branch (`slice02-identity-core`); do not push to `main`.

## Working agreement

- Ledger row wrong/impossible? Raise an amendment — don't work around it.
- Five-iteration cap.
- Your `done` is *proposed done*; CDC re-runs every Verify (compile/test rows in
  CI or a local 1.85+ toolchain) before slice 03 opens.
