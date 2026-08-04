# Closing Report — Slice 06 (Store Lifecycle): `store set-remote`

> Verified by: CC (this session). F-1…F-8 attested (real end-to-end `odm-cli` integration tests
> against real bare-repo remotes — no mocking). Closed 2026-08-04 on `release/1.0.x`.

## The walk

Per-row disposition is in `ledger.md`'s Evidence column; this is the narrative.

**The gap this closes.** The operator's first live `store sync` against GitHub returned "nothing
to sync — has no upstream," even though `origin` existed and was reachable — the branch simply
hadn't been pushed yet, and `sync` couldn't tell that apart from "no remote at all." This slice
closes two separable problems the slice-doc named: **remote configuration** (previously hardcoded
to `DEFAULT_REMOTE`, no way to point at anything else) and **first-push bootstrapping** (a missing
upstream ref after a successful fetch means "push," not "nothing to sync").

**F-1/F-2 (the config command).** `store set-remote` — explicit (`set-remote <name>`, validated
against `git remote get-url` before writing) and auto-detect (`set-remote`, no argument, D-1: no
prompt, errors clearly on zero or multiple remotes). All three explicit-form fixtures and all three
auto-detect fixtures pass against real bare-repo remotes, reading `odm.toml` back and parsing it
after each run rather than trusting the command's own report of success.

**D-5, implemented differently than sketched — for the better.** The cc-prompt's own suggested
approach was a `toml::Value` parse → mutate → `to_string_pretty` round-trip, with D-5 explicitly
flagging the trade-off: that approach discards comments and manual formatting. `odm-cli` already
depends on `toml_edit` and already has the exact pattern needed
(`commands.rs::write_additional_paths`, which edits one key of the operational config while
preserving everything else byte-for-byte) — reusing it via `toml_edit::DocumentMut` gets D-5's
"if comment preservation becomes important later" concern solved immediately, at zero added
dependency cost (the cc-prompt's own caveat — "do not add that dependency in this slice" — doesn't
apply, since it was already present). `write_remote_to_locator` mirrors `write_additional_paths`'s
structure directly.

**F-3/F-4 (first-push and remote-reading — the actual bug the operator hit).**
`sync_first_push`/`sync_first_push_dry_run` reproduce the operator's exact scenario: `store init` +
`store set-remote` + `store commit` + `store sync` now pushes the store to a bare-repo remote on
the first run, with the dry-run variant confirmed to preview accurately (nothing pushed, then a
real run afterward does push). `sync_reads_configured_remote` is deliberately adversarial: it
configures **only** a remote named `"github"` — no `"origin"` exists anywhere in the fixture — so
if `sync` had kept using the hardcoded default, the push would fail outright rather than merely
going to the wrong place. A passing push is direct proof the config is read, not an inference from
absence of failure. `sync_falls_back_to_default` is the mirror case: an `"origin"` remote exists,
`set-remote` is never called, and `sync` still finds and uses it — backward compatibility for every
existing store.

