# Arc — Store Home & `init` (plan-of-record)

> **Named arc, canonical number deferred** (following the `arc-release-hardening` precedent —
> numbering is under review there; A7/A8 belong to the other CDC). **v1.0.x scope — expands the
> v1.0.0 DoD** (operator call, 2026-07-26). Realizes **ODD-0022** (Accepted). Directory:
> `arc-store-home/`.
>
> **Sequencing:** runs **before RH C-5**, which populates the new home. `depends_on:` A1–A3 (the
> store, config, and command surface). The dogfood cutover of odm's own corpus **rides RH C-5's
> re-self-host** (see §Dependencies).

## Capability

Give odm a **dedicated store home** — an orphan `odm` branch checked out in a `.worktrees/odm` git
worktree — and the **`init`** command that creates, attaches to, or syncs it, decoupling the
planning DB from the code branches (ODD-0022). Realizes the **two-config split** (`odm.toml`
locator + an in-store `config.toml`) and the **store-resolution rework**. On completion the store
no longer lives on the working branch; RH C-5's re-self-host writes the corpus into the new home.

## Slice breakdown (plan-late, plan-deep)

| Slice | Scope | Load-bearing for | Source |
|-------|-------|------------------|--------|
| **01 · Store resolution + two-config split** | `[store]` locator in `odm.toml`; operational `config.toml` loaded **from the store**; two-stage load; store root → `<repo>/<worktree_base>/<worktree_name>`; back-compat fallback to `repo-root/nodes` when `[store]` is absent. **No git-worktree ops** — testable against a hand-placed store dir. | **all** (foundational) | ODD-0022 §4.2 |
| **02 · `git`-worktree plumbing + `init` bootstrap** | The scoped git-shell wrapper (the ratified Q-2 exception: `git worktree add --orphan`, two-step fallback for git < 2.42) + **bootstrap** mode: create worktree + orphan branch, write the `[store]` locator, scaffold `config.toml` + empty `nodes/`, gitignore `/.worktrees/`. Detection: "no `odm` branch anywhere." | 03, 04, cutover | ODD-0022 §4.3, §5 |
| **03 · `init` attach + ff-sync** | **attach** (check out an existing local `odm` / `origin/odm` — fetch + tracking branch if remote-only; store rides *with* the branch, no re-scaffold) + **sync** (existing local store → `merge --ff-only`; divergence/no-upstream **warns + stops**; **no** auto-rebase/merge). Completes the three-way detection. | cutover | ODD-0022 §4.3, §6 |
| **04 · `odm rename` (store rename)** | Rename the worktree/branch; update `[store]` atomically. **Slottable** — can ship after `init` lands. | — | ODD-0022 §4.5 |

**Order:** 01 → 02 → 03; 04 slottable after 02. **Sizing:** each fits one context with iteration
headroom (01 is pure config/resolution; 02/03 are `init` split across the create-path vs the
already-exists paths; 04 is small).

## Dependencies & the C-5 cutover coupling

- **Consumes:** A1–A3 (store / config / CLI); **ODD-0022** (Accepted).
- **The dogfood cutover is RH C-5's job, not a 5th slice here.** Slice 01 makes the store root
  resolve to the worktree, so C-5's re-self-host (fold + names + dates + **one** re-stamp) writes
  **into the new home by default**. Migrating odm's own corpus onto the orphan branch is therefore
  C-5's re-self-host run *after* this arc's `init` exists — **no separate migration, no second
  corpus write** (the churn we designed away). **Sequence:** `arc-store-home` 01–04 → **C-5
  re-self-hosts into the freshly-`init`'d home (= the cutover)** → both arcs close. The old
  working-branch `nodes/` is retired (gitignored/removed) at that cutover.
- **gix:** worktree creation shells out to `git` (ODD-0022 §5 — ratified, `init`/rename setup only);
  **all steady-state reads/writes stay on `gix`**.
- **Git / base branch:** decide at slice 01 (off `release/1.0.x` or the latest RH tip).

## Arc Ledger

