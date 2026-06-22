# Closing Report — Slice 03 (Arc 01): Frontmatter schema + round-trip

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain) before slice 04 opens.

- **Implementation commit:** `06662af`.
- **Branch:** `slice03-frontmatter-schema` (not pushed; not merged to `main`).
- **Scope delivered:** `odm-core::frontmatter` — the `---` YAML + markdown body
  format, the typed schema (§2.3) incl. the edges block (§3), round-trip-stable
  parse/emit, unknown-key preservation, and the YAML-library decision.
- **Result:** 10 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.
- **Aggregate gates:** `cargo test -p odm-core` → 23 tests + 3 doctests pass;
  `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0; region
  coverage 98.79% (line 99.61%).

## Per-row walk

| ID | Status | Evidence (re-runnable at `06662af`) |
|----|--------|-------------------------------------|
| I-1 | done | `cargo test -p odm-core frontmatter_parse` → 2 passed. Typed `FrontmatterError::{MissingOpen,Unterminated,Yaml}`. |
| I-2 | done | `cargo test -p odm-core schema_core_fields` → 1 passed. All 10 core fields asserted. |
| I-3 | done | `cargo test -p odm-core schema_edges_block` → 1 passed. All 8 edge kinds + bare/qualified deps. |
| I-4 | done | `cargo test -p odm-core frontmatter_roundtrip` → 1 passed (128-case proptest, `parse(emit(d)) == d`). |
| I-5 | done | `cargo test -p odm-core unknown_keys_preserved` → 2 passed (fixture + proptest); keys survive + remain in emitted text. |
| I-6 | done | `cargo test -p odm-core canonical_field_order` → 1 passed (snapshot pins §2.3 order). |
| I-7 | done | `cargo test -p odm-core supersedes_kind` → 1 passed (both `obsoletes`/`updates`). |
| I-8 | done | `! grep -RInE 'serde_yaml(_ng)?\|serde_norway' crates/*/src \| grep -v '/frontmatter'` → no match. Maintenance: see decision (2). |
| I-9 | done | `! grep -RnE '\bunsafe\b' crates/odm-core/src` → no match; errors are `thiserror`; `emit` returns `Result`. |
| I-10 | done | clippy → exit 0; `cargo llvm-cov -p odm-core --summary-only` → region 98.79% / line 99.61% / fn 100%. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **Canonical edge order resolved against a doc inconsistency.** ODD-0013 §2.3
   (the example) orders edges `part_of, depends_on, blocked_by, verifies,
   consumes, supersedes, tears` and **omits `affects`**; §3 (the edge table)
   orders them `…, consumes, verifies, affects, supersedes, tears` — i.e. §2.3
   and §3 disagree on `verifies` vs `consumes`, and only §3 lists `affects`.
   I-6 cites §2.3, so I followed §2.3's order (`verifies` before `consumes`) and
   inserted `affects` immediately before `supersedes` (its slot in §3). The
   snapshot test pins this. **Recommend** a one-line doc fix to reconcile §2.3
   and §3; flagging rather than silently picking.

2. **YAML library: serde_yaml_ng is stale → used the named fallback
   serde_norway.** Maintenance check (2026-06-22): serde_yaml_ng's latest is
   0.10.0, published **2024-05-26** (~25 mo ago) — past the slice-doc's ~12-month
   bar, so the decision rule triggers the fallback. serde_norway's latest is
   0.9.42, **2024-12-21** (~18 mo), and it shows a far more active 2024 release
   cadence (7 releases) vs serde_yaml_ng's single May-2024 cluster. **Caveat:**
   *both* exceed a strict 12-month-from-today reading — the entire serde_yaml
   fork ecosystem has been quiet since late 2024. I selected serde_norway as the
   freshest, most-maintained option and the slice-doc's designated fallback. If
   CDC wants a different backend (or to pin a specific commit), that's an
   amendment — the module isolation makes the swap a one-file change.

3. **`Frontmatter` is a standalone schema type, not a wrapper around slice-02's
   `Node`.** The §2.3 field order interleaves Node's fields (created/updated/
   tags/component sit between `name` and `origin`), so flattening `Node` would
   not produce canonical order. `Frontmatter` therefore reuses the `Id`/
   `NodeType`/`Origin` types directly. Bridging frontmatter ↔ `Node` (e.g. a
   `Node::from_parts` for loading persisted nodes) is deferred to slice 04 (the
   store), consistent with slice 02's flagged mint-only `Node::new`.

4. **`created`/`updated` are `chrono::NaiveDate`.** §2.3 shows date-only values
   (`2026-06-20`). `NaiveDate` round-trips as ISO `YYYY-MM-DD`. This exposes
   `chrono` in the public API (accessor return + constructor arg) — `chrono` is
   already a workspace dependency and is not a YAML type, so I-8 is unaffected.

5. **`serde` impls hand-written for `Id`/`NodeType`/`Origin`** (as canonical
   strings) rather than derived, so the on-disk form is exactly the ULID/lower-
   case-name string and no derive ties the wire format to internal structure.

## Uncertainties named

- **Round-trip is model-level over non-adversarial field values.** The proptest
  generates field text from `[a-zA-Z0-9 ._-]` and bodies from the same — i.e.
  "arbitrary valid nodes," not pathological YAML. A `name`/body containing a bare
  `---` line or exotic YAML control characters is out of the generated space;
  parsing takes the *first* closing `---` so a body with later `---` lines is
  fine, but a frontmatter string value that serializes to a bare `---` line is
  not exercised (serde_norway quotes/indents such values, so it should not arise
  — but it is unproven for all inputs).
- **YAML-lib maintenance caveat** (decision 2): serde_norway's last release is
  ~18 months old. It works and is isolated, but "actively maintained" is
  generous; revisit if a YAML CVE or a serde-version bump forces the issue.
- **Coverage gap is the error edges.** The ~1.2% uncovered region is in serde
  `custom`-error branches (e.g. an `Id`/`NodeType` string that fails mid-deser);
  the typed-error paths that matter (`MissingOpen`/`Unterminated`/`Yaml`) are
  covered. Above the 95% target regardless.
- **Sandbox has no Rust toolchain**, so all cargo evidence was produced on the
  local dev host; CDC should reproduce on CI / a local 1.85+ run.
