# Closing Report — Slice 01 (Store Lifecycle): `store commit`

> Verified by: CC (this session), attested. CDC reproduction pending — this report and `ledger.md` are the
> evidence trail for that pass. Closed 2026-08-02 on `release/1.0.x`.

## The walk

Per-row disposition is in `ledger.md`'s Evidence column; this is the narrative behind it, not a restatement.

**F-1/F-2/F-3/F-4/F-5 (the command surface)** landed as specified: `StoreCommand::Commit { message, dry_run,
json }` in `crates/odm-cli/src/lib.rs`, dispatched to `store_cmd::commit` in `crates/odm-cli/src/store_cmd.rs`.
The command resolves `StoreHome`, refuses (rather than silently operating at the repo root) when there is no
`[store]` section, opens the store worktree via `odm_store::Repo::open`, computes the node delta, and:

- a clean worktree (`Repo::is_clean`) is a no-op — exit `0`, "nothing to commit", never an empty commit (F-3);
- otherwise the message is the node-delta summary (`odm_store::delta::NodeDelta::summary`) unless `-m`
  overrides it (F-2);
- `--dry-run` reports the same delta/message and returns before calling `Repo::commit_all` (F-4);
- `--json` emits `{committed, sha, branch, message, delta}` on every path, including the no-op and dry-run
  ones, with `sha: null` when nothing was written rather than omitting the key (F-5).

New `odm-store` surface: `Repo::tree_delta` (`git.rs`) diffs `HEAD`'s tree against the worktree for one
subtree, restricted to `nodes/` by the caller, without touching git's on-disk index — it walks the git tree
object for the "before" side and writes content-addressed blobs for the "after" side (same idiom
`write_tree` already used for `is_clean`/`commit_all`). `odm_store::delta` (new module) turns that into a
`NodeDelta` classified by `NodeType` (parsing each changed file's frontmatter; unparseable files count as
`"unknown"` rather than being dropped, so the total never silently under-reports) and renders the D-1
auto-summary.

