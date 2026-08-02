# Slice 01 (Store Lifecycle) — CDC verification: `store commit`

> **Arc:** Store Lifecycle · **Slice:** 01 · **Verifier:** CDC (independent) · **Date:** 2026-08-02 ·
> **Method:** LEDGER-DISCIPLINE v2.0 §A. **Fixture slice — no live class-(b) row.** Code + fixtures
> reproduced by direct read on `release/1.0.x` (working tree; CC left it uncommitted); runtime
> (`cargo`/`clippy`/`llvm-cov`) attested-by-CC → CI. `odm` store HEAD unchanged at `e06fffe` — no live
> store mutation.

## Verdict

**PASS — CDC-verified.** `odm store commit` persists the store worktree's pending node changes on the
orphan branch, with an odm-aware auto-summary message, idempotent no-op, `-m`/`--dry-run`/`--json` — all
reproduced in code with a thorough, non-vacuous fixture set. Git plumbing stays in `odm-store`; the reusable
`delta` module lands cleanly for s02. **CC found and fixed a real ODD-0022 defect in passing** (§3) — the
kind of catch the class-(a) real-orphan-branch fixture discipline exists to force. Clean slice.

## 1. The command (F-1…F-6) — reproduced

`store_cmd::commit` (`store_cmd.rs:453`): resolves `StoreHome`; **refuses** (typed bail, "the store is the
repo root — run `odm store init` first") when there is no `[store]`; opens the store `Repo`; computes the
node `delta`; then —

- **clean worktree** (`repo.is_clean()`) → "nothing to commit", **no empty commit**, exit 0 (json or plain).
- **message** = `-m` override, else `auto_message(&delta)` (the node delta in odm terms).
- **`--dry-run`** → renders the branch + message, writes nothing.
- **`--json`** → `{committed, sha, branch, message, delta}`.
- else `repo.commit_all(&message)` → the sha on the orphan branch.

`StoreCommand::Commit { message, dry_run, json }` + dispatch (`lib.rs:275/838`). Every row is covered by a
dedicated, non-vacuous test (§4).

## 2. Plumbing — reproduced, reusable

- `Repo::tree_delta(dir)` (`git.rs:148`) diffs HEAD against the worktree over one subtree **without touching
  git's index** — a read-only delta, right for a status/commit-preview primitive.
- `odm_store::delta` (new module): `NodeDelta { created, modified, removed, by_type: BTreeMap<type,
  TypeCounts> }` + `is_empty()` + an odm-terms formatter. Classified by node `type`, and — as the slice-doc
  asked — **reusable as-is for s02 (`store status`)**, not built into commit.

## 3. Finding CC caught + fixed (verified sound) — the `.odm/` cache landmine

`commit_all` builds its tree by walking the filesystem directly (not via git's index), so it never consulted
the worktree's own `.gitignore`. Under ODD-0022 the store's `.gitignore` marks odm's **derived caches**
(`.odm/index`, `.odm/drift`) expendable — so the first `store commit` after any `orient`/`validate` would
have **baked those caches permanently into the orphan branch**, a spec violation with no rewrite-history
escape. CC made `write_tree` **gix-exclude-aware** (`git.rs:260` — builds an ignore stack once per commit,
skips excluded paths in the recursion) and proved it with `commit_all_honours_the_worktrees_own_gitignore`:
a fixture with `/cache/*` + a `!/cache/keep.txt` negation asserts `tracked.md` **and** the re-admitted
`cache/keep.txt` are committed while `derived.bin` is **not** — so exclusion *and* negation both hold.
Reproduced and sound; a genuinely valuable catch, honestly disclosed.

## 4. Tests — reproduced, non-vacuous

`tests/store_commit.rs` (11): `commit_persists_a_new_node_on_the_orphan_branch_and_leaves_the_code_branch_alone`,
`default_message_reads_the_node_delta_in_odm_terms`, `dash_m_overrides_the_default_message`,
`a_second_commit_on_a_clean_worktree_is_a_no_op` (+ plain-text), `dry_run_reports_the_delta_and_writes_no_commit`
(+ plain-text), `json_shape_is_correct_on_a_real_run` / `…_no_op_run`, `a_later_commit_captures_modifications_and_removals_by_type`,
`without_a_store_section_commit_refuses_rather_than_touching_the_repo_root`. `tests/edge_cases.rs`:
`commit_all_honours_the_worktrees_own_gitignore` (§3), `commit_skips_empty_subdirectories`,
`atomic_write_rename_failure_cleans_temp`. One-to-one with the ledger; the orphan-branch + code-branch-intact
assertions cover the ODD-0022 discipline (F-6). Clippy/coverage/`unsafe`-free → CI.

## 5. Ledger — CDC disposition

F-1…F-6 reproduced in code + non-vacuous fixtures (structural / attested→CI on execution). F-7 (no model
drift) — CLI + `odm-store` git plumbing only; no node-schema change; ODD-0022 unamended (the fix *enforces*
its existing `.gitignore` intent, no new line needed). F-8 attested→CI. **8 rows, no silent drops.** The
flagged decisions are dispositioned in the closing-report (auto-summary = git-delta node counts per D-1;
staged set = the worktree minus gitignored derived caches; author = ambient git).

## 6. Bubble-up (PM Part IV)

- **Did s01 deliver?** Yes — `store commit` is native, idempotent, dry-run/json-capable, orphan-branch-safe;
  the freeze-commit step is no longer raw git. `delta` is ready for `store status`.
- **Arc-plan change:** SL-1 → **done (CDC-verified PASS)**. **The Store Lifecycle arc stays ⏸ PAUSED** per
  the scoped-run note (s02 `store status` / s03 `store sync` not started). **Next: resume Migration
  Fidelity at s16** and close it.

## Closure

s01 **CDC-verified PASS** on 2026-08-02. `store commit` reproduced with a thorough fixture set; the
`.odm/`-cache `.gitignore` defect caught + fixed + regression-tested; `odm@e06fffe` untouched. SL-1 done; the
arc pauses; Migration Fidelity/s16 resumes next.

_Verified by: CDC (independent), 2026-08-02 — against `release/1.0.x` (working tree); `odm@e06fffe`
unchanged. Code/fixtures reproduced by direct read; execution rows attested-by-CC → CI._
