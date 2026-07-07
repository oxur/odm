# CC Prompt — Slice 04 (Arc 06): self-host cutover

**The loop-closer.** Bring odm's own `design-v1.0.0` plan set (project-plan, arc-plans, slice
docs) into the node model as **work nodes** — the plan lives under `nodes/`, and
`orient`/`rollup`/`check` run on the self-hosted corpus. After this slice **odm manages its own
plan** and these design docs are queryable via `odm orient` (project-plan P-12; ODD-0013 §9). A
genuine model+import slice (`odm-migrate` + `odm-cli`, reusing the slice01 rails).

> **Start condition:** slices 01–03 CDC-verified. Branch off **`release/1.0.x`**:
> **`arc06-slice04-self-host-cutover`** (not `main`). If 01–03 unmerged, branch off the slice03
> tip + flag for rebase.

## Read first
1. `slice04-self-host-cutover/slice-doc.md` (same dir) — the plan-of-record: the crux
   (**reflexive-import ordering**), the scope boundary (**A1–A6 only**), the mechanism
   recommendation, and the resolved/open design questions.
2. `slice04-self-host-cutover/ledger.md` (7 rows S-1…S-7).
3. `../arc-plan.md` — A6 Arc Ledger (this slice closes **A-4**, mechanism for compose **A-8**);
   the open Qs (§ "What migrates", "Self-host cutover safety").
4. `../../project-plan.md` — §2 roadmap (A1–A6 = MVP; §4 A7/A8 = scoped horizon); §5 **Project
   Ledger** (P-1…P-6 = arc-close status, the authoritative status source; **P-12** = the
   self-host DoD this slice makes reproducible).
5. Reuse: `crates/odm-migrate` (report / idempotence / `--dry-run` / never-delete / two-pass
   id resolution — the slice01 core); `crates/odm-core` (`stamp_schema` from slice03; the
   work-node model, gates, `part_of` containment, `check`); `crates/odm-cli`
   (`orient`/`rollup`/`check`).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then API design + error handling.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence `attested`; per-row walk +
  **Bubble-up to the arc**; **propagate into `arc-plan.md` (A-4 + a version entry)**).

## Task
1. **Plan-set → work nodes** (S-1): import project + **A1–A6** arcs + their slices as
   `project`/`arc`/`slice` nodes under `nodes/`, each `stamp_schema`'d `<type>/v1.0`;
   number/name/created/updated carried; legacy plan-set MD **untouched**.
2. **Containment tree** (S-2): `arc part_of project`, `slice part_of arc`, matching the
   directory hierarchy; no orphan work nodes. **Containment only** — work↔odd reference edges
   are out.
3. **Gate status from the plan** (S-3): closed arc ⇒ gates reached; active ⇒ partial; planned ⇒
   none — at `Evidence::Asserted`. Status source = ledgers / Project-Ledger P-rows (A1–A3 lack
   a `closing-report.md` → use the P-row).
4. **Mixed-corpus `check` green** (S-4): the 13 `odd` nodes + the work tree → `odm check` exit 0
   (findings, if any, are Warnings, never Errors). Verify **no wrong-type-field** on work nodes.
5. **`orient`/`rollup` reproduce reality** (S-5): rollup shows A1–A5 done, A6 active, rest
   planned; orient runs clean over the self-hosted corpus.
6. **Reflexive-import safety** (S-6): `--dry-run` writes nothing; **project node last**; git
   checkpoint; idempotent re-run; never-delete (byte-snapshot at plan-set scale).
7. **Gates + no regression** (S-7): clippy `-D warnings`; no `unsafe`; ≥ 90% new-path line;
   `cargo test --workspace` green; **no `odm-index` change**.

## The crux (do not skip)
The plan-set **contains the plan governing this import** (arc06 arc-plan + slice04's own docs).
Import at a **committed git checkpoint**, `--dry-run` first, build the tree **children-up with
the project root committed last**, and rely on idempotence for the reflexive node (slice04
creates the node describing slice04). A bad import must be `git reset`-recoverable.

## Constraints (flag, don't silently change)
- **Scope = A1–A6.** Do **not** import `arc07-*`/`arc08-*` (the post-MVP horizon, owned by
  another CDC, 0 slices). If you believe the full roadmap should be represented, **flag it** —
  don't import them by default.
- **Mechanism:** recommended = directory-structure + ledger migrate adapter (reuse the slice01
  rails). If ledger-table parsing is fragile, a one-time **cutover manifest** is the fallback.
  **Pick one and flag it** — amend-don't-workaround.
- **Never delete; never mutate the legacy plan-set MD.** The cutover writes `nodes/` only; the
  MD stays the human-authored source (both coexist — retirement is slice06).
- **Asserted evidence** for migrated status (claimed-from-record), consistent with slice01.
- **No `odm-index` change** — self-host rides the store + command surface, not the index.
- **Amend, don't work around.** If the work-node model / `check` / gates need to change to host
  the plan faithfully, raise it as a model amendment — don't hand-fudge nodes to force green.
- **Render** `writeln!`+`tabled`; **branch off `release/1.0.x`**.

## Deliverables
The plan-set adapter (or manifest) + the self-hosted work-node corpus committed under `nodes/`
+ mixed-corpus `check`/`rollup`/`orient` working, with `ledger.md` evidence per row
(`attested`); a `closing-report.md` — per-row walk **plus the Bubble-up to the arc** (did
slice04 deliver A-4 + the A-8 mechanism; the mechanism chosen; the A7/A8 scope call; the status-
derivation for the A1–A3 gap; what it reveals for slice05/06 — esp. which prose is now
redundant, feeding slice06's retirement) — **and propagate into `arc-plan.md` (A-4 + a version
entry)**. Feature branch `arc06-slice04-self-host-cutover`; not `main`/`release/1.0.x` directly.

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap; your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On close,
bubble up to `arc-plan.md` (A-4) per LEDGER-DISCIPLINE v2.0 §A. **This is the self-hosting
trigger** — on close, project-plan P-12 becomes reproducible-at-arc-close.
