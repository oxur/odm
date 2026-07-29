# Slice 12 (Migration Fidelity) — Reconcile capability (plan-of-record)

> Refs: `../arc-plan.md` (s12 row; MF-9 compose; P-12 DoD) · **ODD-0025** §2.1 (the body-hash gate is a
> **migration-time gate only** — nothing re-verified later), §2.8 (update-in-place repair) · s06
> (`reconcile_source` — which today *skips* a drifted non-stub, recording it, deferring the fix here) ·
> s11 (the synthesis capability the vision re-cast applies) · the s08/s10 CDC verifications (the drift
> this reconciles). `depends_on:` s11.
>
> **Capability — fixture-only, no live store mutation** (the arc's s06/s09/s11 rhythm). It builds the
> operation that re-establishes 1:1 fidelity after sources legitimately evolved; the **live close** (fire
> the reconcile + the vision mint on `.worktrees/odm`) is **s13**, and the **arc-close** (MF-9 composition
> + the P-12 acceptance demo + the arc closing-report) follows s13.

## Goal

Give odm a **reconcile** step that makes a corpus *re-faithful* after its sources legitimately changed —
the "repeatable" half of the arc's charter. **Done when** a drifted non-stub node can be **re-snapshotted
in place** (body ← current source, `id`/`edges`/`status` preserved, hard body-hash gate re-passed) instead
of skipped; a node whose source **moved** is re-discovered by identity and its `source.paths` updated to
the portable (s08-relative) new path before re-snapshot; the **living-plan-node policy** is decided and
implemented (how a node whose source is a still-changing doc reconciles); and s11's synthesis is wired
into a **vision-apply** path that re-casts the project node as an editorial-merge synthesis over a 1:1
`project-plan` node — **all fixture-proven, with `.worktrees/odm` untouched** (s13 fires it).

## Why

The arc promised migration that is faithful *and repeatable*. Faithful it now is; repeatable it is not
yet — because when a source legitimately changes, odm currently **skips** the drifted node rather than
re-establishing fidelity. That is deliberate: `reconcile_source` records a drifted non-stub in `drifted`
and defers the fix to this slice (its own doc comment: *"living-doc drift is the arc's s12 reconcile-run's
job"*). The arc's own verifications found the concrete cases: the active **arc node** drifts because
`arc-plan.md` is a living doc (s08); design nodes **ODD-0013/0020** drift because their legacy files were
amended (s10); and s11 **moved** ODD-0013/0017/0018 (`01-draft/`→`04-accepted/`) and changed their
`state:`, so their nodes now carry both a stale `source.paths` **and** a stale body/gate-vector. Reconcile
is the operation that closes all of these — and it is the last capability the P-12 self-host DoD needs,
because "odm self-hosts *faithfully*" (MF-9) requires the corpus to be re-faithful after this arc's own
edits.

**Capability-first** matters most here of anywhere: the s13 live run **rewrites odm's own normative-ODD
nodes** to produce the shippable corpus. Proving the re-snapshot + re-discovery logic on fixtures before
it touches the live store — and before the arc-close reproduces P-12 against it — is the discipline the
whole arc has used (s06→s07, s09→s10). No live write here.

## Scope

**In (code + fixtures; NO `.worktrees/odm` store mutation):**

- **Re-snapshot mode** (the core). Turn a drifted non-stub from *skip-and-record* into an **update-in-place
  re-snapshot**: read the current source, replace the node body, **preserve `id`/`edges`/`status`**, and
  re-pass the §2.1 hard body-hash gate (which now trivially holds, since the body was just set from the
  source). Keep the s04 **stub-repair** path distinct from this **non-stub re-snapshot** — both correct,
  neither silently doing the other.
- **Moved-source re-discovery.** When a node's stored `source.paths` no longer resolves but the source
  exists elsewhere (it moved — e.g. the L-8b `01-draft/`→`04-accepted/` renames), **re-discover it by
  identity** (the `number`/coordinate the corpus already keys on) and rewrite `source.paths` to the
  portable **s08-relative** form of the new path, then re-snapshot the body. One node can need both (a
  moved path *and* a changed body — 0013/0017/0018).
- **The living-plan-node policy** (the s08 open question). Decide and implement how a node whose source is
  a *still-changing* doc reconciles. **Recommended:** reconcile-to-current (a point-in-time snapshot) with
  **no special exclusion** — the §2.1 gate is *migration-time-only*, so inter-reconcile drift is by design
  invisible to `check`, and at arc-close the source is stable so the node stays faithful; the project
  *synthesis* node stays excluded from 1:1 as it already is (§2.3), but a 1:1 arc/plan node does not need
  to. Record the decision; **amend ODD-0025 if it needs a line** (don't work around).
- **Vision-apply path** (MF-7's live half, built here fixture-only). Using s11's `synthesis`, wire the path
  that re-casts the project node as an **editorial-merge synthesis superseding a 1:1 `project-plan` node**.
  Fixture-proven; **no live mint** (s13).
- **The ODD-0025 §4 decision** (the s09 LOW follow-up). Resolve it: either amend §4 to say `artifact`
  stamps the shared `SchemaVersion::CURRENT` (`v1.1`) — which, because ODD-0025 *is* node #25's source,
  makes #25 a re-snapshot target for **s13** — or record an explicit decision to defer. Decide here; do not
  leave it dangling.
- **Fixtures** (`TempDir`) for all: re-snapshot a drifted non-stub (id/edges preserved); re-discover a
  moved source + update `source.paths`; the living-plan-node case; the vision-apply; stub-repair vs
  non-stub-re-snapshot; idempotence + dry-run-safety.

**Out:**

- **The live close → s13.** Firing reconcile (re-snapshot every drifted node: the arc node, ODD-0013/0017/
  0018/0020, and #25 if §4 is amended) **and** the live vision mint on `.worktrees/odm`, behind the s07
  snapshot → dry-run → adjudicate → fire → verify protocol — the run that produces the shippable corpus.
- **The arc-close → after s13.** MF-9 composition (do all twelve slices compose into "odm self-hosts
  faithfully"?) + the **P-12 acceptance demonstration** (a fresh session orients fully from `odm orient`
  alone on the reconciled corpus) + the arc `closing-report.md` + the bubble-up to `project-plan.md`. Per
  PROJECT-MANAGEMENT this is the formal arc-close (CDC assembles; an independent gate signs off), not a
  slice's implementation.
- **L-8a** (migrate the whole design corpus into nodes) — post-1.0 follow-on, not this arc.
- No change to the 1:1 rule or the gate's *migration-time-only* nature; reconcile re-establishes fidelity,
  it does not make the gate a continuous check.

## Verification

Fixture / `TempDir`, class-(a) (fixture-attested → CI). **No live class-(b) row** — the store is
untouched. After the slice: a drifted non-stub re-snapshots in place (id/edges/status preserved, gate
re-passes); a moved source is re-found by identity and its `source.paths` rewritten to the s08-relative
new path; the living-plan-node case reconciles to current without reject; the vision-apply produces an
editorial-merge synthesis over a faithful 1:1 `project-plan` node; reconcile is idempotent + dry-run-safe.
CDC reproduces the code + fixtures + CI; confirms `.worktrees/odm` is untouched.

## Rollback & findings discipline

Fixture-only — nothing live to revert. A capability gap in review is a normal fix-iteration (five-cap).
Any model gap (the re-snapshot semantics, the living-plan-node policy) → **amend ODD-0025, cited, not
worked around**. The re-snapshot must be **provably reversible in principle** (it only ever sets a body
*from* the current source, preserving identity), so s13's live run can trust it behind the snapshot gate.

## Exit

`ledger.md` closed; CDC-verified against code + fixtures + CI. Reconcile is a real, verified capability and
the living-plan-node + §4 questions are decided. On close, bubble up to `../arc-plan.md`: MF-9's
mechanism is ready; **s13 (live reconcile + vision mint) is next**, and after it the **arc closes** with
the MF-9 composition check + the P-12 self-host acceptance demonstration.
