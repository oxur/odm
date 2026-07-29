---
id: 01KYP5G40Y9J4TT4GP4FRPZ4QZ
number: 531812600
type: artifact
schema: artifact/v1.1
name: Arc closing report — Store Home & `init` (`arc-store-home`)
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-home/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SHYYMCPX4JN86ZJ6F
---
# Arc closing report — Store Home & `init` (`arc-store-home`)

> **Realizes:** ODD-0022 · **Slices:** 01–04, all closed · **Date:** 2026-07-26
> **Written by:** CC · **Status:** **complete.** Code-complete and compose-verified at first
> writing, with SH-6 open pending RH C-5; **SH-6 landed the same day** — see the postscript.

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
| odm's own corpus | `check` green at 60 nodes — **now in the store home**, all 60 ULIDs preserved (SH-6) |

CI runs the store suite on **both git arms** (a `>= 2.42` gate on `test`, plus a `test-old-git`
job), which is the standing guard against the two arms diverging again.

## SH-6 — closed (postscript, same day)

At first writing this section read *"odm does not yet live in its own store"* — the corpus was
still `nodes/` on the working branch, resolved by the no-`[store]` back-compat path, and the
cutover was sequenced into **RH C-5** so the corpus would be rewritten once rather than twice.

**It landed.** odm's 60 nodes now live on the orphan `odm` branch at `.worktrees/odm`; `check` is
green at 60 *in the home*; `odm.toml` is a locator and the operational half moved to the store's
`config.toml`. Every ULID was preserved — a relocation and in-place re-stamp, not a re-derivation,
which would have minted ids and broken every edge. Full account:
`arc-release-hardening/c5-closing-report.md`.

### What the dogfood found — and why it belongs in *this* report

The arc shipped four slices without odm ever running on its own store. The moment it did, three
defects appeared, and every one of them is a fourth instance of the pattern this report already
named:

5. **`orient` was blind to a selection `use` had just written.** Slice 01 moved the CLI context
   under the store root "as the `.odm/` index already does" and updated `use` and `context` — the
   two obvious consumers. `orient` is the third, and it kept reading the *invocation* root. That is
   indistinguishable in any repository where the store is the invocation root, which was every
   repository and every test this arc wrote. The symptom, once the two parted: `use` prints
   `✓ context: arc = …`, writes the file, and `orient` reports `(no current arc)`.
6. **`init` scaffolded no `.gitignore` for the store**, so the first `git add -A` on a store branch
   sweeps in the derived index — a conflict generator on a branch whose entire purpose is sharing.
7. **The first fix for (6) was ignoring all of `.odm/`** — which would have withheld
   `context.json`, the current focus, from every fresh clone. The store would carry the plan but
   not the place in it, defeating the project's own success test that a fresh session orients from
   `odm orient` alone.

The through-line the report drew from the first three — **assert both halves** — extends cleanly.
(5) is *"not just that `use` wrote it, but that `orient` can read it"*; (7) is *"not just that the
cache is ignored, but that the declaration is not"*. The regression tests for (5) run against a
**redirected** store on purpose: in an un-redirected one they pass either way, which is precisely
how the bug survived four slices of green suites.

The honest lesson for the arc: **an opt-in feature nobody has opted into is not verified.** Slices
01–04 were green, and the composition (SH-5) was reproduced against synthetic repositories — but
until odm itself moved in, the arm where the store root and the invocation root differ had never
been exercised by a real command against a real corpus. That is the same shape as the git-2.39
bug in slice 02, one level up.

## Bubble-up

- **`arc-plan.md`** v1.6: SH-1…SH-4 closed, SH-5 reproduced, **SH-6 attested** (RH C-5 cutover).
- **`project-plan.md`** (P-14): the arc is complete, dogfood included.
- Carried for whoever picks up the `store` group: `--yes` is accepted and unused on every
  subcommand; `sync` assumes the upstream remote is `origin`; `--json`'s `worktree` duplicates
  `store_root`; `worktree_base` is not renameable.