> Per LEDGER-DISCIPLINE v2.0 §B. Class-(a) slice-closed rows (SH-1…SH-4) accrue as slices close;
> class-(b) compose rows (SH-5/SH-6) reproduced at arc scale, never inherited; class-(c) bubble-up
> (SH-7). IDs are **stable** — new slices append, never renumber.

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| SH-1 | Slice 01 (resolution + two-config split) closed | ptr: `slice01…/closing-report.md` (CDC verify pending) | serious | arc-plan | **attested** | `slice01-store-resolution/closing-report.md` (2026-07-26): build/test (**54 binaries, 0 failed**, +17 new)/clippy `-D warnings`/fmt green; `[store]` resolves to `<repo>/<base>/<name>`, absent ⇒ repo root; operational config from the store's `config.toml` with locator fallback; a hand-placed `.worktrees/odm/` store takes every node write while **nothing lands at the repo root**; back-compat reproduced on odm's own corpus (no `[store]` → `check` green at 60 nodes, `nodes/` unmoved). L-7 mechanically clean: no `git` subprocess, no worktree ops. | attested-by-CC → **reproduced** when CI runs the cargo rows. Foundational — slices 02–04 and RH C-5 build on this. Raised for later slices: `.odm/context.json` still keys off the invocation root while the index follows the store; `ROLLUP.md` likewise; `branch_name` parsed but unconsumed until slice 02. |
| SH-2 | Slice 02 (git plumbing + `init` bootstrap) closed | ptr: `slice02…/closing-report.md` (CDC verify pending) | serious | arc-plan | **attested** | `slice02-init-bootstrap/closing-report.md` (2026-07-26): build/test (**55 binaries, 0 failed**, +17 new)/clippy `-D warnings`/fmt green. `odm store init` creates the worktree + **orphan** branch (proved by `git merge-base` *failing*), writes the `[store]` locator, scaffolds `config.toml` + empty `nodes/`, gitignores `/.worktrees/`; `odm new` + `check` then green in the home. L-14 holds: `Command::new` in src is `worktree.rs` ×2 + the pre-existing shell probe. | attested-by-CC **as amended**. Shipped at `6703394` with a **wrong modern-git argv** — CDC reproduced the failure on git 2.43 (6/8 integration tests) and verified the fix; corrected here, plus a CI guard that now *asserts* the git version (a `>= 2.42` gate on `test`, and a `test-old-git` job for the fallback arm). Three real bugs total: unborn-branch detection and fallback non-equivalence (found in-slice), and the modern argv (found by independent verification). **`reproduced` on CI once both git jobs run green.** |
| SH-3 | Slice 03 (`init` attach + ff-sync) closed | ptr: `slice03…/closing-report.md` (CDC verify pending) | serious | arc-plan | **attested** | `slice03-init-attach-sync/closing-report.md` (2026-07-26): build/test (**56 binaries, 0 failed**, +19 new)/clippy `-D warnings`/fmt green. Attach checks out the existing branch — `merge-base odm origin/odm` **succeeds** (shared history, the mirror of slice 02's orphan check) — and scaffolds nothing, with a custom `config.toml` proven to survive. ff-sync fast-forwards, no-ops when current, reports local-ahead, and **stops on divergence with B's HEAD unmoved and A's work not merged in**. Decision table is a pure function, all five cases unit-tested. Bootstrap → attach → ff-sync reproduced by hand. | attested-by-CC → **reproduced** on CI (both git arms). Tested against a **local bare remote**, so fetch/ancestry/fast-forward are real. **§5 widened on the record** from *create* to *create + attach + ff-sync* — all `init`-time, one module; steady state still all `gix` (L-15 checked mechanically). **Deviation recorded:** a dry-run sync *fetches*, because without it the preview reported the wrong arm. L-13 (repair) stays **deferred** by design. |
| SH-4 | Slice 04 (`rename`) closed | ptr: `slice04…/closing-report.md` (CDC verify pending) | serious | arc-plan | **attested** | `slice04-store-rename/closing-report.md` (2026-07-26): build/test (**57 binaries, 0 failed**, +24 new)/clippy `-D warnings`/fmt green. `odm store rename` moves the worktree, renames the local branch, and rewrites `[store]` from the **observed** git state; every test ends by asserting the corpus is still reachable. Collisions stop (git would not — `worktree move` moves *into* an occupied dir, `mv`-style), no-op is clean, uncommitted work survives, a published branch warns local-only. | attested-by-CC → **reproduced** on CI. **A real bug was found and fixed in-slice:** the first cut skipped the locator write when the branch rename failed, leaving a green `check` over an **invisible corpus** — the exact failure L-5 exists to prevent. Now the locator is written from observation *even on failure*, and the error names what succeeded. |
| SH-5 | **Compose:** `odm init` stands up a working home end-to-end — **bootstrap** on a fresh repo, **attach** on a clone (no fork), **ff-sync** freshens; divergence warns + stops | arc-scale demo: bootstrap → attach → ff-sync → rename, then `check` | serious | arc-plan | **reproduced** | Reproduced end-to-end 2026-07-26 against a local bare remote: **bootstrap** stands the home up on a fresh repo; **attach** checks it out in a clone; **ff-sync** fast-forwards when upstream advances; **rename** moves it — and `check` is green on 2 nodes after all four. Walk recorded in `slice04…/closing-report.md` and the arc closing report. | the arc's composition, reproduced by CC; CDC re-runs independently at arc close. |
| SH-6 | **Compose (dogfood):** odm's own corpus lives on the orphan `odm` branch, `check` green there, and the working branch no longer carries `nodes/` | arc-scale demo: `init` + **RH C-5** re-self-host into the home → `check` green | serious | arc-plan / ODD-0022 | **attested** | Done 2026-07-26 by the RH C-5 cutover — see `arc-release-hardening/c5-closing-report.md`. odm's 60 nodes live on the orphan `odm` branch at `.worktrees/odm`; `check` green at 60 **in the home**; the working branch no longer carries `nodes/`; `odm.toml` is locator-only and the operational half moved to the store's `config.toml`. **Every ULID preserved** (pre/post sets diff-empty) — a relocation and re-stamp, not a re-derivation, which would have minted ids, broken every edge and tripped G-1. | **reproduced jointly with RH C-5** (the cutover). The dogfood proof; the payoff row. **The cutover found three defects nothing before it could have:** `orient` read the invocation root for the CLI context while `use` wrote the store root (identical until odm's own store moved — `use` succeeded, `orient` reported no focus); `init` scaffolded no store `.gitignore`, offering the derived index for commit on a shared branch; and ignoring all of `.odm/` to fix that would have withheld the current focus from every clone. All fixed in-chunk with regression tests that run against a **redirected** store, since an un-redirected one cannot distinguish the two roots. |
| SH-7 | bubble-up findings dispositioned | ptr: arc-plan change-log | correctness | bubble-up | **done** | arc-plan v1.6 | slices 01–04 dispositioned as they closed; SH-5/SH-6 dispositioned at the RH C-5 cutover close (v1.6). |