**F-1/F-6 (real orphan-branch fixture) is where the slice earned its scope.** `crates/odm-cli/tests/store_commit.rs`
uses `init::bootstrap` to stand up a **real** worktree + orphan branch (per the slice-doc's class-(a) call),
not a hand-placed store — and running the actual compiled `odm` binary end-to-end against it (manual smoke
test, not just the in-process fixture) surfaced a genuine defect: see below.

## D-1: the auto-summary decision

Taken as the slice-doc recommended: the default message is the **git-delta node count**, by type
(`store: +2 slice, ~3 arc, ~2 design, -1 project`), computed from `HEAD` vs. the worktree — not a
migrate-aware summary (`reconciled`/`minted`/`collapsed`). That richer summary would need `migrate` to hand
off a pending-summary structure to `commit`, which does not exist yet and is out of this slice's scope per
the constraints ("recommend the git-delta version, flag the migrate-aware handoff as a future enhancement,
don't build it"). Flagging it here as that future enhancement, not building it.

One implementation decision beyond D-1's own scope: when the worktree is dirty but the **node** delta is
empty (e.g. only `config.toml` changed), the default message falls back to `"store: config/settings update"`
rather than an empty `"store: "` string. This wasn't specified explicitly; it's the natural reading of F-2
("default = pending node delta") extended to the case the criterion didn't enumerate.

## The staged-set and author-identity decisions (F-6)

The slice-doc asked to **confirm and flag**, not silently assume, two things: what actually gets staged in a
commit, and whose identity the commit carries.

**What's staged.** `Repo::commit_all` (unchanged, pre-existing) commits *everything* in the worktree except
`.git` — not just `nodes/`. Confirmed via the smoke test: a real commit after `store init` + `node new`
carried `.gitignore`, `config.toml`, and the new node file. That's correct and matches ODD-0022 — the store's
`.gitignore` and `config.toml` are meant to be versioned with the data they govern.

**What's staged, corrected.** Confirming the `.odm/` index's tracked-vs-ignored status surfaced that it
*wasn't* honored: `commit_all` builds its tree by walking the filesystem directly (`git.rs`'s own doc comment
says as much — "it never goes through the on-disk index"), so it never consulted the store's own
`.gitignore` (`STORE_GITIGNORE` in `init.rs`: `/.odm/*` plus `!/.odm/context.json`, written by `store init`
itself). The first time any command populated `.odm/index` or `.odm/drift` (essentially any read command —
`orient`, `validate`, `list`, …) before a `store commit`, those derived, regenerable caches would have been
committed permanently onto the orphan branch — an ODD-0022 violation ("odm owns the orphan branch," implying
it honors its own scaffolded `.gitignore`) that, per ODD-0022 §6, can never be silently undone (no history
rewrite). Confirmed via a real-binary smoke test (`store init` → `node new` → `orient` → `store commit` →
`git show --stat`): before the fix, `.odm/drift` and `.odm/index` were in the commit; after, they are not.

**This was fixed, not merely flagged**, under the working agreement's "amend don't work around": it is a
defect in shared git plumbing (`git.rs::write_tree`, used by both `is_clean` and `commit_all`), not a design
decision this slice is making, and leaving it in would make `store commit`'s first real use actively harmful.
`write_tree` now builds a `gix` exclude stack once per commit (`Repository::excludes`, reading the on-disk
`.gitignore` the same way `git add -A` would) and skips any path it matches, correctly honoring negation
(`!/.odm/context.json` stays tracked). Regression-fixtured directly in `odm-store`
(`edge_cases.rs::commit_all_honours_the_worktrees_own_gitignore`, using a synthetic `.gitignore` with a
negated re-admission) rather than only through the `.odm/`-specific case, so the fix is proven generically,
not just for the one file that happened to surface it. Existing `write_tree`/`commit_all`/`is_clean` fixtures
(none of which use a `.gitignore`) were re-run and are unaffected.

**Commit author identity.** `odm-stamped`, not ambient git config: `git.rs`'s pre-existing `commit_all` signs
every commit as `odm <odm@localhost>` (`SignatureRef` built inline, not read from `git config`), which
`store commit` inherits unchanged. Confirmed via the smoke test's `git show`, `Author: odm <odm@localhost>`.
This was already true of the one existing caller (`init`'s bootstrap commit, once one is made); `store commit`
doesn't change it, just makes it observable on a second and later commit.

## Scope discipline

Diff is confined to `odm-store` (`git.rs`, new `delta.rs`, `lib.rs` re-exports) and `odm-cli`
(`lib.rs`'s `StoreCommand`, `store_cmd.rs`), plus fixtures in both crates. No `odm-core` change, no
node-frontmatter-schema change, `auto_stage_git` untouched (stays dormant), `store status`/`store sync` not
built (the node-delta computation they'll reuse is factored into `odm_store::delta::compute`, per the
slice-doc's ask, and used as-is with no rework needed). ODD-0022 was not amended — the `.gitignore` fix
brings `write_tree` into line with what ODD-0022 and the store's own scaffolded `.gitignore` already
promised, rather than changing the store model; flagged in the ledger (F-7) for CDC to confirm that
judgment rather than asserting it unilaterally.

## Iterations

One pass, no rework loop: the command surface, the delta computation, and the fixtures converged on the
first implementation. The `.gitignore` finding was caught by *scope-appropriate verification* (the real-binary
smoke test the slice-doc's class-(a) fixture discipline calls for), not by a failed fixture and a fix-loop —
so it doesn't count against the five-iteration budget in the usual sense, but is recorded here as the reason
the diff is larger than the command surface alone.

## v2.0 bubble-up → `../arc-plan.md`

- **SL-1 done**: `store commit` is native — persists the worktree's pending node changes on the orphan
  branch, auto-summary + `-m`, idempotent no-op when clean, `--dry-run`/`--json`. Ledger F-1…F-8 all `done`,
  attested; CDC reproduction is the open item.
- **New, arc-relevant finding to carry into s02 (`store status`) and s03 (`store sync`)**: any future verb
  that reads or writes the store worktree via `odm-store`'s filesystem-walking git plumbing (as opposed to
  `gix`'s index-based APIs) needs the same exclude-awareness `write_tree` now has — `store status`'s "is the
  worktree dirty" read goes through `Repo::is_clean`, which already inherits the fix, so no further action is
  expected there, but it's worth an explicit check when s02 is implemented rather than assumed.
- Per the arc-plan's scoped-run note ("do s01 only, then ⏸ PAUSE this arc"), **s02/s03 stay paused** — this
  report closes the slice, not the arc. The arc ledger's SL-1 row is ready to be marked from this slice's
  evidence when the arc resumes.