**F-5 (`store init` auto-sets remote) — the simpler of two offered approaches.** The cc-prompt
offered two implementations and invited a flagged choice: a parse-after-write TOML round-trip
(matching D-5's general shape), or teaching `write_locator` (init.rs's own, separate from
`rename.rs`'s) to emit `remote` inline when present. Chose the second: `bootstrap()` now builds a
local `location` (cloned from the plan, remote auto-detected via `worktree::list_remotes` before
the locator write) and passes it straight through. Simpler, no parse-serialize round-trip, and
consistent with `write_locator`'s existing plain-string-formatting style — `toml_edit`'s
comment-preservation benefit doesn't apply here anyway, since this path writes a **brand-new**
`[store]` section from scratch (there's nothing to preserve yet), unlike `set-remote`'s job of
editing an existing one.

**An unprompted second `odm-store` fix, found while wiring the field through.** F-7's own summary
line anticipated `list_remotes()` and the `remote` field as the only new `odm-store` surface. While
adding `remote: Option<String>` to `StoreLocation`, `cargo build` surfaced every place that
constructs the struct as a literal — and `rename.rs`'s `target_location()`/`observed_location()`
helpers, plus its own separate `write_locator`, did not carry the field through at all. Left
unfixed, the first `odm store rename` on a store with a configured remote would have **silently
dropped it** — `sync` would revert to hardcoded `DEFAULT_REMOTE` right after a rename, with no
error, no warning, just quietly wrong behavior the next time someone ran `sync`. Fixed by threading
`remote` through both location-builders in `rename.rs` and having its `write_locator` emit the
field when present. `odm-store`'s full 49-test suite, including every `rename::tests` case, stays
green. Flagged here rather than silently folded into the "expected" diff.

**F-6 (`--json`).** Implemented exactly per spec: `{remote, url, auto_detected, branch,
store_root}`. Error cases (invalid remote, zero/multiple auto-detect) use the house
`anyhow::bail!` pattern — non-zero exit regardless of `--json`, no separate JSON-error-shape
mechanism needed, matching every other command in this file.

**F-7 (no model drift, with two flagged deviations).** Diff scope: `odm-store`'s `home.rs`
(field), `worktree.rs` (two new helpers), `init.rs` (auto-set + remote-reading), `rename.rs` (the
preservation fix); `odm-cli`'s `lib.rs` (new variant), `store_cmd.rs` (`set_remote` + `sync_cmd`
changes + `status`'s one-line consistency read), and the new test file. `status`'s change is the
one the cc-prompt itself pre-flagged as "a consistency fix, not a modification in the F-7 sense" —
without it, `status` and `sync` would disagree about which remote "ahead/behind" means the moment
a non-default remote is configured. Implemented as recommended.

**F-8.** `make lint` (clippy `-D warnings` + rustfmt `--check`) clean; `grep unsafe` on every
new/changed file finds none; `make test` (full workspace, all crates + doctests) green. All 16
`store_set_remote.rs` fixtures pass; the three existing store-lifecycle test files
(`store_commit`/`store_status`/`store_sync`) pass unchanged, confirming backward compatibility.

## Scope discipline

Diff: `odm-store/src/{home,worktree,init,rename}.rs`, `odm-cli/src/{lib,store_cmd}.rs`,
`odm-cli/tests/store_set_remote.rs` (new). No remote *management* (`add`/`remove`/`rename` of git
remotes themselves) — `set-remote` only chooses which existing remote the store syncs to, per the
slice-doc's explicit scope boundary. No multi-remote fan-out. `commit` untouched.

## Iterations

One pass for the designed behavior. One unplanned but necessary fix (`rename.rs`'s field
preservation), caught by the compiler itself (a missing-field error at every construction site)
rather than by a failing test — resolved in the same pass, no design rework needed. Well inside
the five-iteration cap.

## D-1…D-5 disposition

- **D-1 (auto-detect, no prompt):** implemented as specified.
- **D-2 (config location — `[store].remote`):** implemented as specified; backward-compatible via
  `#[serde(default)]`.
- **D-3 (`store init` auto-sets remote):** implemented as specified, via the simpler of the two
  offered mechanisms (see F-5's Notes).
- **D-4 (first-push in `sync`):** implemented as specified; JSON `action` is `"pushed"`, matching
  `LocalAhead`'s existing value, from the caller's perspective it *is* a push.
- **D-5 (TOML update approach):** implemented via `toml_edit::DocumentMut` rather than the
  sketched `toml::Value` round-trip — see F-1's Notes for the justification.

## Bubble-up → `../arc-plan.md`

- **Slice 06 done**, delivering ledger row **SL-8**: `store set-remote` configures the sync
  target; `store init` auto-sets it when unambiguous; `sync` reads the configured remote (falling
  back to `DEFAULT_REMOTE` for old stores) and handles the first-push case. The operator's blocked
  first `store sync` against GitHub is unblocked — `init → set-remote → commit → sync` (or, for
  single-remote repos, `init` alone auto-configures it) pushes on the first run, no raw git.
- **What implementing it revealed that the arc-plan didn't need to anticipate further:** adding a
  field to a widely-constructed struct is a good compiler-enforced audit of every construction
  site — the `rename.rs` gap would very likely have shipped silently otherwise, since none of the
  existing rename fixtures had any reason to exercise a `remote`-carrying `StoreLocation` (the
  field didn't exist until this slice). Worth naming for future slices that add a field to a
  struct with several independent constructors: let the compiler enumerate them, don't rely on
  remembering where they all are.
- **Silent-drop check:** all 8 ledger rows closed done, none deferred or no-op. No rows dropped.
  Two deviations from the cc-prompt's sketch flagged rather than silently absorbed: the D-5
  `toml_edit` choice, and the `rename.rs` preservation fix as a second `odm-store` change beyond
  what F-7 anticipated.
