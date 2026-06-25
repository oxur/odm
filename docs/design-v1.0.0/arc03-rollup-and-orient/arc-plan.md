# Arc 03 — Rollup & orient (plan-of-record)

> The arc that turns `odm` into *the cheap global state for the plan* — the MVP
> capstone. Refs: ODD-0013 §6 (generated rollup), §4.1 (derived-order queries),
> §7 (command surface), §2.2 (node body = way-finding text); ODD-0015 §1 (A3 row)
> + §2 (MVP program-DoD) + §5 (E5 deferred re-entry); ODD-0001 E1/C5/E5.
> `depends_on:` Arc 02 (the graph engine + queries must exist).
>
> **On close, MVP (A1–A3) is complete and self-hosting is triggered** — the plan
> migrates *into* `odm` as nodes (A6).

## Goal

Make the whole plan legible from one cheap call. Generate a `ROLLUP.md` (+`--json`)
as a *derived view* of the node files, and an `orient`/`brief` that leads with
**vision → current focus → ready/blocked → drift** so a fresh session orients fully
without reading the tree by hand. Bare `odm` orients, never bare-errors.

This arc also clears the three Arc 02 CDC follow-ups first (slice01), so the rollup
can render tear rationale and `check` severities are calibrated before they surface.

## Exit criteria (arc acceptance)

- A fresh session reaches full situational awareness from `odm orient` alone — the
  MVP program-DoD (0015 §2).
- `ROLLUP.md` (+`--json`) is **generated and regenerable from the node files alone**
  (`odm rollup`); never hand-edited. Any future cache (A4) is derived from it.
