# Arc 06 — Migrate, self-host & PM-skill (plan-of-record)

> Refs: ODD-0013 §9 (migration) + §11 (scope beyond the engine: the PM-skill overhaul);
> ODD-0015 A6 row; `project-plan.md` §2; `docs/dev/skill/0001-project-x-pm-post-mortem.md`
> (the GOOD/BAD seed for the PM skill); the collaboration-framework PM layer in
> `billosys/ai-engineering`. `depends_on:` A1 (the node model + store) + A3 (a stable
> command surface — `orient`/`rollup`/`check` — for the self-hosted plan to run on).
>
> **Status:** planned, not started. Slice breakdown at one-line altitude; per-slice doc
> sets written when the arc becomes active.
>
> **This is the self-hosting trigger.** The MVP (A1–A3) has been flagged throughout as
> the point after which *the plan migrates into `odm` as nodes*. This arc closes that
> loop: once `migrate` can import odm's own docs, odm manages its own plan, and these
> design documents become queryable through `odm orient`.

## Capability

Close the bootstrap loop and retire the prose it replaces. A **`migrate` importer**
(`odm-migrate`) maps the legacy number-/directory-based model onto the new one —
`number` → fresh ULID (legacy number kept as metadata), `DocState` scalar → `odd`
gate-set position, state-directory → dropped (was redundant truth), `supersedes` pair →
`supersedes` edge, dustbin/Removed → supersede-don't-delete + git history. It is
**idempotent**, **`--dry-run`-able**, and **never deletes** legacy files. Run on odm's
own `docs/`, it makes odm **self-host** its plan; then the framework's *mechanical* PM
prose is replaced by "when you need to X, run `odm <cmd>`" entries in a standalone PM
**skill** (seeded by the ODD-0001 post-mortem's missteps), and the redundant prose is
retired in favor of `odm check`. The `odm-migrate` crate + the
`billosys/ai-engineering` PM skill.

## Exit criteria (arc acceptance)

- `odm migrate <legacy-path>` imports legacy ODDs into the new model: idempotent
  (re-running is a no-op), `--dry-run`-able, supersede-not-delete (no legacy file
  removed; git preserves history).
- The importer runs cleanly on odm's **own** `docs/design` corpus; `check` passes on the
  imported graph.
- **odm self-hosts:** its plan (0011–0018 + the `design-v1.0.0` plan set) lives under
  `nodes/`, and `odm orient`/`rollup`/`check` run on the real corpus.
- The PM **skill** in `billosys/ai-engineering` is populated from ODD-0001 (GOOD/BAD
  counter-examples + "run `odm <cmd>`" entries), and the redundant *mechanical*
  framework prose is retired with a pointer to `odm check`.

## Slices (dependency-ordered, one-line scope)

1. **slice01 — `migrate` importer core.** The legacy → new mapping (number→ULID,
   DocState→gate, dir-state dropped, supersedes-pair→edge, dustbin→supersede+git);
   idempotent describe-or-create; `--dry-run`; never-delete. — `odm-migrate`.
2. **slice02 — migrate odm's own docs.** Run the importer on `docs/design` (0011–0018);
   resolve real-corpus edge cases; `check` green on the imported graph.
3. **slice03 — self-host cutover.** Bring the `design-v1.0.0` plan set (project-plan,
   arc-plans, slice docs) into the node model; the design docs move under `nodes/`;
   `orient`/`rollup` run on the self-hosted corpus. *The loop closes.*
4. **slice04 — PM-skill population.** From ODD-0001, build the standalone PM skill in
   `billosys/ai-engineering`: GOOD/BAD counter-examples + "when you need to X, run
   `odm <cmd>`" entries, seeded by the prior project's missteps.
5. **slice05 — retire redundant framework prose.** Replace the framework's *mechanical*
   PM rules (numbering, ordering, deferral-tracking, drift-watching) with pointers to
   `odm check` / the relevant commands; keep the posture/craft prose that odm does not
   mechanize.

## Dependencies

Consumes: A1's node model + store + `supersedes` edge + `retire`; A3's command surface
(`orient`/`rollup`/`check`) for the self-hosted plan to run on. Optional adjacency:
A5's reconciler, once present, runs on the self-hosted corpus; interop (ODD-0017
export-projection) rides alongside self-host but is its own thread (horizon, not an A6
slice). Independent of A4 (self-host does not require the index).

## Open design questions (resolve in slice docs)

- **What migrates, and from where.** odm's own `docs/design` ODDs are the obvious first
  corpus; whether to also migrate oxur's `crates/design/docs` (ODD-0012's mention) is a
  scope call for slice02. The in-flight `design-v1.0.0` plan set (this very directory) is
  the self-host target in slice03 — sequence its import carefully so the plan describing
  the migration is itself migrated last.
- **PM-skill boundary.** Which framework prose is *mechanical* (retire → `odm check`)
  vs. *posture/craft* (keep) — the ODD-0013 §11 split. Settle in slice04/05 against the
  actual PM-skill draft, not in the abstract.
- **Self-host cutover safety.** Migrating the plan that governs the migration is
  reflexive; slice03 needs a clean before/after (`--dry-run`, git checkpoint) so a bad
  import is recoverable.

## Method

Ledger per slice; CC implements, CDC verifies; cargo rows via CI / local 1.85+;
five-iteration cap. Slice closes bubble up to this arc-plan; the arc closes with its own
`closing-report.md` + composition check. **On arc close, the v1.0.0 MVP-plus is
self-hosting** — and per the project-plan, the A7/A8 telemetry/forecasting horizon
becomes scopable.

## Version History

### v1.0 — 2026-06-26
Initial arc-plan, drafted from ODD-0013 §9/§11 + the ODD-0015 A6 row + the ODD-0001
PM-skill seed, as part of the project-plan synthesis session. No slices started;
one-line altitude per *plan late, plan deep*.
