---
id: 01KYP5G5MSYTA5QRAFRCEQRKS2
number: 521125300
type: artifact
schema: artifact/v1.1
name: Slice 03 (arc-store-home) — CDC verification
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-home/slice03-init-attach-sync/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SAX5GZJ53WFTVPBWF
---
# Slice 03 (arc-store-home) — CDC verification

> **Verifies:** SH-3 · **Slice:** `init` attach + ff-sync · **Branch:** `sh-slice03-init-attach-sync`
> (`5620f0c`, off slice-02 tip) · **Date:** 2026-07-26 · **Verifier:** CDC, independent of CC — clean
> container clone (git bundle) on **git 2.43.0**, cargo 1.95. The slice-03 git ops (attach `worktree
> add`, `fetch`, `merge --ff-only`) are **not** version-gated the way bootstrap's `--orphan` was, so a
> single modern-git run reproduces them fully.

## Verdict

**Slice 03 delivered; SH-3 attested → reproduced (CDC, git 2.43).** The three-way `init` is complete
and the safety invariant holds under independent reproduction. No defects. A clean contrast to slice
02 — this one survived the same independent-environment run that broke the last. The one decision CC
routed to CDC (the dry-run fetch) is **endorsed**. CI (both git-arm jobs) flips `reproduced` to the
durable state once the branch pushes.

## Reproduced by CDC (git 2.43)

- **Compose demo (SH-5) — green.** `store_attach_sync` (11) + `store_init` (8) = **19 integration
  tests pass**; `odm-store --lib` = **31 unit tests pass** (incl. the full `sync_action` table). The
  three-way walk reproduces: bootstrap stands the home up, attach checks out the existing branch,
  ff-sync fast-forwards, a second run reports up-to-date.
- **The decision table is a pure function, correctly mapped.** `sync_action(Option<Ancestry>)`:
  `None → NoUpstream`; `(local⊑up, up⊑local)` → `(t,t) UpToDate` / `(t,f) FastForward` / `(f,t)
  LocalAhead(n)` / `(f,f) Diverged`. All five unit-tested with no repo/remote/network; the integration
  tests then prove the *measurement* (`is_ancestor`, `count_commits`) against a local bare remote.
  Measurement and decision are cleanly separated.
- **Safety invariant — asserted on state, not messages.** `sync_stops_on_divergence_and_touches_nothing`
  checks three things after a diverged `init`: B's `HEAD` is unmoved, B's own node is still listed, and
  A's pushed node is **absent** — i.e. no rebase, no merge-behind-the-user's-back. This is the whole
  point of the arc, and it is verified against `git rev-parse` + `odm list`, not stderr text. Endorsed.
- **§5 boundary (L-15) holds.** Every `Command::new("git")` is in `worktree.rs`; the six new functions
  (`attach`, `fetch`, `merge_ff_only`, `rev_parse`, `is_ancestor`, `count_commits`) all live there.
  Steady-state node reads/writes touch no subprocess. The widening (create → create + attach + ff-sync)
  is `init`-time only, exactly as recorded.
- **Repair case correctly intercepted.** `needs_repair` is checked in `store_cmd.rs` **before** the
  `detect` dispatch, so the "branch exists, worktree gone" state (which slice-02 `detect` classifies as
  `ExistsLocally`) warns-and-returns rather than letting `sync` fast-forward a missing worktree. The
  message points at `git worktree add` / future `--force`. Deferred by design (L-13), not silently
  fixed — and not a latent crash.

## The dry-run fetch deviation — **endorsed**

CC deviated from the prompt's "`--dry-run` touches nothing": a dry-run of the **sync** arm performs a
`fetch`. I agree with the call, and the way it was built removes my two reservations:

1. **The store is provably untouched.** `fetch` moves remote-tracking refs only — no branch move, no
   worktree write. `dry_run_touches_nothing_on_the_sync_arm` passes, so this is test-backed, not
   asserted.
2. **The user is told.** The dry-run FastForward message says *"upstream was fetched so this preview
   matches the real run; remote-tracking refs only"* — the network consultation is disclosed, not
   hidden.
3. **Offline degrades gracefully.** The fetch result is discarded (`let _ =`), so no network → no hard
   error; the flow falls through to whatever ref state exists (`NoUpstream`, or a stale comparison),
   which the table already handles. A dry-run never *requires* the network.

The reasoning is correct: a sync preview that doesn't fetch is a preview of a *different, stale*
operation, and a preview that can disagree with the run it previews is worse than none. The store's
safety (the thing "touch nothing" is really protecting) is intact.

**One nice-to-have, non-blocking:** offline, the "this preview matches the real run" line is slightly
overconfident, because the discarded fetch failure is invisible to the message. A future refinement
could surface "couldn't reach upstream — comparison may be stale" when the fetch fails. Minor; the
online common case (the one that matters) is exactly right.

## Raised-for-later (CC's bubble-up) — all legitimate, non-blocking

1. **`--yes` accepted but unused on every arm.** Correct: no arm prompts (all are non-destructive —
   create / checkout / ff-or-stop), so there is nothing to bypass. It becomes meaningful only when
   destructive `--force`/`--overwrite` arrive (§6, deferred). Keep for surface consistency; note it.
2. **sync assumes the upstream remote is `origin`** (`DEFAULT_REMOTE`). A reasonable 1.0 simplification;
   a store tracking a non-`origin` remote is the edge case. **Record as a known limitation** — a future
   refinement reads the branch's configured `@{upstream}` instead of hardcoding. Not blocking.
3. **`--json`'s `worktree` duplicates `store_root`.** Cosmetic schema wart (two keys, one value; also
   noted at slice 02). Worth a small cleanup — drop one or make `worktree` the parent dir — before the
   `--json` contract is depended on. Non-blocking.

## Exit-code note (observed, acceptable)

Diverged / no-upstream exit **success** (warn + stop), not non-zero — `init` isn't *failing*, it's
declining to auto-resolve. Scripts detect these via the `--json` `mode` (`sync-diverged` /
`sync-no-upstream`), which is the right machine signal. Deliberate and fine; recorded so it isn't
mistaken for a missing error path.

## Ledger

- **SH-3 → `attested` (CC) → `reproduced` (CDC, git 2.43)** for every arm (none are `--orphan`-gated,
  so 2.43 exercises them all). Durable `reproduced` on CI once the branch pushes and **both** git-arm
  jobs (the slice-02 matrix) run this suite green.
- **Silent-drop diff:** none. L-13 (repair) deferred by design; the three raised items are recorded
  above with dispositions.
- **SH-5 (compose) is now reproducible** — the three-mode walk above *is* it.
- **Next:** slice 04 (`rename`) — the last slice, slottable; then RH C-5 re-self-hosts into the home
  (the cutover, SH-6).
