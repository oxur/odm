# Slice 03 (Arc 01): Frontmatter schema + round-trip

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+ (sandbox has
> none). Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| I-1 | Parse splits `---`-delimited YAML frontmatter from the markdown body; malformed/missing frontmatter → typed error | `cargo test -p odm-core frontmatter_parse` → ok | serious | 0013 §2.3 | open | | |
| I-2 | Core fields parse: id, number, type, name, created, updated, origin, reserved, tags, component | `cargo test -p odm-core schema_core_fields` → ok | serious | 0013 §2.3 | open | | |
| I-3 | Edges block parses: part_of, depends_on (+ optional `satisfied_at`), blocked_by, consumes, verifies, supersedes `{node,kind}`, affects, tears | `cargo test -p odm-core schema_edges_block` → ok | serious | 0013 §3 | open | | |
| I-4 | **Round-trip: `parse ∘ emit == identity`** (proptest over arbitrary valid nodes) | `cargo test -p odm-core frontmatter_roundtrip` → ok | serious | 0013 §2.3 | open | | |
| I-5 | Unknown keys preserved across round-trip (forward-compat for status/desired_facts) | `cargo test -p odm-core unknown_keys_preserved` → ok | serious | 0013 §2.3 | open | | |
| I-6 | Emission uses the canonical field order documented in 0013 §2.3 | `cargo test -p odm-core canonical_field_order` (snapshot) → ok | correctness | 0013 §2.3 | open | | |
| I-7 | `supersedes` carries `kind` ∈ {obsoletes, updates} | `cargo test -p odm-core supersedes_kind` → ok | correctness | 0013 §3 | open | | |
| I-8 | YAML lib isolated behind the `frontmatter` module (no YAML-crate type elsewhere) + maintained (release ≤ ~12 mo) | `! grep -RInE 'serde_yaml(_ng)?\|serde_norway' crates/*/src \| grep -v '/frontmatter'` (no leak) AND maintenance noted in evidence | correctness | 0014 / slice-doc | open | | |
| I-9 | No `unsafe`; no panics on public paths; typed errors | `! grep -RnE '\bunsafe\b' crates/odm-core/src` AND parse errors are `thiserror` types | serious | rust-guidelines | open | | |
| I-10 | Clippy clean (`-D warnings`); coverage ≥ 90% (target 95%) | `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0 AND `cargo llvm-cov -p odm-core --summary-only` ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at `<SHA>` on `<date>`. CDC: `<name>` (cargo rows via CI/local 1.85+).
Total rows: 10. Done: _. Deferred: _. No-op: _.
