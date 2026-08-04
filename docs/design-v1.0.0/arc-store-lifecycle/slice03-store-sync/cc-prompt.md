# CC Prompt — Slice 03 (Store Lifecycle): `store sync`

Add `odm store sync` — the verb that pushes and pulls the orphan-branch store
to/from its configured remote, honouring ODD-0022's discipline: ff-only pull,
normal push, divergence stops. The last gap in the store's git lifecycle. After
this, `migrate → store status → store commit → store sync` is fully odm-native.

> **Start condition:** on `release/1.0.x`, green. `store commit` (s01) and
> `store status` (s02) are both landed. The ancestry plumbing
> (`Ancestry`/`SyncAction`/`sync_action()`) is confirmed reusable without
> modification (s02's closing report). `init::sync()` already does fetch + ff
> for the pull direction; this slice lifts that into a standalone verb and adds
> push.

## Read first

1. `slice03-store-sync/ledger.md` (8 rows) + `slice-doc.md` (esp. **D-1**
   through **D-3**).
2. **ODD-0022** (the store model: orphan branch, odm owns it, never rewrite
   history other clones hold, divergence stops rather than merges).
3. **Code — the existing sync plumbing (reuse):**
   `crates/odm-store/src/init.rs` — `sync()` (lines 538+, the pull direction:
   fetch → ancestry → ff-merge), `Ancestry` (lines 165–178), `SyncAction`
   (lines 180–207), `sync_action()` (lines 218–229), `DEFAULT_REMOTE` (line
   496).
4. **Code — the worktree git ops (reuse + extend):**
   `crates/odm-store/src/worktree.rs` — `fetch()` (line 224), `merge_ff_only()`
   (line 236), `rev_parse()` (line 248), `is_ancestor()` (line 260),
   `count_commits()` (line 274). **There is no `push()` — you must add one.**
5. **Code — the newest sibling:**
   `crates/odm-cli/src/store_cmd.rs` — `status()` (lines 615+, the read-only
   handler that computes ancestry directly using `worktree::*` + `init::*`
   plumbing) and `StatusJson`/`UpstreamJson` (lines 530–551, the JSON shape
   pattern). Also the `init`-mode `sync()` handler (lines 212+, the text output
   pattern for all five `SyncAction` arms).

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first, then `03-error-handling`, `05-type-design`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **`worktree::push()`** — the one new `odm-store` function. Add it to
   `crates/odm-store/src/worktree.rs` following `fetch()`'s pattern:

   ```rust
   pub fn push(repo_root: &Path, remote: &str, branch: &str) -> Result<()> {
       run(repo_root, &["push".into(), remote.into(), branch.into()])
   }
   ```

   Never `--force`. The module's doc header (§5 scope-widening note) should be
   updated to name `push` alongside the other additions.

2. **The command** (F-1/F-2/F-3/F-4). `StoreCommand::Sync { dry_run: bool,
   json: bool }` in `lib.rs` + a `sync_cmd()` handler in `store_cmd.rs`. The
   handler orchestrates — it does **not** call `init::sync()` (which takes an
   `InitPlan` and is tied to `init`'s own output struct); instead it calls the
   same underlying plumbing directly, exactly the way `status` does:

   a. Resolve the store home (`StoreHome::resolve`), bail if no `[store]`.
   b. **Fetch** from `DEFAULT_REMOTE` — always, including under `--dry-run`
      (D-2). A fetch failure (no remote, no network) is not fatal: it leaves no
      upstream to compare, handled by the `NoUpstream` arm.
   c. **Compute ancestry** — `rev_parse` local and upstream, `is_ancestor` +
      `count_commits` → `Ancestry` → `sync_action()`. Include `behind` count
      (an independent `count_commits(local..upstream)`) alongside the
      `Ancestry`-driven classification, as `status` does.
   d. **Act on `SyncAction`:**
      - `FastForward` — pull: `worktree::merge_ff_only(&store_root)`, unless
        `--dry-run`.
      - `LocalAhead(_)` — push: `worktree::push(&repo_root, DEFAULT_REMOTE,
        &branch)`, unless `--dry-run`.
      - `UpToDate` — no-op.
      - `Diverged` — stop; report clearly; change nothing.
      - `NoUpstream` — report; not an error.

   **D-3 (dirty-worktree handling):** the recommendation is to check
   `Repo::is_clean()` before a `FastForward` pull and refuse with "commit your
   changes first — `odm store commit`, then retry" rather than letting `git
   merge --ff-only`'s error surface. For `LocalAhead` (push), a dirty worktree
   is fine — push only sends committed work. **Flag your choice in the ledger's
   Notes column** for F-1; don't silently adopt either path.

3. **`--json`** (F-6). Shape:

   ```json
   {
     "action": "pushed",
     "dry_run": false,
     "branch": "odm-store",
     "store_root": ".worktrees/odm",
     "upstream": {
       "ref": "origin/odm-store",
       "ahead": 0,
       "behind": 0
     }
   }
   ```

   `action` values: `"pushed"` | `"pulled"` | `"up-to-date"` | `"diverged"` |
   `"no-upstream"`. When `action` is `"no-upstream"`: `"upstream": null`.
   `ahead`/`behind` are the **post-sync** counts (both 0 after a successful
   push or pull; the pre-sync values for diverged/no-op). Under `--dry-run`,
   `ahead`/`behind` reflect what `fetch` showed (the preview values — these are
   accurate because `fetch` runs even under dry-run).

   Struct: `SyncJson { action, dry_run, branch, store_root, upstream:
   Option<SyncUpstreamJson> }`, `SyncUpstreamJson { ref, ahead, behind }`.
   Reuse `#[serde(rename = "ref")]` on a `reference` field (Rust keyword), as
   `StatusJson`'s `UpstreamJson` does.

4. **Plain text** (all arms). Follow the `init`-mode `sync()` handler's
   wording pattern (lines 212+): `term::success` for happy paths (`pushed`,
   `pulled`, `up-to-date`), `term::warning` for `diverged` (the one that
   requires human attention), `term::info` for dry-run variants that report
   what *would* happen. The divergence message should name the counts ("N
   ahead, M behind") and point at the store root, matching `init`'s sync
   arm — don't invent a different wording.

5. **Fixtures** (F-1…F-6). In `crates/odm-cli/tests/store_sync.rs` (new).
   The fixture pattern for push/pull/divergence needs a **bare-repo remote** —
   the same `TempDir` approach s02 used for the ahead/behind rows:

   - **Setup helper** (reusable across fixtures): `with_bare_upstream()`
     bootstraps a store (`store init`), creates a `TempDir` bare repo (`git
     init --bare`), `remote add origin <bare>`, pushes, fetches — yielding a
     store with a real remote-tracking branch.
   - **F-1 (pull):** on the bare remote, advance the branch by one commit
     (direct git in the bare repo, or push from a second worktree); in the
     test store, `store sync --json` → action `"pulled"`, local HEAD now
     matches remote HEAD.
   - **F-2 (push):** in the test store, `store commit` to create a local-ahead
     state; `store sync --json` → action `"pushed"`; verify the bare remote
     now has the commit (`rev_parse` against the bare repo).
   - **F-3 (divergence):** advance both sides independently → `store sync
     --json` → action `"diverged"`, local HEAD unchanged, remote HEAD
     unchanged.
   - **F-4 (no-op):** up-to-date → `"up-to-date"`; no remote → `"no-upstream"`.
   - **F-5 (dry-run):** upstream ahead → `--dry-run --json` → would pull, but
     local HEAD unchanged; local ahead → `--dry-run --json` → would push, but
     remote unchanged.

   *(For advancing the bare remote in pull/divergence fixtures: the simplest
   approach is to clone the bare into a second TempDir, commit there, push back
   — or use `git commit-tree` + `git update-ref` directly on the bare repo to
   mint a commit without a worktree.)*

## Constraints (flag, don't silently change)

- **Never force-push, merge, or rebase.** The push is `git push origin
  <branch>`; the pull is `git merge --ff-only`. If either would rewrite
  history, it fails, and that failure is the `Diverged` arm's job.
- **Git ops live in `odm-store`** (the `worktree` module), not the CLI. The
  CLI orchestrates.
- **`push()` is the sole new `odm-store` function.** Everything else the
  handler calls already exists and is already `pub`/exported. If you find
  yourself widening visibility for anything, flag it.
- Don't change `status` (s02) or `commit` (s01). Don't change `init::sync()`
  — the new handler orchestrates independently.
- **The `worktree.rs` module's doc header** names each scope-widening; update
  it to include `push` alongside `fetch` and `merge --ff-only`.
- No `unsafe`; typed errors; clippy `-D warnings` clean; cover the new code.

## Deliverables

The `store sync` command + fixtures on `release/1.0.x`; `worktree::push()` in
`odm-store`; `ledger.md` evidence per row; `closing-report.md` — the walk,
D-1/D-2/D-3 dispositions, the v2.0 bubble-up (SL-3 done; SL-4 testable).
Branch: `release/1.0.x` only.

## Working agreement

Amend don't work around; flag deviations; five-iteration cap. Your `done` is
proposed-done — CDC reproduces the command + fixtures + CI, and stages the musl
binary to run `odm store sync` live in the cloud container (the s05-established
workflow). On close, bubble up to `../arc-plan.md`.
