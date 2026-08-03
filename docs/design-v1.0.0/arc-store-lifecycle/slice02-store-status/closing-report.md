# Closing Report — Slice 02 (Store Lifecycle): `store status`

> Verified by: CC (this session). F-1…F-8 attested (real end-to-end `odm-cli` integration tests
> against a bootstrapped orphan-branch store, plus a real bare-repo remote for the ahead/behind
> rows — no mocking). Closed 2026-08-03 on `release/1.0.x`.

## The walk

Per-row disposition is in `ledger.md`'s Evidence column; this is the narrative.

**The command.** `StoreCommand::Status { json: bool }` (`lib.rs`) dispatches to
`store_cmd::status` — the read half of `commit`'s coin. It resolves the store home, opens the
worktree's `Repo`, computes the node delta via `odm_store::delta::compute` (**the identical
function `commit` calls** — F-1's invariant is enforced by construction, not just tested), reads
`Repo::is_clean()` for the clean/dirty split, and separately computes the upstream comparison via
the same remote-tracking primitives `init.rs`'s sync arm already uses
(`worktree::rev_parse`/`is_ancestor`/`count_commits`, `init::Ancestry`/`SyncAction`/
`sync_action()`) — with **no fetch** (D-1). Reports both dimensions, `--json` or plain text, never
writes anything.