Closes in `arc-store-home/closing-report.md`: per-slice walk + composition verdict (SH-5/SH-6),
independently gated, **plus a bubble-up to `project-plan.md`** — a new arc in the roadmap (§2) and a
DoD row (§5), since this expands the v1.0.0 DoD. A failed compose row spawns a remediation slice.

## Method

Per-slice **open set** (`slice-doc.md` / `ledger.md` / `cc-prompt.md`) written when each slice
becomes the active work (plan-late, plan-deep). CC implements on local 1.85+; CDC verifies (writing
`cdc-verification.md`); cargo rows attested-by-CC → reproduced-on-CI. Slice closes bubble up here;
the arc closes with `closing-report.md` + the composition check + the project bubble-up.

## Version History

### v1.6 — 2026-07-26
**The arc closes: SH-6 attested via the RH C-5 cutover; SH-7 dispositioned.** RH C-5 relocated odm's
own corpus onto the orphan `odm` branch (`.worktrees/odm`, 60 nodes), reduced `odm.toml` to a locator,
split the operational config into the store's `config.toml`, and retired `nodes/` from the working
branch — closing **SH-6** (the dogfood row) and completing **project-plan P-14**. **CDC reproduced the
cutover independently** on a clean clone: `check` green cold at 60 nodes; VISION + CURRENT FOCUS render
cold (L-3a); ids preserved (60 = 60, nothing minted — G-1-safe); `created` spans 2025-12-27 →
2026-07-25 (14 distinct days, was 45× one date); one F-18 role-suffix hit, the #15 *document* node's
genuine `(build plan)` title. An **independent fresh-context arc-gate** returned **PASS-WITH-NOTES**:
all six load-bearing dogfood claims verified against the running binary; both implementer judgment calls
endorsed — ULID-shard files are *not* relocated by corrected `created` dates (the shard is
identity-derived; moving them would break `Store::load`), and `context.json` is shared-by-design (rides
to clones to serve the fresh-session DoD), with a two-level project-focus/working-focus split recorded
as the clean longer-term model. The gate's one note was this ledger reconciliation itself (SH-7 open + a
v1.6 entry the closing report already cited) — now fixed. SH-1…SH-4 closed, SH-5 reproduced, SH-6
attested. **arc-store-home is CLOSED** (durable CI-`reproduced` on the cargo rows + both-git-arms matrix
still rides the push). Surfaced by: the RH C-5 cutover + the independent arc-gate.

