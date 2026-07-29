---
id: 01KYP5G49YJYB1GY2EWWTY8DDJ
number: 501198900
type: artifact
schema: artifact/v1.1
name: Slice 01 (arc-store-home) — CDC verification
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-home/slice01-store-resolution/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SYP3MYNPHCTNEA9PM
---
# Slice 01 (arc-store-home) — CDC verification

> **Verifies:** SH-1 · **Slice:** store resolution + two-config split · **Branch:**
> `sh-slice01-store-resolution` (`72987b0`, off `release/1.0.x`) · **Date:** 2026-07-26 ·
> **Verifier:** CDC, independent of CC. Structural rows reproduced by code inspection; cargo/exec
> rows attested-by-CC → reproduced-on-CI.

## Verdict

**Slice 01 delivered; SH-1 `attested`.** Clean, well-scoped, back-compatible. Three items correctly
raised for slices 02–04 (not defects). CI is the only gate to `reproduced`.

## Checks (reproduced by CDC)

| Row | Result |
|-----|--------|
| L-1/L-2 (`[store]` locator + resolution) | **PASS** — new module `odm-store/src/home.rs`: `StoreLocation { worktree_base, worktree_name, branch_name }` (all optional, documented defaults) + `StoreHome::resolve`; module doc states it **"creates nothing: no worktree, no branch."** |
| L-3 (back-compat, no `[store]`) | **PASS** — resolves to the un-redirected default; CC reproduced on odm's own corpus (`check` green, 60 nodes, `nodes/` unmoved, no `.worktrees/` invented). |
| L-4/L-5 (two-stage load; `config.toml` + fallback) | **PASS** — `config.rs::from_home` loads operational config from the resolved home; absent `config.toml` falls back to `odm.toml`. |
| L-6 (node tree follows the store) | **PASS** — CC asserts both halves: with a hand-placed `.worktrees/odm/`, `odm new` writes into the worktree **and** nothing appears at the repo root. |
| L-7 (**no** git subprocess / worktree ops) | **PASS (reproduced)** — `grep Command::new("git")` over `crates/` returns only `odm-reconcile`'s shell probe; `git.rs` untouched. The scope boundary held. |
| L-8 (build/test/clippy/fmt) | attested-by-CC → CI — 54 test binaries, 0 failed, **+17 new**; clippy `-D warnings`, fmt clean. |
| L-9 (`check` green both modes) | attested-by-CC → CI — green with and without `[store]`. |

## Decisions CC recorded — endorsed

1. **Resolution never fails.** A malformed `[store]` falls back to the default rather than erroring —
   a typo must not become "odm can't find your data." (Strict parsing + its error stay in
   `StoreConfig::load` on the same file.) Correct call — resolution is a read path, and losing the
   store over a typo is the worse failure.
2. **`root` still means the invocation root.** Only the **node tree** follows the store; the config
   search and relative-path args stay `cwd`-relative. Sound — keeps CLI ergonomics stable.

## Carried to slices 02–04 (raised, not dropped — good bubble-up)

1. **`.odm/` splits across two roots.** The index follows `store.root()`, but `.odm/context.json`
   does not — arguably it *should* follow the store, but moving it is a behaviour change out of
   slice 01's scope. **→ slice 02** (and it touches the store-home model directly; worth an explicit
   call there).
2. **`ROLLUP.md` stays at the invocation root.** Defensible for a *projection out of* the store, but
   **ODD-0022 doesn't make that call.** **→ decide in slice 02/03** (and relevant to the LLM arc's
   flat-rollup work).
3. **`branch_name` parsed but unconsumed** until slice 02 (bootstrap/attach need it). Expected.

## Ledger

- **SH-1 → `attested`** (structural verify + CC attestation). Flips `reproduced` on CI.
- **Silent-drop diff:** none (CC-reported; confirmed against the slice-doc scope).
- Remaining: CI green on `sh-slice01-store-resolution`. Then slice 02 (`git` plumbing + `init`
  bootstrap) — which should also dispose of carried items #1/#2.