**F-1/F-2 (the delta, and the clean path).** `dirty_store_matches_commits_own_delta_computation`
is the row's real teeth: it runs `status --json` and `commit --dry-run --json` against the
identical dirty worktree and asserts the two `delta` objects are equal — not merely that each
looks individually plausible. The clean-path wording ("nothing to commit") is driven by
`Repo::is_clean()`, matching exactly what `commit` branches on — not `delta.is_empty()`, which
would have quietly diverged from `commit`'s own semantics the moment someone edits `config.toml`
without touching a node (a case `commit`'s existing `auto_message` fallback already exists to
handle, and `status`'s `delta_phrase` now reuses that same fallback for consistency).

**F-3 (ahead/behind — the new dimension).** The cc-prompt's own suggestion for a fixture — "a
`TempDir` bare repo as upstream" — worked exactly as sketched. `with_fetched_upstream` stands up a
real `git init --bare` repo, `remote add origin`, pushes the orphan branch's current state, and
fetches — a genuine remote-tracking branch, no mock. `local_ahead_of_upstream_reports_the_count`
then makes two further unpushed commits and asserts `ahead: 2, behind: 0, action: "local-ahead"`;
`synced_store_reports_up_to_date` confirms `ahead: 0, behind: 0, action: "up-to-date"` right after
a push+fetch. One deliberate widening beyond the cc-prompt's `Ancestry`-reuse suggestion: `Ancestry`
only ever carried `local_ahead` (all `init::sync()`'s own callers needed), but `status`'s JSON
contract wants `ahead` **and** `behind` unconditionally — so `status` computes `behind` as an
independent `count_commits(local..upstream)` alongside the `Ancestry`-driven `ahead`/action
classification, rather than stretching `Ancestry` itself to carry a field its other caller has no
use for.

**F-4 (no upstream).** Falls out of the same code path for free: `rev_parse` on the upstream ref
returns `None` when there's no remote or no tracking branch, the ancestry match's fallthrough arm
calls `init::sync_action(None)` → `SyncAction::NoUpstream`, and `status` reports `upstream: null`
(JSON) / "no upstream configured" (text) — never an error.

**F-5 (`--json` shape).** Implemented per the cc-prompt's sketch: `StatusJson{clean, branch,
store_root, delta, upstream: Option<UpstreamJson>}`, `UpstreamJson{ref, action, ahead, behind}`.
One naming note: `ref` is a Rust keyword, so the struct field is `reference` with
`#[serde(rename = "ref")]` — the wire shape is exactly what the cc-prompt specified. `action`'s
four possible values (`up-to-date`/`local-ahead`/`upstream-ahead`/`diverged`) deliberately drop the
`SyncAction::mode()`-style `"sync-"` prefix `store init`'s sync arm uses for its own `mode` field —
`status` isn't a sync, and prefixing every value with a verb this command never performs would
have read oddly; there's no `no-upstream` action value because that state is `upstream: null`
instead (F-4), not a fifth action string.

**F-6 (read-only — the distinguishing constraint).**
`status_never_mutates_the_store_or_the_orphan_branch` dirties the worktree, sets up a real fetched
upstream (so there's ancestry work to do, not just a no-op), snapshots `HEAD` oid + `git status
--porcelain` + the store's node-path listing, runs `status --json`, and asserts all three
unchanged. It also runs `status` a second time and diffs the JSON output byte-for-byte against the
first run — proof no hidden fetch silently moved the remote-tracking ref between calls, which a
before/after-only snapshot could miss if the two `status` invocations happened to fetch identical
state both times. Enforced by construction as well: the implementation calls only
`delta::compute`, `Repo::is_clean`, and the three read-only `worktree` functions — never
`commit_all`, `fetch`, `merge_ff_only`, or any index/worktree write.

**F-7 (no model drift).** Diff is `odm-cli/src/lib.rs` (one enum variant + one dispatch arm),
`odm-cli/src/store_cmd.rs` (the `status` function + two small `Serialize` structs + three small
helpers), and the new `store_status.rs` test file. **Zero changes to `odm-store`** — every
function `status` calls already existed and was already fully `pub` (not `pub(crate)`), so the
cc-prompt's contingency note about widening visibility for `store_cmd.rs` to reach `init.rs`'s
plumbing turned out to be unnecessary; the existing surface was already sufficient. No ODD-0022
amendment — this is exactly the plumbing exposure the model already implied.

**F-8.** `make lint` (clippy `-D warnings` + rustfmt `--check`) clean; `grep unsafe` on every
new/changed file finds none; `make test` (full workspace, all crates + doctests) green. All 12
`store_status.rs` fixtures pass, covering dirty/clean/ahead/synced/no-upstream/json/read-only/
no-store-section.

## Scope discipline

Diff: `odm-cli/src/lib.rs`, `odm-cli/src/store_cmd.rs`, `odm-cli/tests/store_status.rs` (new). No
`odm-store` change (every needed function was already public). No `--dry-run` flag added (the
cc-prompt scoped `status` as inherently dry — nothing to preview). No `--fetch` flag added — D-1
stands as designed, flagged rather than silently built. `store sync` (s03) untouched.

## Iterations

One pass. The one design point requiring a small deviation from a literal reading of the
cc-prompt — computing `behind` independently rather than trying to make `Ancestry` itself carry
it — was flagged in the ledger's Notes rather than silently worked around, per the working
agreement (amend, don't work around).

## Bubble-up → `../arc-plan.md`

- **Slice 02 done**, delivering ledger row **SL-2**: `odm store status` reports the pending node
  delta (odm-aware, identical to what `commit` would write) and ahead/behind vs. the configured
  upstream, read-only, `--json`-capable. The lifecycle now reads `init → mutate → status → commit
  → sync` with three of the four verbs native.
- **What implementing it revealed that the arc-plan didn't need to anticipate further:** the
  `Ancestry`/`SyncAction` plumbing `init.rs`'s sync arm built for its own narrower need (one
  ahead-count, used only in the `LocalAhead` case) generalizes cleanly to `status`'s broader need
  (ahead **and** behind, always) without touching `odm-store` at all — a second independent
  `count_commits` call alongside the existing classification, not a refactor of the shared types.
  Worth naming for `s03` (`store sync`): the same plumbing is confirmed reusable a third time
  without modification.
- **Silent-drop check:** all 8 ledger rows closed done, none deferred or no-op. No rows dropped.
- **s03 (`store sync`) is next** — the arc's immediate priority per the 2026-08-03 resume note,
  now with `status` available as a pre-flight check the operator (or CDC, via the s05 musl binary)
  can run before syncing.
