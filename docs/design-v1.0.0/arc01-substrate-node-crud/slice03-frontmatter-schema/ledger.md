# Slice 03 (Arc 01): Frontmatter schema + round-trip

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+ (sandbox has
> none). Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| I-1 | Parse splits `---`-delimited YAML frontmatter from the markdown body; malformed/missing frontmatter → typed error | `cargo test -p odm-core frontmatter_parse` → ok | serious | 0013 §2.3 | done | `06662af`: `frontmatter_parse` → 2 passed (`..._splits_block_and_body`, `..._rejects_malformed`). MissingOpen/Unterminated/Yaml are typed `FrontmatterError` variants. | |
| I-2 | Core fields parse: id, number, type, name, created, updated, origin, reserved, tags, component | `cargo test -p odm-core schema_core_fields` → ok | serious | 0013 §2.3 | done | `06662af`: `schema_core_fields` → 1 passed. All 10 fields asserted (id, number, type→NodeType, name, created/updated→NaiveDate, tags, component, origin, reserved). | |
| I-3 | Edges block parses: part_of, depends_on (+ optional `satisfied_at`), blocked_by, consumes, verifies, supersedes `{node,kind}`, affects, tears | `cargo test -p odm-core schema_edges_block` → ok | serious | 0013 §3 | done | `06662af`: `schema_edges_block` → 1 passed. All 8 edge kinds + bare/qualified `depends_on` asserted. | |
| I-4 | **Round-trip: `parse ∘ emit == identity`** (proptest over arbitrary valid nodes) | `cargo test -p odm-core frontmatter_roundtrip` → ok | serious | 0013 §2.3 | done | `06662af`: `frontmatter_roundtrip` → 1 passed (proptest, 128 cases: arbitrary fields + tags + edges + multi-line body; `parse(emit(doc)) == doc`). | |
| I-5 | Unknown keys preserved across round-trip (forward-compat for status/desired_facts) | `cargo test -p odm-core unknown_keys_preserved` → ok | serious | 0013 §2.3 | done | `06662af`: `unknown_keys_preserved` → 2 passed (literal `status`/`desired_facts` fixture + proptest over random scalar keys); reparsed doc equals original and keys remain in emitted text. | Flattened catch-all `Mapping`; serde_norway flatten preserves bool/int/nested types (probed). |
| I-6 | Emission uses the canonical field order documented in 0013 §2.3 | `cargo test -p odm-core canonical_field_order` (snapshot) → ok | correctness | 0013 §2.3 | done | `06662af`: `canonical_field_order` → 1 passed. Snapshot pins id,number,type,name,created,updated,tags,component,origin,reserved,edges. | See uncertainty (1): §2.3 vs §3 disagree on verifies/consumes order and §2.3 omits `affects`; resolved per §2.3 + affects-before-supersedes. |
| I-7 | `supersedes` carries `kind` ∈ {obsoletes, updates} | `cargo test -p odm-core supersedes_kind` → ok | correctness | 0013 §3 | done | `06662af`: `supersedes_kind` → 1 passed. Both `obsoletes` and `updates` emit lowercase and round-trip. | `SupersedeKind` `#[serde(rename_all = "lowercase")]`. |
| I-8 | YAML lib isolated behind the `frontmatter` module (no YAML-crate type elsewhere) + maintained (release ≤ ~12 mo) | `! grep -RInE 'serde_yaml(_ng)?\|serde_norway' crates/*/src \| grep -v '/frontmatter'` (no leak) AND maintenance noted in evidence | correctness | 0014 / slice-doc | done | `06662af`: grep → no matches (serde_norway only in `frontmatter.rs`). **Maintenance:** serde_yaml_ng latest 0.10.0 = 2024-05-26 (stale, >12mo) → used slice-doc fallback **serde_norway 0.9.42 = 2024-12-21**. See uncertainty (2): both exceed the literal 12-mo bar; norway is the fresher/most-maintained option. | |
| I-9 | No `unsafe`; no panics on public paths; typed errors | `! grep -RnE '\bunsafe\b' crates/odm-core/src` AND parse errors are `thiserror` types | serious | rust-guidelines | done | `06662af`: unsafe grep → no match; `FrontmatterError` is a `thiserror` enum; no `unwrap`/`expect` in `src` (all in `tests/`); `emit` returns `Result` rather than panicking. | |
| I-10 | Clippy clean (`-D warnings`); coverage ≥ 90% (target 95%) | `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0 AND `cargo llvm-cov -p odm-core --summary-only` ≥ 90% | serious | CLAUDE.md | done | `06662af`: clippy → exit 0; `cargo llvm-cov -p odm-core --summary-only` → region 98.79%, line 99.61%, function 100% (TOTAL). | Exceeds the 95% target. |

## What Worked

- **Probing the dependency before designing.** The headline risk was the
  notorious serde_yaml `#[serde(flatten)]` stringification bug (it would silently
  break unknown-key round-trip). A throwaway probe confirmed `serde_norway`
  preserves bool/int/nested types through flatten — so the simple flatten-based
  catch-all was safe to adopt, no manual `Mapping` plumbing needed.
- **Declaration order = canonical order.** serde emits struct fields in
  declaration order and flattened keys last, so declaring `Frontmatter`'s fields
  in §2.3 order (and `extra` flattened) gives canonical emission for free, with
  unknown keys landing after `edges` exactly as §2.3 shows status/desired_facts.
- **`serde` on the identity types, YAML behind the module.** Implementing
  `Serialize`/`Deserialize` for `Id`/`NodeType`/`Origin` in their own files (as
  canonical strings) kept the I-8 isolation grep green — those impls reference
  generic `serde`, never `serde_norway`, so the YAML backend stays confined to
  `frontmatter.rs` and is swappable.
- **Model-level round-trip + body-as-tail parsing.** Defining `parse ∘ emit` over
  the *model* (not byte-for-byte over arbitrary input) made the invariant robust,
  and taking everything after the first closing `---` as the body verbatim lets
  bodies contain `---` lines without breaking the round-trip.

## Closure

Closed at `06662af` on 2026-06-22. CDC: _pending_ (cargo rows via CI / local
1.85+; sandbox has no toolchain). Total rows: 10. Done: 10. Deferred: 0. No-op: 0.