### v1.5 — 2026-07-26
**Slice 04 closed (SH-4 attested); the arc is code-complete and SH-5 reproduced.**
`odm store rename` moves the worktree and/or renames the local branch, rewriting `[store]` from
the **observed** git state so the store is never lost. Three git behaviours were probed rather
than assumed, and two of them shaped the design: **`git worktree move` behaves like `mv`** (an
occupied destination is moved *into*, and reported as success — so odm must pre-check
collisions itself, and must read back where the tree actually landed), and **`branch -m` works
on an unborn branch**, so a freshly bootstrapped store needs no special case.

**A real bug was found and fixed in-slice, and it is the one this arc most needed to catch.**
The first cut wrote the locator only after both git ops succeeded. With the move succeeding and
the branch rename failing, the locator was left naming a directory that no longer existed —
whereupon resolution self-healed an empty store at the stale path and `odm check` reported
**"ok (0 node(s), no problems)"**. A green check over an invisible corpus: silence, not an
error. Fixed by writing the locator from observation *even when a later step fails*, and
reporting what did succeed. Pinned by a regression test.

**SH-5 reproduced:** bootstrap → attach → ff-sync → rename, `check` green on 2 nodes after all
four (against a local bare remote). **SH-6 (dogfood cutover) stays open, pending RH C-5** —
recorded, not dropped. Raised for later: `--yes` is accepted and unused across all `store`
subcommands; `worktree_base` is not renameable (only the leaf dir and the branch are). Silent-
drop diff: none. Surfaced by: slice 04 implementation (CC).

### v1.4 — 2026-07-26
**Slice 03 closed (SH-3 attested); `odm store init` is now all three arms.** Slice 02's two
"defer to slice 03" stops are replaced: **attach** checks out an existing branch (never
re-orphans, never re-scaffolds — the store rides in with the branch), and **ff-sync** fetches,
classifies by ancestry, and fast-forwards or explains why it will not. The classification is a
**pure function** — git measures ancestry, a five-case table decides — so every case is
unit-tested without a repo, a remote or a network, and the integration tests then prove the
measurement against a **local bare remote**.

**The safety invariant is the point of the slice:** divergence is never resolved automatically.
The store branch is shared by construction (§6), so merging or rebasing it would rewrite
history other clones already hold. The test asserts that three ways rather than trusting the
message — B's HEAD unmoved, B's work intact, and A's work *not* silently merged in.

**§5 widened, deliberately and on the record**, from *create* to *create + attach + ff-sync*:
`gix` 0.66 offers none of `worktree add`, `fetch` or `merge --ff-only` for worktrees. All of it
stays in the one shell-out module, and the invariant that matters is unchanged — **`init`-time
only, steady state on `gix`** (L-15, checked mechanically).

**One deviation from the prompt, recorded not hidden:** a `--dry-run` sync *does* fetch. Built
the other way, the preview compared against a stale upstream and reported `sync-up-to-date` for
a store a real run would fast-forward — a dry run that can disagree with the run it previews is
worse than none, because it is trusted. A fetch cannot touch the store (remote-tracking refs
only), and the dry-run message says so outright.

Also: one slice-02 test changed *meaning* rather than regressing — a branch with no worktree is
now classified precisely as the repair case (still refused, now pointing at `--force`), so its
assertion moved to the current message. **SH-5 is now reproducible**; the closing report's
bootstrap → attach → ff-sync walk is that demo. Raised for later: `--yes` is still accepted and
unused on every arm; `sync` assumes the upstream remote is `origin`; `--json`'s `worktree`
still duplicates `store_root`. Silent-drop diff: none. Surfaced by: slice 03 implementation (CC).

### v1.3 — 2026-07-26
**Slice 02 amended — the flagged risk was the defect.** The closing report warned that the
modern `--orphan` arm was verified at the argv level only, because local git 2.39.5 can never
run it, and that the CI git version was therefore load-bearing evidence. It was: the argv was
**wrong** (`worktree add --orphan <branch> <dir>`, where 2.42+ requires `-b <branch>`), so
`odm store init` failed on every current git. CDC reproduced it in a container on **git 2.43**
— 6 of 8 integration tests — root-caused it, and verified the one-line fix. Corrected here; the
unit test was corrected too, since it had asserted the broken argv and so could never have
caught it.

**This is the closer-≠-verifier rule paying for itself.** The implementing environment could
not execute the failing path at all, so no amount of care there would have found it; an
independent verifier on a different git did, immediately.

