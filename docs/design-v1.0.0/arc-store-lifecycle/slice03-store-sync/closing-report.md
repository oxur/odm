# Closing Report — Slice 03 (Store Lifecycle): `store sync`

> Verified by: CC (this session). F-1…F-8 attested (real end-to-end `odm-cli` integration tests
> against a bootstrapped orphan-branch store and a real bare-repo remote — push, pull, and
> divergence all reproduced against genuine git history, no mocking). Closed 2026-08-03 on
> `release/1.0.x`.

## The walk

Per-row disposition is in `ledger.md`'s Evidence column; this is the narrative.

**The command.** `StoreCommand::Sync { dry_run, json }` (`lib.rs`) dispatches to
`store_cmd::sync_cmd` — a single bidirectional verb, per D-1. It fetches unconditionally (D-2),
computes the same `Ancestry`/`SyncAction` classification `status` (s02) already established as
reusable a second time without modification, and acts on the result: `FastForward` fast-forwards,
`LocalAhead` pushes, `UpToDate`/`Diverged`/`NoUpstream` are no-ops. **D-1 confirmed correct**: the
five-arm decision table maps cleanly onto "what should sync do," with no case needing a second
verb.

**F-1/F-2 (pull and push — the real teeth).** Both directions are proven against a genuine
bare-repo remote, not a mock: `bootstrapped_store_with_upstream()` stands up a store, commits once,
and pushes it to a fresh `git init --bare` repo; `advance_bare_repo()` simulates "someone else
pushed" via a **second, independent clone** (not direct ref manipulation on the bare repo, which
would be a weaker fixture — a real clone-commit-push round-trip is what an actual second
collaborator's push looks like). `local_ahead_pushes_to_the_remote` and
`upstream_ahead_pulls_and_fast_forwards` each assert the *actual* git state (`rev-parse` on both
sides) matches, not just that the command printed the right word.

**The real finding: `merge_ff_only` was never actually exercisable.** The first fixture run for
F-1 failed outright with `fatal: No remote for the current branch` — not a fixture bug, but a
genuine defect in `odm-store`. `worktree::merge_ff_only(worktree_dir)` ran a bare `git merge
--ff-only` with no target ref, which depends on `git`'s upstream-tracking config
(`branch.<name>.merge`) to know what to merge from. Nothing in `odm-store` ever sets that config —
not `store init`'s bootstrap or attach arms, not a plain `fetch`/`push` without `-u`. This means
`init::sync()`'s own FastForward arm — the pull half of `store init`'s existing sync arm, landed in
an earlier arc — had **never actually been fixture-tested end to end**; its test suite covers
`sync_action()`'s pure decision-table logic exhaustively, but nothing had ever driven a real
`git merge --ff-only` call through it. This slice's fixtures are the first to do that, for either
call site.

**The fix** (not a workaround): `worktree::merge_ff_only` now takes an explicit `upstream_ref: &str`
and runs `git merge --ff-only <ref>` — correct regardless of whatever tracking config does or
doesn't exist, which is the more robust design anyway (no implicit git state to get right). Both
call sites were updated: `init.rs::sync()` (passing the `upstream_ref` it already had in scope) and
this slice's `sync_cmd`. `odm-store`'s own 49-test suite, including every `init::tests` case, stays
green after the change — the signature widening is additive, not behavior-changing for any caller
that already worked. This is flagged explicitly (ledger F-1/F-7 Notes) as a second `odm-store`
change beyond the cc-prompt's anticipated sole addition (`worktree::push()`), per the "amend, don't
work around" working agreement — the alternative (leaving the bug and finding some other way to
make the fixture pass) would have shipped a `store sync` whose pull path was still broken.

**F-3 (divergence — the discipline that matters most).**
`diverged_store_changes_nothing` advances both sides independently, snapshots both branch tips,
runs `sync`, and asserts **both** tips are byte-identical to their pre-run values — not merely that
the JSON says `"diverged"`. This is also the fixture that proves push-never-forces in the case that
actually matters: `sync` classifies `Diverged` *before* ever attempting a push, so the remote is
never even approached, let alone force-pushed over.

**F-4 (no-op cases).** Falls out of the same classification for free, mirroring `status`'s s02
handling exactly: `UpToDate` and `NoUpstream` are both `SyncAction` arms already excluded from the
mutating `if !dry_run` match.

**F-5 (`--dry-run`).** Each dry-run fixture does more than assert nothing changed — it also runs a
**real** sync immediately afterward and confirms the previewed action *does* then happen, proving
the preview matched reality rather than merely doing nothing safely. `ahead`/`behind` in the
`--dry-run` JSON reflect the state right after `fetch` (which always runs), before the (skipped)
merge/push — exactly the "preview values" the cc-prompt specified.