- `orient` leads with **vision** (the current project node's body — D-1), then current
  arc, then ready/blocked, then drift. Bare `odm` orients (never bare-errors).
- Per-node status vectors render in **gate-sequence order** (not alphabetical — D-4);
  active tears show their **rationale** (once slice01 lands); provenance (`origin`)
  and deferred nodes are surfaced.
- The three Arc 02 cleanup follow-ups are closed and the 0013 doc reconciliations are
  applied (slice01).
- `--json` carries a stable, documented schema for rollup and orient (slice04).

## Decisions (ratify here; cited downstream)

- **D-1 — Vision source = the current project node's own body.** `orient` reads the
  current project (selected via `use`/`context`, built in arc01 slice05) and leads
  with its way-finding body. Vision is a **derived view of the top node** — *not* a
  new `vision:` frontmatter field, *not* an `odm.toml` string, *not* a separate
  vision file. Rationale: (a) it is already in the model — every node's body is its
  way-finding doc (0013 §2.2), and the project sits atop the `part_of` tree, so its
  body simply *is* the program vision; (b) the plumbing already exists — `use`/`context`
  selects the current project, so "where does vision come from" is answered by the
  context that is already there; (c) a separate field/config/file would re-introduce
  a **second source that drifts from the plan** — exactly the E1/C5 (vision-loss /
  stale-doc) failure `orient` exists to kill. Same principle as "the rollup is a
  derived view, never separately maintained": vision is a derived view of the project
  node, not its own artifact. *(Recommended by Duncan 2026-06-25; ratified here.)*
  - **D-1a — Rendering: lead, don't dump.** `orient` leads with the project `name`
    + the body's `# Vision` section if present, **else the body's lead section**
    (content before the first subheading) — not the whole body. Rationale: `orient`
    is read at the start of *every* session and must stay scannable (vision → focus →
    ready/blocked → drift); dumping the full body buries the actionable part, and the
    full text is one call away via `odm show <project>`. The `# Vision` convention
    gives authors explicit control and degrades gracefully when absent. *Exact parse
    rule is a slice03 detail (see Open questions).*
  - **D-1b — Escalation (deferred; not built in A3).** If a project later wants a
    richer north-star, the right move is a **vision *document node*** (`part_of` the
    project) that `orient` surfaces — never a top-level frontmatter field. Note it as
    a future option; out of A3 scope.
- **D-2 — The rollup is a derived, full-scan regenerate (not a cache).** A3 has no
  index; the `.odm/` incremental cache is A4 (0013 §6.1, research-gated Q-9). A3's
  rollup walks `nodes/` each time. (Per the handoff; ties A4 to a real need rather
  than speculative caching.)
- **D-3 — The rollup model lives in `odm-core`; `odm-cli` renders only.** A `rollup`
  model (way-finding tree + per-node status + ready/blocked, assembled from the graph)
  is built in `odm-core`; the CLI renders Markdown + `--json` over it. Mirrors arc02's
  "thin CLI over odm-core ops" and lets `orient` (slice03) and `--json` (slice04)
  reuse one model instead of re-deriving views.
- **D-4 — Status vectors render in gate-sequence order**, not alphabetical (slice03
  CDC note carried from Arc 02). The gate order is the per-type `odm.toml` sequence.

## Slices (dependency-ordered)

1. **slice01 — Arc 02 cleanup.** The three CDC follow-ups + 0013 doc reconciliations.
   Persist the **tear rationale** (`tears` carries `{ edge, because }`; round-trip;
   surface in `check`'s active-tears); a **binary-level `assert_cmd` suite** in
   `oxur-odm/tests/` reaching `run()`/`ExitCode`; **`check` severity recalibration**
   (orphan + decomposition-drift = Error; undeveloped-stub + advanced-without-decomposition
   = Warning). — `odm-core`/`odm-store`/`odm-cli`/`oxur-odm`. *Orthogonal to the
   rollup/orient path; sequenced first so slice02 can render tear rationale.*
2. **slice02 — Rollup generation.** Build the `odm-core` rollup model (D-3) and the
   `odm rollup` command that regenerates `ROLLUP.md` from a full scan (D-2):
   way-finding tree (`part_of`), per-node status vectors in gate-sequence order (D-4),
   ready/blocked sets, active tears with rationale, provenance (`origin`) view, and a
   (pre-A5) drift section. — `odm-core`/`odm-cli`. `depends_on:` slice01.
3. **slice03 — `orient`/`brief` + bare-`odm`.** Compose orient over the rollup model:
   **vision (D-1) → current arc → ready/blocked → drift**; bare `odm` orients, with a
   graceful no-current-project fallback (never bare-errors). — `odm-cli`.
   `depends_on:` slice02.
4. **slice04 — `--json` + polish.** Stable documented `--json` schema for rollup and
   orient; errors-as-affordances polish. — `odm-cli`. `depends_on:` slice02, slice03.
   **← MVP COMPLETE on close.**

## Open design questions (resolve in the slice docs; recommendations marked)

- **Q-A3-1 — RESOLVED (2026-06-25, Duncan + CDC): deferred-node surfacing is out of
  A3.** A3 was nominally to *surface* deferred nodes + their re-entry predicate (0013
  §6; 0015 §5.3), but predicate *evaluation* is A5 and there is **no `deferred`
  representation in the schema yet** (gates are `planned/built/…`; DoR/entry-gate is
  Q-8-deferred; retirement is `retired: {}`). Decision: the rollup surfaces only what
  is representable today; we **do not invent a `deferred` status just to render it**.
  The full deferred + re-entry-predicate path lands with **A5**, once the
  schema/metadata firms up. slice02's rollup model leaves a defined slot for it but
  renders nothing until then.
- **Q-A3-2 — Drift in the rollup before A5.** `reconcile`/drift is A5. *Rec:* slice02
  renders a `DRIFT` section that is structurally present but reads "not yet tracked
  (A5)" until reconcile is wired — keeps the rollup shape stable with no fake data.
- **Q-A3-3 — RESOLVED (2026-06-25, in slice03): exact vision parse rule.** From the
  project body: a `# Vision` heading (case-insensitive, ATX) → that section (to the
  next same-or-higher heading); else the lead section (body before the first ATX
  heading); truncate to a ~15-line budget with a `… (full vision: odm show <project>)`
  continuation marker; always lead with the project `name`. Spec'd in
  `slice03-orient-brief/slice-doc.md` (row O-2).

## Method

Ledger per slice (collaboration-framework discipline); CC implements, CDC verifies
every row. Cargo compile/test rows are reproduced in CI or a local 1.85+ toolchain —
the Cowork sandbox has none (apt cargo is 1.75, below the 1.85 floor); CDC reproduces
grep/structural rows in-session. Coverage rows scope per-crate
(`--ignore-filename-regex`) and state the **line** metric. Five-iteration cap.
Commit the CDC verification alongside the slice push so no slice advances unverified.
