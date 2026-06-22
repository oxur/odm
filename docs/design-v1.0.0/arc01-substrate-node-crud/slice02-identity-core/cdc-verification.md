# CDC Verification — Slice 02: Stable identity core

> Independent verification of CC's closed ledger (impl `403b146`; closed `dfbe586`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran the
> Verify; **attested** = CC's evidence, not reproduced here.

## Environment constraint (disclosed)

Same as slice 01: the Cowork sandbox has **no Rust toolchain** (apt offers only
cargo 1.75, below the 1.85 edition-2024 floor), so the `cargo`-gated rows cannot be
reproduced here. They are routed to CI / a local 1.85+ run.

## Row dispositions

**Row count:** 13 opened, 13 addressed. No silent drops. ✔

**Reproduced by CDC (structural greps, re-run in-session):**
G-4 (no `From<u32> for Id` — grep clean; `Id`/`number` distinct types), G-5 (no
`Step` in `src` — grep clean), G-10 (`#![deny(missing_docs)]` present), G-12 (no
`unsafe`, no `unwrap`/`expect` in `src` — both greps clean). → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
G-1, G-2, G-3 (proptests), G-6, G-7, G-8, G-9, G-11 (clippy), G-13 (coverage
100%). Evidence is consistent and candid. → **PENDING CI.**

## Rulings on CC's four flagged deviations

1. **`IdParseError` self-contained, not `#[from] ulid::DecodeError`.** **Accepted
   — and preferred.** Not leaking a third-party error type through our public API
   is good API design (rust-guidelines); the trade-off (hiding the dep) is the
   right one for a public surface.
2. **`Id: Default` (delegates to `new`) to satisfy `clippy::new_without_default`.**
   **Accepted.** Minting a fresh ULID as the "default" is consistent with `new`;
   harmless under `-D warnings`.
3. **`Node::new` mints the id; no `with_id` reconstruction ctor yet.** **Accepted.**
   Reconstruction-from-id is a *store* concern (loading persisted nodes) — correctly
   deferred to A1.3/04.
4. **Reworded docs to avoid the bare word "step" for G-5's `-iw 'Step'` grep.**
   **Accepted, with a process note:** the grep is an *over-literal proxy* — it
   forbids the word in prose, not just a `Step` node type. Semantics are intact, so
   no rework. Future ledgers should scope such greps to code (e.g. `enum`/`::Step`
   patterns) rather than the bare word. (Logged as the recurring "grep matched
   prose" lesson, now seen in slice01 F-6 and here.)

## Verdict

Slice 02 is **CDC-verified on structure; all four deviations accepted; 9
cargo-rows attested, pending CI.** Same closure path as slice 01: CI (or local
1.85+) reproduces the test/clippy/coverage rows.

**Note (dogfooding §4.4 live):** slice 03 `depends_on` slice 02, whose terminal
gate is currently `attested`, not `reproduced`. By the evidence-leveled-satisfaction
rule we just adopted, that dependency is **soft-satisfied** — slice 03 may proceed,
but it is consciously built on attested (not reproduced) evidence until CI flips
slices 01–02. The discipline makes that visible rather than silent.

CDC: planning thread, 2026-06-22. Iterations used: 1.

## Closure update (2026-06-22)

**CI is green.** The previously-attested cargo rows (G-1/2/3/6/7/8/9/11/13 —
proptests, classification, clippy, 100% coverage) are now **reproduced** by an
independent CI run. **Slice 02 is fully CDC-closed**, and slice 03 opens with its
dependency on slice 02 now satisfied at `reproduced` (no longer soft-satisfied).
(CI-green taken on operator confirmation; CDC did not read the CI log directly.)