**F-6 (`--json` shape).** Implemented per the cc-prompt's exact spec. One deliberate naming choice:
`sync`'s `action` values (`pushed`/`pulled`/`up-to-date`/`diverged`/`no-upstream`) are a **separate
vocabulary** from `status`'s `action` values (`up-to-date`/`local-ahead`/`upstream-ahead`/
`diverged`) — `status` describes a *state* (what's true right now), `sync` describes an *outcome*
(what happened, or under `--dry-run`, what the state calls for doing). Reusing one vocabulary for
both would have made `local-ahead` mean two different things depending on which command printed it.

**F-7 (no model drift).** Diff is `odm-cli/src/lib.rs` (one enum variant + one dispatch arm),
`odm-cli/src/store_cmd.rs` (`sync_cmd` + two small `Serialize` structs + one helper), and the new
`store_sync.rs` test file — plus the `odm-store` fix described above (`worktree::push()`, new, and
`worktree::merge_ff_only`'s widened signature, a bug fix). No ODD-0022 amendment: the push/pull/
divergence discipline this slice implements is exactly what ODD-0022 §6 already specifies.

**F-8.** `make lint` (clippy `-D warnings` + rustfmt `--check`) clean; `grep unsafe` on every
new/changed file finds none; `make test` (full workspace, all crates + doctests) green. All 13
`store_sync.rs` fixtures pass.

## Scope discipline

Diff: `odm-cli/src/lib.rs`, `odm-cli/src/store_cmd.rs`, `odm-cli/tests/store_sync.rs` (new),
`odm-store/src/worktree.rs` (`push()` + `merge_ff_only`'s signature + the module doc header's
"three times" update), `odm-store/src/init.rs` (one call-site update). No `--force` anywhere in the
push path. No merge/rebase resolution for divergence. No node-schema change. `status` (s02) and
`commit` (s01) untouched; `init::sync()` untouched apart from the one-line call-site fix required by
the `merge_ff_only` signature change.

## Iterations

One pass for the designed behavior; one root-cause fix (`merge_ff_only`) discovered by the first
real fixture run and resolved in the same pass rather than treated as a second iteration — the fix
was mechanical once diagnosed (pass the ref explicitly) and required no design rework. Well inside
the five-iteration cap.

## D-1/D-2/D-3 disposition

- **D-1 (single bidirectional verb):** confirmed correct — the five-arm `SyncAction` table maps
  cleanly onto `sync`'s behavior with no case needing a separate verb.
- **D-2 (fetch always, even under `--dry-run`):** implemented as specified — `worktree::fetch` runs
  unconditionally, before the dry-run check; only `merge_ff_only`/`push` are gated.
- **D-3 (dirty-worktree handling):** implemented the recommended path — a `FastForward` pull first
  checks `Repo::is_clean()` and refuses with "commit your changes first (`odm store commit`), then
  retry" rather than letting `git merge --ff-only`'s own error surface. A `LocalAhead` push
  performs no such check (a dirty worktree doesn't matter for push — it only sends committed work).
  Not separately fixtured (the slice-doc flagged this as CC's design call, not a required ledger
  row) — worth a fixture in a future hardening pass if the dirty-pull path proves to matter in
  practice.

## Bubble-up → `../arc-plan.md`

- **Slice 03 done**, delivering ledger row **SL-3**: `odm store sync` pushes and pulls the
  orphan-branch store to/from its remote, ff-only pull, plain push, divergence stops with zero
  mutation. The lifecycle now reads `init → mutate → status → commit → sync` — **all four verbs
  native**. **SL-4 (the composition row: "no raw git for the normal lifecycle") is now testable.**
- **What implementing it revealed that the arc-plan didn't need to anticipate further:** the
  `merge_ff_only` defect. It was invisible until now because nothing had ever fixture-tested
  `init::sync()`'s FastForward arm against a real remote — the ancestry *classification* was
  well-tested, the actual *git operation* it triggers was not. Worth naming for future arc-plan
  readers: a decision table with exhaustive unit tests on its pure logic can still hide an
  unexercised side effect in one of its action arms; the fixture that finally drives a real git
  operation through a long-dormant code path is often where the actual bug surfaces, arc scopes
  later than the code that path.
- **Silent-drop check:** all 8 ledger rows closed done, none deferred or no-op. No rows dropped.
  One deviation flagged rather than silently absorbed: a second `odm-store` change
  (`merge_ff_only`'s signature) beyond the cc-prompt's anticipated sole addition.
- **This was the arc's last remaining slice.** With s01/s02/s03/s04/s05 all done, the Store
  Lifecycle arc's slice breakdown is complete; SL-4 (composition) and SL-5 (no model drift, arc
  scale) remain open as arc-scale rows for the arc's own closing pass.
