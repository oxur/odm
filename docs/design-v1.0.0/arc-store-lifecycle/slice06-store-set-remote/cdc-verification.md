# CDC Verification — SL slice 06: `store set-remote`

> **Verifier:** CDC (Cowork cloud session, 2026-08-04)
> **Evidence method:** Independent code review (all changed files read via
> device bridge) + independent build and test run (cloud container, `rustc
> 1.95.0`, `cargo test --test store_set_remote` and full workspace `cargo
> test`)
> **Code:** operator's `release/1.0.x` HEAD (`47993a9`, "Close SL slice 06:
> store set-remote")

## Summary

**All eight ledger rows verified. No regressions.**

CDC independently built the codebase from a shallow clone of `release/1.0.x`
with the local commits patched in, then ran the full test suite. The 16 s06
fixtures pass; the remaining ~770 tests pass; the only failures (2 in
`odm-store/tests/edge_cases.rs`) are pre-existing and root-caused to the
container running as UID 0 (filesystem-permission tests don't bite). Clippy
is clean on both s06 crates (`odm-store`, `odm-cli`).

CC's implementation includes two deliberate deviations from the cc-prompt and
one unprompted fix, all three improvements over what was specified:

1. **`toml_edit::DocumentMut`** instead of the cc-prompt's `toml::Value`
   round-trip sketch — preserves comments and formatting byte-for-byte in
   existing `odm.toml` files. Already a dependency. Strictly better.
2. **`init.rs::write_locator` inline approach** for F-5 — the "simpler
   alternative" the cc-prompt offered. Correct choice: `init` writes a
   brand-new `[store]` section, so `toml_edit`'s format-preservation doesn't
   apply.
3. **`rename.rs` latent bug fix** — adding `remote` to `StoreLocation`
   surfaced that `rename`'s location-builders and its own `write_locator`
   didn't carry the field, meaning a store rename would have silently dropped
   a configured remote. Fixed by threading `remote` through both paths.
   Compiler-enforced: the exhaustive struct pattern caught it at build time.

## Per-row walk

### F-1 — `set-remote <name>`: explicit remote configuration

**Status: PASS (reproduced)**

Evidence:

- `store_set_remote.rs::set_remote_explicit` — creates a bare repo remote,
  runs `odm store set-remote origin`, reads `odm.toml` back and asserts
  `remote = "origin"` is present under `[store]`.
- `set_remote_explicit_nonexistent` — calls `set-remote nosuch`, asserts
  non-zero exit and `odm.toml` unchanged.
- `set_remote_explicit_overwrite` — sets `origin`, then sets `upstream`,
  reads back and asserts the second value wins.

Code path verified: `store_cmd::set_remote()` validates via
`worktree::remote_url()` (which calls `git remote get-url`), then
`write_remote_to_locator()` uses `toml_edit::DocumentMut` to edit `[store]`
in place, preserving all other content.

### F-2 — `set-remote` (no arg): auto-detect

**Status: PASS (reproduced)**

Evidence:

- `set_remote_auto_one` — one remote exists, auto-detects it,
  `auto_detected: true` in JSON output.
- `set_remote_auto_zero` — no remotes, error exit, nothing written.
- `set_remote_auto_multi` — two remotes, error exit listing both names.

Code path verified: `worktree::list_remotes()` (new function, follows
`capture()`'s existing pattern) returns the remote list; `set_remote()`
branches on `remotes.len()` — exactly-one, zero, or many.

### F-3 — `sync` first-push bootstrapping

**Status: PASS (reproduced)**

Evidence:

- `sync_first_push` — `store init` + `store set-remote` + `store commit`
  → `store sync --json` → action `"pushed"`. Asserts the bare remote's
  branch is absent before sync, present and equal to local HEAD after.
- `sync_first_push_dry_run` — same setup, `--dry-run` leaves the bare
  remote's branch absent, then a real sync pushes it.

Code path verified: `sync_cmd()` tracks `fetch_ok` (set to `true` when
`git fetch` succeeds even if the branch doesn't exist on the remote).
The new arm `(Some(_local), None) if fetch_ok` yields
`SyncAction::LocalAhead(count)`, reusing the existing push execution
path. Fetch-failed still falls through to true `NoUpstream`.

### F-4 — `sync` reads configured remote

**Status: PASS (reproduced)**

Evidence:

- `sync_reads_configured_remote` — **adversarial test**: creates *only* a
  remote named `"github"` (no `"origin"` at all), calls `set-remote github`,
  then syncs. If sync had used the hardcoded `DEFAULT_REMOTE` it would have
  failed outright. A passing push is direct proof, not an inference.
- `sync_falls_back_to_default` — `"origin"` remote exists but `set-remote`
  was never called; sync finds and uses it via the `DEFAULT_REMOTE` fallback.

Code path verified: `sync_cmd()` reads
`plan.location.remote.as_deref().unwrap_or(DEFAULT_REMOTE)`.

### F-5 — `store init` auto-sets remote

**Status: PASS (reproduced)**

Evidence:

- `init_auto_sets_remote` — single-remote repo, `store init`, `odm.toml`
  contains `remote = "origin"`.
- `init_no_remote_no_field` — no remotes, init succeeds, no `remote` line
  in `odm.toml`.
- `init_multi_remote_no_field` — two remotes, init succeeds, no `remote`
  line (ambiguity → don't guess).

Code path verified: `bootstrap()` clones `plan.location`, detects remotes
via `list_remotes()`, sets `location.remote = Some(name)` when exactly one
exists, then calls `write_locator()` which emits `remote = "{name}"` inline
when `location.remote.is_some()`.

### F-6 — `--json` for `set-remote`

**Status: PASS (reproduced)**

Evidence:

- `set_remote_json` — explicit arg, all five fields asserted: `remote`,
  `url`, `auto_detected` (false), `branch`, `store_root`.
- `set_remote_json_auto` — no arg, `auto_detected: true`.

Code path verified: `SetRemoteJson` struct at `store_cmd.rs` line 742
carries all five fields; error cases use `anyhow::bail!` (the house
pattern — no separate JSON error shape needed).

### F-7 — No model drift

**Status: PASS (reproduced)**

Diff scope confirmed via `git diff --name-only` — changes are confined to:

- `odm-store/src/home.rs` — one field added (`remote: Option<String>`)
- `odm-store/src/worktree.rs` — two new functions (`list_remotes`,
  `remote_url`)
- `odm-store/src/init.rs` — `bootstrap()` auto-set + `sync()` reads config
- `odm-store/src/rename.rs` — `remote` threaded through location builders
  (latent-bug fix)
- `odm-cli/src/lib.rs` — `SetRemote` variant + dispatch
- `odm-cli/src/store_cmd.rs` — `set_remote()` handler, `sync_cmd()` remote
  reading + first-push arm, `status()` one-line consistency fix
- `odm-cli/tests/store_set_remote.rs` — new, 16 fixtures

No node-schema change. No ODD-0022 amendment. `status()` changed by one
line (reads `location.remote` instead of hardcoding `DEFAULT_REMOTE`) —
the cc-prompt itself flagged this as a consistency fix and recommended it
explicitly. The rename.rs fix is a genuine preservation bug caught by the
compiler's exhaustive struct pattern.

### F-8 — Clippy clean; no `unsafe`; new code covered

**Status: PASS (reproduced)**

CDC ran independently in cloud container:

```
$ cargo clippy -p odm-store -p odm-cli 2>&1 | tail -1
Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.95s
```

No warnings. (One unrelated clippy lint in `odm-core/tests/check.rs:269`
from a different commit — `cloned_ref_to_slice_refs`, new in clippy 1.95.0
— is pre-existing and outside s06's scope.)

`grep -r 'unsafe' ` on all changed files: none.

Test coverage: 16 new fixtures in `store_set_remote.rs` cover all ledger
rows. Full workspace test suite: ~770 tests pass; 2 pre-existing failures
in `edge_cases.rs` (root-in-container, confirmed identical without the s06
patch).

## Guard test

`without_a_store_section_set_remote_refuses` — calling `set-remote` in a
repo whose `odm.toml` has no `[store]` section exits with an error,
preventing configuration of a remote for an un-redirected store. Passes.

## Deviations from cc-prompt

| # | What | CC's choice | CDC assessment |
|---|------|-------------|----------------|
| 1 | Config editing approach | `toml_edit::DocumentMut` instead of `toml::Value` round-trip | **Better.** Already a dependency; preserves comments; the cc-prompt itself anticipated this ("switch to toml_edit... but do not add that dependency in this slice" — it was already present). |
| 2 | `init` remote writing | Inline `write!` in `write_locator()` (the "simpler alternative") | **Correct choice.** `init` writes a new `[store]` from scratch — `toml_edit`'s format preservation doesn't apply here. |
| 3 | `rename.rs` bug fix | Unprompted: threaded `remote` through `target_location()`, `observed_location()`, and `rename::write_locator()` | **Real bug.** Without this, `store rename` would silently drop a configured remote. Compiler caught it (exhaustive struct pattern). The cc-prompt didn't anticipate this because `rename.rs` wasn't in scope — but the compiler's audit was more thorough than the spec's, which is how it should work. |

## Bubble-up to arc

CC's closing report flagged the `rename.rs` compiler-enforced audit pattern
as a positive signal for future slices: adding a field to `StoreLocation`
automatically surfaces every location-building site that needs updating.
This is a structural property of the codebase, not a one-off — future
`StoreLocation` extensions (if any) will get the same protection. Worth
noting in the arc-plan's closing assessment as a design-quality indicator.

## Overall assessment

**PASS.** All eight ledger rows reach `reproduced` evidence strength. CC's
implementation is faithful to the spec where it matters (the behavior) and
better than the spec where it deviated (the mechanism). The three deviations
are all improvements. Zero regressions. The slice is ready to close.
