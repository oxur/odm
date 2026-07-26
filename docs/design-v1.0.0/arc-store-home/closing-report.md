# Arc closing report — Store Home & `init` (`arc-store-home`)

> **Realizes:** ODD-0022 · **Slices:** 01–04, all closed · **Date:** 2026-07-26
> **Written by:** CC · **Status:** **code-complete and compose-verified**, with **SH-6 (dogfood
> cutover) explicitly open, pending RH C-5**.

## What the arc set out to do

Give odm's store a **dedicated home** — a git worktree holding a single orphan branch, separate
from the code branches — so the planning database has its own lineage, can be checked out
alongside the code without branch-switching, and cannot be moved by a code merge. One branch,
one database, one history; nothing to synchronise because there is nothing duplicated.

## What shipped

| Slice | Delivered | Row |
|-------|-----------|-----|
| **01** | Two-config split + store resolution. `odm.toml` becomes a **locator** (`[store]`); operational settings move to a **`config.toml` inside the store**. Both absences are the status quo, so no existing repo changed behaviour. | SH-1 attested |
| **02** | `odm store init` **bootstrap**: worktree + orphan branch + locator + scaffold + gitignore. The `git` shell-out, confined to one module (ODD-0022 §5). | SH-2 attested *(amended)* |
| **03** | `init` **attach** (check out an existing branch — never re-orphan, never re-scaffold) and **ff-sync** (fast-forward only; warn and stop on divergence). | SH-3 **reproduced** |
| **04** | `odm store rename`: move the worktree and/or rename the branch, rewriting the locator from **observed** git state so the corpus is never lost. | SH-4 attested |

**SH-5, the composition, is reproduced:**

```
1. bootstrap    ✓ /A/.worktrees/odm on orphan branch "odm" — the store is live
2. attach       ✓ attached /B/.worktrees/odm to the existing branch "odm"
3. ff-sync      ✓ fast-forwarded "odm" to origin/odm — the store is fresh
4. rename       ✓ …/odm → …/planning, branch "odm" → "planning"
5. check        ✓ ok (2 node(s), no problems)
```

## The three bugs the arc found, and what they have in common

Each was a case where **the code path could not be exercised in the environment that wrote it**,
or where **success and failure looked identical**.

1. **The modern-git argv was wrong** (slice 02). `worktree add --orphan <branch> <dir>` needs
   `-b`; without it git rejects the command. The implementing machine ran git 2.39, which only
   ever takes the *fallback* path, so the suite was green on an arm that had never executed.
   Found by CDC on git 2.43. **Fix included a CI guard** that now asserts the git version and
   runs both arms, because the missing guard is why it hid.
2. **The old-git fallback was not equivalent to the modern path** (slice 02). `checkout --orphan`
   keeps the working tree, so the store was born holding the code branch's entire checkout.
   Found by *running* it; the test asserted what the store contained but not what it did **not**.
3. **A rename could leave a green `check` over an invisible corpus** (slice 04). With the
   worktree moved and the branch rename failed, the locator still named the old path; resolution
   self-healed an empty store there and reported `ok (0 node(s), no problems)`.

The through-line: **assert both halves.** Not just "the store is here" but "and nothing is
there"; not just "git moved it" but "and odm can still find it". Every one of these passed a
test that checked only the positive half.

## Decisions recorded along the way

- **§5 widened twice, deliberately** — from *create* (02) to *create + attach + ff-sync* (03) to
  *+ rename* (04). All of it lives in one module, and the invariant never moved: **setup-time
  only; every steady-state read and write stays on `gix`.** Checked mechanically each slice.
- **Resolution never fails** (01). A malformed `[store]` falls back to the un-redirected default
  rather than making the corpus unreachable; strict parsing and its error live in
  `StoreConfig::load`.
- **A dry-run sync fetches** (03). Without it the preview compared against a stale upstream and
  reported the wrong arm. Routed to CDC as a deviation and **endorsed**.
- **Divergence is never auto-resolved** (03), and **a published branch rename is local-only**
  (04). Both are the same principle: odm does not rewrite history other clones already hold.
- **Repair is detected, not performed** (03, L-13 deferred). Re-creating a worktree someone
  removed on purpose is the kind of help that loses work.
- **`.odm/context.json` follows the store** (02); **`ROLLUP.md` stays at the invocation root**
  (02) — a projection out of the store, for a code-branch reader.

## Verification

| Check | Result |
|-------|--------|
| `cargo test --all-features --workspace` | **57 binaries, 0 failed** |
| `cargo clippy --all-targets --workspace -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none added across the arc |
| odm's own corpus | `check` green at 60 nodes, **unchanged** — the arc is opt-in until SH-6 |

CI runs the store suite on **both git arms** (a `>= 2.42` gate on `test`, plus a `test-old-git`
job), which is the standing guard against the two arms diverging again.

## SH-6 — open, and why

**odm does not yet live in its own store.** Its corpus is still `nodes/` on the working branch,
resolved by the no-`[store]` back-compat path. The cutover — moving odm's own nodes onto the
orphan branch and splitting `odm.toml` into locator + `config.toml` — is **RH C-5's** work, and
it is sequenced there deliberately: C-5 already rewrites `migrate`/`self-host`, and F-20 (the
creation-date data loss) has to be fixed in the same pass, so the corpus is rewritten **once**
rather than twice.

Recorded, not dropped: **SH-6 stays open on this arc's ledger** until that lands.

## Bubble-up

- **`arc-plan.md`** v1.5: SH-1…SH-4 closed, SH-5 reproduced, SH-6 open pending RH C-5.
- **`project-plan.md`** (P-14): the arc is code-complete; the dogfood cutover rides RH C-5.
- Carried for whoever picks up the `store` group: `--yes` is accepted and unused on every
  subcommand; `sync` assumes the upstream remote is `origin`; `--json`'s `worktree` duplicates
  `store_root`; `worktree_base` is not renameable.
