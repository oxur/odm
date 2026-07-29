---
id: 01KYP5G31FP3Q590FABBRBR37P
number: 502696800
type: artifact
schema: artifact/v1.1
name: Release Hardening — CDC arc-gate verification (close)
created: 2026-07-27
updated: 2026-07-27
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/rh-arc-gate-verification.md
  class: other
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# Release Hardening — CDC arc-gate verification (close)

> **Gates:** the RH arc close (`closing-report.md`, `69f0f9f` on `release/1.0.x`) · **Date:**
> 2026-07-27 · **Verifier:** CDC — an independent **fresh-context arc-gate** (a subagent with no prior
> exposure, the A4/A5/arc-store-home pattern) reviewing against the running binary, plus CDC's own
> reproduction. The arc lives on `release/1.0.x` (landed directly, not merged from a branch).

## Verdict

**Release Hardening is closed — PASS-WITH-NOTES.** The arc's substance reproduces end-to-end on the
self-hosted store, the ledger is fully resolved (RH-1…RH-10, none open), and the evidence claims are
honest (attested-on-a-real-toolchain, explicitly **not** CI-green — no overclaim). The fresh-context
gate confirmed every load-bearing claim independently. Three notes; two acted on here, one recorded.

## What the gate reproduced (independently, against the binary)

- **RH-6 compose** — `orient` (real VISION + CURRENT FOCUS, not placeholders), `project`/`chain`
  resolve, the old `context`/`path` are gone (unrecognized subcommand), `validate` exit 0 (the two
  G-3 warns), `check` exit 0 (validate+reconcile), and one `node list` showing the C-1 theme + C-2
  types + C-3 tree/de-numbering + C-5 real dates (2025-12-27 → 2026-07-25) & clean names + C-8
  normalized status **together**; a done arc and done slice both read `done`.
- **RH-7** — `check` green; 60 nodes, 9 `design` + 5 `research` + 1 project + 6 arc + 40 slice, **zero
  `odd`**, names de-numbered.
- **Ledger** — RH-1…RH-5/RH-9/RH-10 attested, RH-6/RH-7 reproduced, RH-8 done; C-7 retired into C-5;
  no `open`.
- **Bubble-up + honesty** — §2a marks RH closed; the report states plainly the cargo rows are attested
  but **not** CI-reproduced (branch unpushed, origin SSH); carried-forward items (`store status`,
  shared-`context.json`, **L-8b**, **G-1**, F-16/F-17/F-21, the L-1 remainder) all recorded, silent-drop
  diff none.

## The three notes — disposition

### 1. Bubble-up drift in `project-plan.md` — **FIXED (CDC)**
The gate caught that the RH close updated the top of §2a/§3 but left older prose beneath it
**self-contradictory** — the §2a **Store Home row still said "SH-6 open, pending RH C-5"** (contradicting
the RH row and reality), a leftover "**C-4 + C-5 remain**" clause in the RH §3 bullet, the whole **Store
Home §3 bullet still "▶ ACTIVE / slice 04 next"**, the §3 header still dated 2026-07-26, and the
sequencing prose still "Release Hardening (in progress; C-1…C-3 done)". Part of this was a merge
artifact (CC's v1.11 update layered over my earlier v1.10 text). **Ironic given the arc hardened against
exactly this** (F-15/L-2, plan-of-record drift). I swept all of it: Store Home → CLOSED, the RH bullet's
stale tail removed, the header re-dated, the sequencing rewritten to "RH closed → arc-store-home closed →
LLM (lighter) → A6". Reconciliation the gate surfaced; recorded that I did it (bubble-up is normally the
closer's, but this was drift the gate found).

### 2. `orient` READY surfaces the **retired** #1605 — **CONFIRMED, recorded as a residual (F-22)**
A real one, and worth naming. `node list` correctly withholds the retired tombstone #1605 (the F-15
fix), but **`odm orient`'s READY block lists `slice #1605 Slice 05 (Arc 06): UAT — CLI feedback`** as
actionable — the exact "trust the filesystem tombstone as live work" hazard (F-15 / UAT L-2) the arc
existed to kill, still present in `orient`'s readiness view. Verified directly: `orient` READY includes
#1605; `node list --all` shows it `retired`. C-3 applied the withdrawn-node filter to `list`; it does
**not** reach `orient`'s ready-frontier computation. **Not a close-blocker** — outside RH-6's literal
claim set (which is `list`/`project`/`chain`) and the arc's ledger is honestly complete — but a genuine
residual. **New finding F-22**, recommended route: the **LLM-command-surface arc slice 02** (the "ready
half" work already reshapes `next`/readiness), or a small fast-follow. The fix is one filter applied at
the readiness set, mirroring C-3's `list` exclusion. Operator's call on route; recorded, not dropped.

### 3. Cosmetic — the closing-report's compose block writes "`project #1600`"; `project` actually
reports `arc #1600` (the focus is an arc). The command resolves; the label is imprecise. Noted, not
worth a re-commit on its own.

## Ledger

- **RH arc CLOSED** — PASS-WITH-NOTES; RH-6/RH-7 reproduced, RH-8 done, all chunk rows attested. The
  **durable `reproduced` (CI) rides the push** — origin is SSH, the branch unpushed; this stays visible
  until the push + both-git-arm CI clears (do not read the cargo rows as CI-green).
- **CDC-fixed:** the `project-plan.md` bubble-up drift (note 1).
- **Recorded forward:** **F-22** (`orient` READY shows retired nodes → route to LLM arc slice 02 / a
  fast-follow); plus the arc's own carried items (`store status`, shared-`context.json`, **L-8b** +
  **G-1** as the two standing pre-ship gates, F-16/F-17/F-21, the L-1 remainder).
- **Next:** the **LLM command surface** arc (materially lighter — C-8 delivered its blocking slice L-1)
  → **A6 resumes at slice05** (PM-skill) + slice06 against the now-settled surface → A6 arc-close →
  the v1.0.0 DoD demo (P-7).
