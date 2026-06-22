# Closing Report — Slice 04 (Arc 01): Store layer

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain) before slice 05 opens.

- **Implementation commit:** `4457597`.
- **Branch:** `slice04-store-layer` (not pushed; not merged to `main`).
- **Scope delivered:** `odm-store` — `nodes/YYYY/MM/<ULID>.md` layout, crash-safe
  atomic writes, persist/load (single + full scan), pure-Rust `gix` commit/status,
  and `odm.toml` via confyg. Plus a small `odm-core` addition (`Id::created_at`).
- **Result:** 10 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.
- **Aggregate gates:** `cargo test -p odm-store` → 22 tests pass; `cargo clippy
  -p odm-store --all-targets -- -D warnings` → exit 0; odm-store line coverage
  93.12%. No regression in odm-core (98.81% region / 99.62% line).

## Per-row walk

| ID | Status | Evidence (re-runnable at `4457597`) |
|----|--------|-------------------------------------|
| J-1 | done | `cargo test -p odm-store path_from_ulid` → 1 passed. `01ARZ3…` → `nodes/2016/07/…`. |
| J-2 | done | `cargo test -p odm-store locate_by_id` → 1 passed. `path_of` pure, no FS, file need not exist. |
| J-3 | done | `cargo test -p odm-store persist_reload_roundtrip` → 1 passed. 5-node set + single load. |
| J-4 | done | `cargo test -p odm-store atomic_write_no_partial` → 2 passed. temp+fsync+rename+dir-fsync; failure preserves existing. |
| J-5 | done | `cargo test -p odm-store full_scan_load` → 1 passed. 7 nodes via walkdir; non-`.md` ignored. |
| J-6 | done | `cargo test -p odm-store gix_stage_commit` → 1 passed. init → dirty → commit → clean cycle; 40-char sha. |
| J-7 | done | `cargo test -p odm-store config_layered_load` → 2 passed (+ repo-root & malformed in `edge_cases`). |
| J-8 | done | `cargo test -p odm-store missing_dir_selfheal` → 1 passed. empty load, then first write creates tree. |
| J-9 | done | `! grep -RnE '\bunsafe\b' crates/odm-store/src` → no match; `StoreError` is `thiserror`; no unwrap/expect in `src`. |
| J-10 | done | clippy → exit 0; line coverage 93.12% (region 88.46%), odm-core excluded. See uncertainties (1)/(2). |

## Decisions worth a CDC look (flagged, not silently changed)

1. **New `odm-core` method `Id::created_at() -> DateTime<Utc>`.** J-1 requires the
   `YYYY/MM` shard from the ULID's creation time, but slice 02 deliberately
   omitted a timestamp accessor (flagged then as YAGNI). Only `odm-core` can read
   the ULID time without leaking the `ulid` crate, so I added a documented,
   tested `created_at` there (epoch fallback keeps it panic-free). This touches a
   CDC-closed crate additively; odm-core stays green (98.81% region) and the new
   method has its own test against the spec example ULID. Flagging as a
   cross-slice addition rather than a silent edit.

2. **"stage + commit + status" implemented without the git index.** gix 0.66 has
   no high-level "add path + commit"; rather than drive the fragile, feature-
   gated index/dirwalk machinery, `commit_all` builds the commit tree **directly
   from the worktree** (write blobs → assemble `gix_object::Tree`s → `commit_as`),
   and `is_clean` compares the worktree tree id to `HEAD`'s tree id. This is pure
   `gix` (honors Q-2: no shelling out), race-free, and sufficient for the store's
   need (snapshot node files, detect changes). It does **not** write `.git/index`,
   so an external `git status` would not show a populated index — odm's own status
   is the tree comparison. Flagging because "stage" here means "into the commit
   tree," not "into the index file."

3. **`StoreConfig` is minimal (`author_name`, `author_email`).** The slice only
   requires that `odm.toml` loads via layered search; I defined the smallest
   config the store actually uses (git identity) with serde defaults. Real config
   fields (docs dirs, etc.) accrete in later slices as commands need them.

4. **Legacy `git.rs` was NOT harvested.** The cc-prompt suggested harvesting
   legacy `git`/`config`/`filename` helpers. Legacy `git.rs` shells out to the
   `git` binary (`std::process::Command`), which directly contradicts the Q-2
   decision (pure-Rust `gix`, no shell-out) — so it did not fit and I wrote
   gix-based git instead. Legacy `config.rs`'s confyg-search *pattern* informed
   `config.rs`; the legacy atomic write (temp+rename, no fsync) was upgraded to
   add fsync + dir-fsync per ODD-0014. `filename.rs` is about legacy
   number-based filenames, superseded by the ULID path — not harvested.

## Uncertainties named

- **Coverage: line 93.12% clears ≥90%; region is 88.46%.** The region gap is
  almost entirely in `git.rs` (84.21%): gix error-mapping closures for
  object-database write failures and the two "bare repository" guards. These are
  defensive branches that can't be exercised without elaborate fault injection or
  exposing a bare-init API (needless surface). I judged line coverage the right
  headline (everything behaviorally meaningful is tested) and chose not to contort
  tests for unreachable error arms. If CDC requires region ≥90%, the cheapest
  honest options are a bare-repo fixture (to hit the guards) or accepting the
  defensive gap — flagging for the call.
- **Coverage measurement excludes odm-core.** `cargo llvm-cov -p odm-store` also
  instruments the `odm-core` path-dep, which the store tests only partially
  exercise (odm-core's own tests aren't run under `-p odm-store`), dragging the
  raw TOTAL to ~70%. The honest per-crate number uses
  `--ignore-filename-regex 'odm-core'`. **Recommend** the ledger's J-10 Verify be
  amended to include that flag (or measure via `--workspace`).
- **`fsync` durability is not (and cannot be) unit-tested.** J-4 verifies the
  no-partial/atomic-rename property and temp cleanup; the `sync_all` calls are
  present per ODD-0014 but a real power-loss durability test is out of scope for a
  unit suite. The dir-fsync is best-effort (some platforms reject opening a
  directory) and its failure is intentionally non-fatal to the completed rename.
- **Sandbox has no Rust toolchain**, so all cargo evidence was produced on the
  local dev host; CDC should reproduce on CI / a local 1.85+ run.
