# Slice 04 (Arc 01): Store layer

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| J-1 | Path = `nodes/<YYYY>/<MM>/<ULID>.md`, month from the ULID's creation timestamp | `cargo test -p odm-store path_from_ulid` → ok | serious | 0013 §6 | open | | |
| J-2 | Locate-by-id is O(1): path is a pure function of the id (no scan needed) | `cargo test -p odm-store locate_by_id` → ok | serious | 0013 §6 | open | | |
| J-3 | Persist → reload round-trips a node set identically (uses slice03 emit/parse) | `cargo test -p odm-store persist_reload_roundtrip` → ok | serious | 0013 §6 | open | | |
| J-4 | Atomic write: temp + fsync + rename + dir-fsync; a simulated mid-write failure leaves no partial/corrupt file | `cargo test -p odm-store atomic_write_no_partial` → ok | serious | 0014 | open | | |
| J-5 | Full-scan load reads every `.md` under `nodes/` into memory | `cargo test -p odm-store full_scan_load` → ok | serious | 0013 §6 | open | | |
| J-6 | `gix` stage + commit works against a temp repo; status reflects changes | `cargo test -p odm-store gix_stage_commit` → ok | correctness | 0013 §6 | open | | |
| J-7 | `odm.toml` loads via confyg layered search (cwd → repo root → user) | `cargo test -p odm-store config_layered_load` → ok | correctness | 0013 §6 | open | | |
| J-8 | Missing `nodes/` dir self-heals (created on first write; empty load is not an error) | `cargo test -p odm-store missing_dir_selfheal` → ok | correctness | 0013 §6.1 | open | | |
| J-9 | No `unsafe`; no panics on public paths; typed errors | `! grep -RnE '\bunsafe\b' crates/odm-store/src` AND errors are `thiserror` types | serious | rust-guidelines | open | | |
| J-10 | Clippy clean (`-D warnings`); coverage ≥ 90% (target 95%) | `cargo clippy -p odm-store --all-targets -- -D warnings` → exit 0 AND `cargo llvm-cov -p odm-store --summary-only` ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(Filled in at slice close.)_

## Closure

Closed at `<SHA>` on `<date>`. CDC: `<name>` (cargo rows via CI/local 1.85+).
Total rows: 10. Done: _. Deferred: _. No-op: _.