**The missing guard is why it hid, so the guard is part of the fix.** CI now *asserts* the git
version instead of assuming it: the `test` job fails fast below 2.42 (so the modern arm is
always under test), and a new **`test-old-git`** job runs `store_init` on a pre-2.42 container
for the fallback arm. Two arms, two jobs — they must produce an identical empty store, which is
exactly what the in-slice fallback bug violated, so this is a standing check rather than a
one-off. **SH-2 attested as amended**; `reproduced` once both jobs run green. Surfaced by: CDC
verification (`slice02…/cdc-verification.md`).

### v1.2 — 2026-07-26
**Slice 02 closed (SH-2 attested).** `odm store init` stands the home up end to end — worktree,
orphan branch, `[store]` locator, scaffolded `config.toml` + empty `nodes/`, gitignore entry —
and slice-01 resolution then redirects every command into it. The command is born under
**`odm store`** (ODD-0023's third tier) rather than as a top-level `init` to be renamed later;
none of the rest of that reorg is here. The `git` shell-out is confined to one module
(`worktree.rs`), as ODD-0022 §5 ratifies, and steady state stays on `gix`.

**Two bugs the slice surfaced, both fixed here.** (1) **An orphan branch is invisible to a refs
lookup** — `checkout --orphan` leaves it *unborn*, so `refs/heads/<branch>` does not exist until
the first commit, and the arc-plan's refs-only detection could not see the home `init` had just
created (a second `init` would try again). Detection now also treats an existing store directory
as "exists locally": the directory is the reliable signal during the unborn window, the ref
afterwards. (2) **The old-git fallback was not equivalent to the modern path** — `checkout
--orphan` deliberately keeps the working tree, so on git < 2.42 the code branch's whole checkout
landed in the store, staged; `worktree add --orphan` produces an empty tree. Fixed with a
`git rm -rf` step. Caught by *running* it, not by the tests, which asserted what the store
contained but not what it did **not** — the both-halves discipline slice 01 used and this slice
initially missed; the assertion is now there.

**⚠ CI needs git ≥ 2.42.** Local git is **2.39.5**, so the *fallback* is the arm actually
exercised here and the modern `--orphan` arm is verified only at the argv level. Both arms must
produce identical stores — which is exactly what bug (2) violated — so the version CI runs is
load-bearing evidence, not an environment detail.

**Carried items disposed:** `.odm/context.json` now follows the resolved store root (it names
node ids, so it rides with the nodes); `ROLLUP.md` stays at the invocation root by decision — it
is a projection out of the store for a code-branch reader. Three things raised for slice 03: the
`--json` `worktree`/`store_root` duplication, `--yes` accepted but unused, and that `init`
leaves an untracked `config.toml` (the branch is unborn by design), which sync must reason
about. Silent-drop diff: none. Surfaced by: slice 02 implementation (CC).

### v1.1 — 2026-07-26
**Slice 01 closed (SH-1 attested).** The store root resolves through `odm.toml`'s `[store]`
locator, and the operational config (gate-sets, display, author) is read from the store's own
`config.toml` — falling back to `odm.toml` when the store has none, so **no existing repo
changes behaviour**: odm's own corpus is untouched, `check` green at 60 nodes with `nodes/`
still at the repo root. Nothing is created — no worktree, no branch, no directory — and L-7
verifies that mechanically (no `git` subprocess anywhere new).

Two design points settled in the slice, both recorded in the closing report: **resolution never
fails** (a malformed `[store]` falls back to the un-redirected default rather than making the
corpus unreachable — strict parsing and its error stay in `StoreConfig::load`), and **`root`
keeps meaning the *invocation* root** (only the node tree follows the store; a relative path
argument and the config search still resolve against the cwd).

**Three items the arc-plan did not anticipate**, raised for slices 02–04 rather than decided
here: (1) `.odm/` now splits across two roots — the index follows the store, but
`.odm/context.json` does not, and since it names node ids it arguably should; (2) `ROLLUP.md`
stays at the invocation root, which is defensible for a projection *out* of the store but is a
choice ODD-0022 does not make; (3) `branch_name` is parsed but has no consumer until slice 02.
Silent-drop diff: none. Surfaced by: slice 01 implementation (CC).

### v1.0 — 2026-07-26
Arc created from **ODD-0022 (Accepted)**. Named arc, number deferred (`arc-release-hardening`
precedent). **v1.0.x — expands the v1.0.0 DoD** (operator call). Four slices: resolution/two-config
→ `init` bootstrap → `init` attach/ff-sync → `rename`. The **dogfood cutover rides RH C-5's
re-self-host** (coupling recorded in §Dependencies + SH-6), so the corpus is regenerated-and-relocated
in one pass. Surfaced by: ODD-0022 acceptance + the operator slice-breakdown session.
