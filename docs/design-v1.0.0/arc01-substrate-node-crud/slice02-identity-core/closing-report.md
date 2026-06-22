# Closing Report — Slice 02: Stable identity core

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. Every row gets a
> disposition + reproducible evidence. CC's `done` is **proposed done**; CDC
> re-runs every Verify (compile/test rows in CI or a local 1.85+ toolchain)
> before slice 03 opens.

- **Implementation commit:** `403b146`.
- **Branch:** `slice02-identity-core` (not pushed; not merged to `main`).
- **Scope delivered:** `odm-core` identity primitives — `Id`, `NodeType`,
  `Origin`, minimal `Node` — pure value types, no serde/persistence/edges/CLI.
- **Result:** 13 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.
- **Aggregate gates:** `cargo test -p odm-core` → 13 tests + 2 doctests pass;
  `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0;
  `RUSTDOCFLAGS="-D warnings" cargo doc -p odm-core --no-deps` → exit 0;
  `cargo llvm-cov -p odm-core` → 100% region/line/function.

## Per-row walk

| ID | Status | Evidence (re-runnable at `403b146`) |
|----|--------|-------------------------------------|
| G-1 | done | `cargo test -p odm-core id_uniqueness` → ok (1). Proptest mints 1k–10k ids/case into a `HashSet`; `len == n`. |
| G-2 | done | `cargo test -p odm-core id_roundtrip` → ok (2). `parse(display(id)) == id`; plus garbage→typed errors. |
| G-3 | done | `cargo test -p odm-core id_creation_ordered` → ok (1). 5 ids, 2ms apart, strictly increasing. |
| G-4 | done | `! grep -REq 'impl[^;]*From<u32>[^;]*for Id' crates/odm-core/src` → no match; `cargo test -p odm-core identity_not_number` → ok (1). |
| G-5 | done | `cargo test -p odm-core nodetype_variants` → ok (1); `! grep -qiw 'Step' crates/odm-core/src/*.rs` → no match. |
| G-6 | done | `cargo test -p odm-core nodetype_classification` → ok (1). |
| G-7 | done | `cargo test -p odm-core valid_child_types` → ok (1). Project→[Arc], Arc→[Slice], Slice/docs→[]. |
| G-8 | done | `cargo test -p odm-core origin_roundtrip` → ok (1). |
| G-9 | done | `cargo test -p odm-core node_identity_stable` → ok (2; unit + proptest). |
| G-10 | done | `grep -q '#!\[deny(missing_docs)\]' crates/odm-core/src/lib.rs` → match; `cargo doc -p odm-core --no-deps` (RUSTDOCFLAGS=-D warnings) → exit 0. |
| G-11 | done | `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0. |
| G-12 | done | `! grep -RnE '\bunsafe\b' crates/odm-core/src` → no match; `! grep -RnE '\.(unwrap\|expect)\(' crates/odm-core/src/*.rs` → no match. |
| G-13 | done | `cargo llvm-cov -p odm-core --summary-only` → region/line/function 100.00%. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **`IdParseError` is a self-contained enum, not a `#[from] ulid::DecodeError`
   wrapper.** The slice forbids leaking impl deps (AP-44) and wants typed parse
   errors (AP-54/60). I map ulid's two `DecodeError` variants
   (`InvalidLength`/`InvalidChar`) into our own enum, so the public API never
   names the ulid type. Cost: if ulid adds a decode-failure mode in a future
   version, the match in `Id::from_str` would fail to compile — a compile-time
   prompt to extend our enum, which I consider a feature, not a risk.

2. **`Id` implements `Default` (delegating to `new`).** A no-arg `pub fn new()`
   trips `clippy::new_without_default`, which `-D warnings` (G-11) makes fatal.
   Per ID-10, `new()`-with-no-args must agree with `Default`, so `Default` mints
   a fresh id too. A non-deterministic `Default` is mildly unusual; flagging in
   case CDC would rather drop `Default` and `#[expect]` the lint. (I think
   `Default = new` is the conventional, guideline-endorsed choice.)

3. **`Node::new` mints the id internally; there is no `with_id`/`from_parts`.**
   Reconstructing a `Node` from an existing id (e.g., loading a file) needs such
   a constructor, but that is slice 04's concern (the store). Keeping `new`
   mint-only here is minimal and makes the uniqueness/identity tests trivial.
   Flagging so CDC doesn't read the absent constructor as an oversight.

4. **Doc wording changed to satisfy literal greps.** G-5's `-iw 'Step'` matched
   doc sentences that explained the no-step rule; I reworded them ("no node
   smaller than a slice") to keep the meaning without the bare token. The
   *behavioral* check that `NodeType::from_str("step")` errors lives in
   `tests/identity.rs` (not `src/`), so the rule is still tested.

5. **`valid_child_types` returns work children only.** Project→[Arc],
   Arc→[Slice], and Slice/Odd/Adr/Note→[]. Document containment via `part_of`
   is deliberately deferred; the row (G-7) only specifies the work tree.

## Uncertainties named

- **G-3 relies on a 2ms sleep, not a same-millisecond guarantee.** ULID puts the
  timestamp in the high 48 bits, so ids minted ≥1ms apart always compare in
  creation order regardless of the random low bits; the 2ms gap makes the test
  deterministic. The only way it could fail is a wall-clock regression (e.g.,
  NTP step backwards) of >2ms mid-test, which is negligible. `Id::new()` is
  **not** monotonic *within* a millisecond — and the code/docs do not claim it
  is (the doc says ordering within the same ms is unspecified).
- **Coverage is 100% on a 1.85+ host; the Cowork sandbox has no Rust toolchain.**
  All compile/test/coverage evidence here was produced locally on the dev
  machine. CDC should reproduce on CI or a local 1.85+ run, per the ledger note.
- **`id_uniqueness` is probabilistic.** It asserts 1k–10k freshly minted ULIDs
  are distinct. Collisions are astronomically unlikely (80 bits of randomness
  per millisecond), but it is a statistical, not a structural, guarantee — which
  is the nature of the ULID uniqueness claim itself.
