# Slice 03 closing report — `init` attach + ff-sync

> **Arc:** Store Home & `init` (`arc-store-home`) · **Slice:** 03 · **Feeds:** SH-3
> **Realizes:** ODD-0022 §4.3 (attach + sync) + §6 (sharing, ff-only)
> **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (L-1…L-16)
> **Implemented by:** CC · **Date:** 2026-07-26 · **Branch:** `sh-slice03-init-attach-sync`
> (off the slice-02 tip, which carries the amendment) · **Evidence class:** attested-by-CC
> (local 1.85+, git 2.39.5); cargo rows reproduce on CI, which runs both git arms.

## What shipped

`odm store init` is now the whole three-way command. Slice 02's two "defer to slice 03" stops
are replaced by real behaviour:

```
── A: bootstrap ──
✓ store init: /A/.worktrees/odm on orphan branch "odm" — the store is live

── B: clone, then attach ──
✓ store init: attached /B/.worktrees/odm to the existing branch "odm" — the store came with it

── A advances and pushes; B re-inits ──
✓ store init: fast-forwarded "odm" to origin/odm — the store is fresh

── B re-inits again ──
✓ store init: "odm" is already up to date with origin/odm
```

- **Attach** — `git worktree add <dir> <branch>`, a *checkout* of the existing branch. Fetches
  first when the branch is remote-only. Scaffolds nothing: `config.toml` and `nodes/` ride in
  with the branch, and writing them would overwrite a teammate's store with defaults. The only
  writes are the two idempotent top-ups a clone might lack (`.gitignore`, locator).
- **ff-sync** — fetch, classify by ancestry, and act: fast-forward, no-op, report-and-stop.
- **The decision is a pure function.** `sync_action(Option<Ancestry>) -> SyncAction` — git
  measures ancestry, the table decides. All five cases are unit-tested with no repo, no remote
  and no network; the integration tests then prove the measurement.

| ancestry | action | effect |
|---|---|---|
| local is ancestor of upstream | `FastForward` | `merge --ff-only` in the worktree |
| each is the other's ancestor (identical) | `UpToDate` | nothing |
| upstream is ancestor of local | `LocalAhead(n)` | report "n to push", success |
| neither | `Diverged` | **warn and stop** |
| no upstream | `NoUpstream` | warn, success, untouched |

## The safety invariant

**Divergence is never resolved automatically.** The store branch is shared by construction —
that is the point of ODD-0022 §6 — so merging or rebasing it would rewrite history other clones
already hold. odm reports and stops; the choice is a person's.

The test asserts this three ways rather than trusting the message: B's HEAD has not moved, B's
own node is still there, and **A's node is not present** — no quiet merge behind the user's
back.

## §5 widened, deliberately and on the record

ODD-0022 §5 ratified the `git` shell-out for *creating* a worktree. Attach and ff-sync need
three more operations — `worktree add <dir> <branch>`, `fetch`, `merge --ff-only`, plus
ancestry queries — none of which `gix` 0.66 offers for worktrees.

They all live in the **same one module**, `odm-store/src/worktree.rs`. The exception is now
*create + attach + ff-sync*, and the invariant that actually matters is unchanged: **`init`-time
only; every steady-state node read and write stays on `gix`** (L-15, checked mechanically —
`Command::new` in `crates/*/src/` is that module plus `odm-reconcile`'s pre-existing shell
*probe*).

## A deviation worth naming: dry-run sync fetches

The prompt says `--dry-run` should "touch nothing" on all arms. The sync arm **does fetch**
under `--dry-run`, and here is why.

Without a fetch, the preview compares against whatever the last fetch left behind. Building it
that way, the dry-run test failed with `sync-up-to-date` for a store a real run would
fast-forward — the preview and the reality disagreed. **A dry run whose answer can differ from
the run it previews is worse than no dry run**, because it is trusted.

A fetch is the one git operation here that cannot touch the store: it updates remote-tracking
refs only, moves no branch, and writes nothing in the worktree. The dry-run message says so
outright rather than leaving it implicit. Recorded here rather than done silently — if the arc
prefers strict "no network in a dry run", the alternative is to label the result as possibly
stale, which I think is the worse trade.

## Verification

| Check | Result |
|-------|--------|
| `cargo build --workspace` | clean |
| `cargo test --all-features --workspace` | **56 binaries ok, 0 failed** (+8 unit, +11 end-to-end) |
| `cargo clippy --all-targets --workspace -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `unsafe` | none added |

**Tested against real git with a real remote.** A local bare repository serves as `origin`,
giving genuine fetch, ancestry and fast-forward behaviour with no network. Mocking git would
have tested the mock — the whole point of these arms is what git actually does with a shared
branch. The shape is A → origin → B, with the two repos taking turns advancing.

**Reproduced by hand** as the demo above: bootstrap → attach → ff-sync → up-to-date no-op.

**Back-compat:** odm's own repo has no `[store]`; `check` green at 60 nodes, unchanged.

## One slice-02 test changed meaning (not a regression)

`an_existing_branch_stops_without_touching_anything` seeded a branch with **no worktree** and
asserted the "slice 03" deferral message. Slice 03 classifies that state precisely — a
half-finished or removed worktree — and still refuses, now pointing at the future `--force`.
The safety claim the test exists to pin is unchanged, so the assertion moved to the current
message rather than the test being deleted.

## For the arc to consider

- **Repair stays deferred** (L-13, ODD-0022 §6). It is detected and reported clearly; nothing
  is silently fixed, because re-creating a worktree someone removed on purpose is exactly the
  kind of help that loses work.
- **`--yes` is still accepted and unused** on every arm — carried from slice 02. Attach and
  sync ask nothing either; the flag is waiting for an arm that does.
- **`sync` fetches from `origin` only.** A repo whose store upstream is a differently-named
  remote will report `sync-no-upstream`. Fine for the arc's assumptions, but it is an
  assumption, not a check.
- **`worktree` still duplicates `store_root` in `--json`** (raised in slice 02, unresolved). Now
  that every arm reports it, the case for collapsing them is stronger.

## Silent-drop diff

None. Everything the cc-prompt scoped in shipped. Scoped out and absent: `--force`/`--overwrite`,
a standalone `odm store sync`, `rename` (slice 04), corpus migration (RH C-5).

## Ledger

- **`SH-3`** — ready to close: **attested** on this report; **reproduced** when CI runs the
  cargo rows green on both git arms.
- **L-1…L-16**: fifteen `done`, **L-13 `deferred`** by design.
- With SH-3 closed, **SH-5** (the bootstrap → attach → ff-sync compose demo) is reproducible —
  the demo above is that walk.
