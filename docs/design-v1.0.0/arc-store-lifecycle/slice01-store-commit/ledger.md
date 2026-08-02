# Slice 01 (Store Lifecycle): `store commit`

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence strength `asserted < attested < reproduced <
> reconciled`; a `done` row reaches ≥ `reproduced`. Code/fixtures class-(a): CDC reproduces by direct read;
> runtime (`cargo`/`clippy`/a live commit) attested→CI / operator. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **`store commit` exists + persists**: `odm store commit` stages the worktree's pending node changes + commits on the **orphan branch** (not the code branch) | fixture: mutate a node → `store commit` → orphan branch has +1 commit containing the change | serious (the gap) | operator | done | attested — `crates/odm-cli/tests/store_commit.rs::commit_persists_a_new_node_on_the_orphan_branch_and_leaves_the_code_branch_alone` (real orphan-branch worktree fixture, `cargo test -p odm-cli --test store_commit`, 11/11 pass); manual smoke test against the real `odm` binary confirmed the same. | `Repo::commit_all` (`crates/odm-store/src/git.rs`, already used by `init`) reused as-is; the new entry point is `odm_store::delta::compute` for the node delta. |
| F-2 | **Auto-summary message + `-m` override**: default message = node delta (created/modified/removed, by type); `-m` overrides | run with no `-m` → message reads the delta in odm terms; `-m "x"` → "x" | serious (odm-aware, not porcelain) | slice-doc D-1 | done | attested — `store_commit.rs::default_message_reads_the_node_delta_in_odm_terms` (`"store: +1 slice"`), `::dash_m_overrides_the_default_message`. | **D-1 taken as recommended:** git-delta node counts in v1 (`odm_store::delta::NodeDelta::summary`); the migrate-aware ("reconciled/minted/collapsed") summary is flagged, not built — see slice-doc D-1 and closing-report. |
| F-3 | **Idempotent no-op when clean**: a clean worktree → exit 0, "nothing to commit", **no empty commit** | run twice: second run makes no commit, exits 0 | serious | slice-doc | done | attested — `store_commit.rs::a_second_commit_on_a_clean_worktree_is_a_no_op` (asserts `commit_count` unchanged) + `::a_clean_no_op_reports_in_plain_text_too`. | Never errors on clean; never an empty commit — checked via `Repo::is_clean` before any commit attempt. |
| F-4 | **`--dry-run`**: renders the delta + message, writes no commit | dry-run on a dirty store → reports, orphan branch HEAD unchanged | serious | house contract | done | attested — `store_commit.rs::dry_run_reports_the_delta_and_writes_no_commit` (asserts `HEAD` stays unborn) + `::dry_run_reports_in_plain_text_too`. | |
| F-5 | **`--json`**: `{committed, sha, branch, message, delta:{created,modified,removed,by_type}}` | `--json` parses + fields correct on a real + a no-op run | serious (LLM ergonomics) | ODD-0023 | done | attested — `store_commit.rs::json_shape_is_correct_on_a_real_run` + `::json_shape_is_correct_on_a_no_op_run` (every field asserted, including `by_type` per node type). | `sha`/`committed` are always present (`sha: null` rather than omitted) on a no-op/dry-run, since the ledger's shape names `sha` unconditionally. |
| F-6 | **Store discipline honored**: commit lands on the orphan branch in the worktree; the code branch is untouched; no history rewrite | direct read: the code checkout has no new commit; the orphan branch does; existing history intact | serious (ODD-0022) | ODD-0022 | done | attested — F-1's fixture reads `commit_count` on *both* worktrees (same repo, different checked-out branch) and asserts the code branch stays at 1; real-binary smoke test cross-checked with `git log --oneline` on both. | **Flag (staged set — see closing-report):** confirmed `commit_all` stages node files + `config.toml` + `.gitignore` — **and found + fixed** that it was *also* staging the gitignored `.odm/index`/`.odm/drift` caches, since `commit_all` walks the filesystem rather than git's index and so never consulted the store's own `.gitignore`. Fixed in `git.rs::write_tree` (now exclude-aware); regression fixture `crates/odm-store/tests/edge_cases.rs::commit_all_honours_the_worktrees_own_gitignore`; real-binary smoke test confirmed `.odm/index`/`.odm/drift` no longer land in `git show --stat`. **Author identity: odm-stamped** (`odm <odm@localhost>`, from `git.rs`'s existing `ensure_identity`/`SignatureRef`), not ambient git config — confirmed via `git show`'s `Author:` line in the smoke test. |
| F-7 | **No model drift**: CLI + `odm-store` git plumbing only; no node-schema change; ODD-0022 amended only if a line is genuinely needed | cross-read: diff scope is CLI + store; ODD cited if touched | correctness | ODD-0022 | done | attested — diff scope: `odm-store/src/{git.rs,delta.rs (new),lib.rs}` + `odm-cli/src/{lib.rs,store_cmd.rs}` + fixtures only; no `odm-core` schema touched; ODD-0022 not amended. | The F-6 `.gitignore` fix is scoped to `write_tree` (git plumbing) and *brings behavior into line with* ODD-0022/the store's own `.gitignore`, rather than changing the model — judged not to need an ODD-0022 amendment; flagged here for CDC to confirm that judgment. |
| F-8 | **Clippy clean; no `unsafe`; new code covered** | clippy `-D warnings` exit 0; `! grep unsafe`; fixtures cover commit / no-op / dry-run / json | polish | CLAUDE.md | done | attested — `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `grep -rn unsafe` on every changed/new file empty; `cargo fmt --check` clean; `make check` green; `cargo llvm-cov -p odm-store -p odm-cli -p oxur-odm` shows `delta.rs` 97–100%, `store_cmd.rs`'s `commit`/`auto_message`/`CommitJson` covered on both JSON and plain-text paths (remaining misses are an unreachable `Repo::open` error-context closure and llvm-cov's known closing-brace/paren line artifacts, matching the pattern already accepted elsewhere in `git.rs`). | |

## What Worked

- The class-(a) fixture style (a **real** orphan-branch worktree via `init::bootstrap`, not a hand-placed
  store) caught a real bug the first time it ran end-to-end as the real binary (F-6's `.odm/` leak) —
  exactly the payoff LEDGER-DISCIPLINE names for reproduced-not-attested evidence. A hand-placed-store
  fixture (the `store_home.rs` style) would never have exercised `.gitignore` at all.
- `odm-store`'s existing git plumbing (`Repo::commit_all`, `Repo::is_clean`, the `write_tree`/`write_blob`
  idioms) needed no new commit path — only a new read (`Repo::tree_delta`) built the same way (walk +
  compare, blobs written content-addressed) as what was already there. Low model drift in practice, not
  just by intent.
- Factoring `odm_store::delta` as its own module (rather than inlining delta computation into
  `store_cmd.rs`) paid off immediately: it is already reusable for s02 (`store status`) with zero rework,
  as the slice-doc asked.

## Closure

Closed 2026-08-02. Verified by: CC (this session) — attested; CDC reproduction pending. Rows: 8. Done: 8.
Deferred: 0. No-op: 0. On close, bubble up to `../arc-plan.md`: SL-1 done (`store commit` native, including
the F-6 `.gitignore`-exclusion fix); per the arc's scoped-run note, **s02/s03 stay paused** — the arc is not
closed here.
