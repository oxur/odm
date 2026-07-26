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
| SH-1 | Slice 01 (resolution + two-config split) closed | ptr: `slice01…/cdc-verification.md` | serious | arc-plan | open | | attested-on-close. Foundational. |
| SH-2 | Slice 02 (git plumbing + `init` bootstrap) closed | ptr: `slice02…/cdc-verification.md` | serious | arc-plan | open | | attested-on-close. |
| SH-3 | Slice 03 (`init` attach + ff-sync) closed | ptr: `slice03…/cdc-verification.md` | serious | arc-plan | open | | attested-on-close. |
| SH-4 | Slice 04 (`rename`) closed | ptr: `slice04…/cdc-verification.md` | serious | arc-plan | open | | attested-on-close. Slottable. |
| SH-5 | **Compose:** `odm init` stands up a working home end-to-end — **bootstrap** on a fresh repo, **attach** on a clone (no fork), **ff-sync** freshens; divergence warns + stops | arc-scale demo: all three modes in scratch repos | serious | arc-plan | open | | reproduce at arc scale. |
| SH-6 | **Compose (dogfood):** odm's own corpus lives on the orphan `odm` branch, `check` green there, and the working branch no longer carries `nodes/` | arc-scale demo: `init` + **RH C-5** re-self-host into the home → `check` green | serious | arc-plan / ODD-0022 | open | | **reproduced jointly with RH C-5** (the cutover). The dogfood proof; the payoff row. |
| SH-7 | bubble-up findings dispositioned | ptr: arc-plan change-log | correctness | bubble-up | open | | accrues as slices close. |

Closes in `arc-store-home/closing-report.md`: per-slice walk + composition verdict (SH-5/SH-6),
independently gated, **plus a bubble-up to `project-plan.md`** — a new arc in the roadmap (§2) and a
DoD row (§5), since this expands the v1.0.0 DoD. A failed compose row spawns a remediation slice.

## Method

Per-slice **open set** (`slice-doc.md` / `ledger.md` / `cc-prompt.md`) written when each slice
becomes the active work (plan-late, plan-deep). CC implements on local 1.85+; CDC verifies (writing
`cdc-verification.md`); cargo rows attested-by-CC → reproduced-on-CI. Slice closes bubble up here;
the arc closes with `closing-report.md` + the composition check + the project bubble-up.

## Version History

### v1.0 — 2026-07-26
Arc created from **ODD-0022 (Accepted)**. Named arc, number deferred (`arc-release-hardening`
precedent). **v1.0.x — expands the v1.0.0 DoD** (operator call). Four slices: resolution/two-config
→ `init` bootstrap → `init` attach/ff-sync → `rename`. The **dogfood cutover rides RH C-5's
re-self-host** (coupling recorded in §Dependencies + SH-6), so the corpus is regenerated-and-relocated
in one pass. Surfaced by: ODD-0022 acceptance + the operator slice-breakdown session.
