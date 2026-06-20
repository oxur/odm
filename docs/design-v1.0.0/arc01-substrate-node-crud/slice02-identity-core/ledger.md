# Slice 02: Stable identity core

> Per LEDGER_DISCIPLINE. Every row reaches `done`/`deferred`/`no-op` with evidence
> (commit SHA + Verify output) before the slice advances. Compile/test rows are
> reproduced by CDC in CI or a local 1.85+ toolchain (the Cowork sandbox has none).
> Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| G-1 | `Id` is a ULID newtype; `Id::new()` mints unique ids | `cargo test -p odm-core id_uniqueness` (proptest: 10k ids all distinct) → ok | serious | 0013 §2.1 | open | | |
| G-2 | `Id` round-trips through its string form | `cargo test -p odm-core id_roundtrip` (proptest: `parse(display(id)) == id`) → ok | correctness | 0013 §2.1 | open | | |
| G-3 | `Id` ordering equals creation order (ULID lexicographic == temporal) | `cargo test -p odm-core id_creation_ordered` → ok | correctness | 0013 §2.1 | open | | |
| G-4 | Identity ≠ human number: no `From<u32>`/numeric ctor for `Id`; `Node.number` and `Node.id` are distinct types | `! grep -REq 'impl[^;]*From<u32>[^;]*for Id' crates/odm-core/src` AND `cargo test -p odm-core identity_not_number` → ok | serious | 0013 §2.1, 0001 A1 | open | | |
| G-5 | `NodeType` = exactly {Project,Arc,Slice,Odd,Adr,Note}; no `Step` | `cargo test -p odm-core nodetype_variants` (asserts the 6 + parse round-trip) AND `! grep -qiw 'Step' crates/odm-core/src/*.rs` | serious | 0013 §2.2 | open | | |
| G-6 | `NodeType` classifies work vs document | `cargo test -p odm-core nodetype_classification` (project/arc/slice → work; odd/adr/note → document) → ok | correctness | 0013 §2.2 | open | | |
| G-7 | Containment rule data present: project→arc→slice via `valid_child_types` | `cargo test -p odm-core valid_child_types` (Project⊇{Arc}; Arc⊇{Slice}; Slice has no work children) → ok | correctness | 0011 R1 / 0013 §2.2 | open | | |
| G-8 | `Origin` = {Planned,Discovered,Amendment}; parse/Display round-trip | `cargo test -p odm-core origin_roundtrip` → ok | correctness | 0013 §2.3 | open | | |
| G-9 | `Node` skeleton holds id/number/node_type/name/origin/reserved; identity stable under rename & number change | `cargo test -p odm-core node_identity_stable` (mutate name + number; `id` unchanged) → ok | serious | 0013 §2.1 | open | | |
| G-10 | Every public item documented; `#![deny(missing_docs)]` in odm-core | `grep -q '#!\[deny(missing_docs)\]' crates/odm-core/src/lib.rs` AND `cargo doc -p odm-core --no-deps` → exit 0, no warnings | correctness | rust-guidelines (docs) | open | | |
| G-11 | Clippy clean (`-D warnings`) | `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0 | serious | CLAUDE.md | open | | |
| G-12 | No `unsafe`; no `unwrap`/`expect` outside tests/docs | `! grep -RnE '\bunsafe\b' crates/odm-core/src` AND `! grep -RnE '\.(unwrap\|expect)\(' crates/odm-core/src/*.rs` (test modules excepted) | serious | rust-guidelines (anti-patterns) | open | | |
| G-13 | `odm-core` coverage ≥ 90% (target 95%) | `cargo llvm-cov -p odm-core --summary-only` → region/line ≥ 90% | correctness | CLAUDE.md (95% target) | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at commit `<SHA>` on `<date>`. CDC verification: `<name/session>` (compile/
test rows via CI or local 1.85+). Total rows: 13. Done: _. Deferred: _. No-op: _.
