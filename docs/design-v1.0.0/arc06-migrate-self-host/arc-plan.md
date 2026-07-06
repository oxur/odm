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
3. **slice03 — schema versioning (ODD-0020).** Add the per-type `schema: <type>/vN.N`
   frontmatter marker + per-type field-validity (a `check` extension); `migrate` stamps
   `<type>/v1.0` and reads unversioned legacy as `v0.1`; new odm-created nodes stamp `v1.0`.
   **Precedes self-host** so the plan-set enters `nodes/` already at `v1.0`. — `odm-core` /
   `odm-migrate` / `odm-cli`. *(Inserted v1.5 — the operator's schema-versioning decision.)*
4. **slice04 — self-host cutover** *(was slice03)*. Bring the `design-v1.0.0` plan set
   (project-plan, arc-plans, slice docs) into the node model; the design docs move under
   `nodes/`; `orient`/`rollup` run on the self-hosted corpus. *The loop closes.*
5. **slice05 — PM-skill population** *(was slice04)*. From ODD-0001, build the standalone PM
   skill in `billosys/ai-engineering`: GOOD/BAD counter-examples + "when you need to X, run
   `odm <cmd>`" entries, seeded by the prior project's missteps.
6. **slice06 — retire redundant framework prose** *(was slice05)*. Replace the framework's
   *mechanical* PM rules (numbering, ordering, deferral-tracking, drift-watching) with
   pointers to `odm check` / the relevant commands; keep the posture/craft prose that odm
   does not mechanize. *(Folds in the carried `CLAUDE.md` oxur-cli doc-drift fix, v1.2.)*

## Arc Ledger

> Per LEDGER-DISCIPLINE v2.0 §B (Option A: opens here, closes in the companion
> `closing-report.md`). Class-(b) composition rows stated up front from the capability;
> class-(a) slice-closed and class-(c) bubble-up rows accrue as slices close. **Class-(b)
> rows are reproduced at arc scale — an end-to-end demonstration, never inherited.**

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| A-1 | slice01 (`migrate` importer core) closed | ptr: slice01 `cdc-verification.md` | correctness | arc-plan | open | attested: CC closing-report (`4479076`; 7/7; cov `odm-migrate` lib 99.56% / mapping 99.05% / legacy 91.03% line, `odm-cli` migrate.rs 96.61%); new `odm-migrate` crate + `odm migrate <path> [--dry-run]`; faithful mapping (number→number+ULID, state→cumulative `odd` gate / dustbin→retire, supersedes-pair→edge, type=odd, metadata carried, author→`extra`); idempotent on preserved `number`; `--dry-run` writes nothing; never-delete proven by byte-snapshot; malformed/edge → reported skip/warn, no panic; **no** `odm-index` change. Tested on fixtures (`test-data/legacy`, `test-data/legacy-edge`). cargo rows pending CI. Branched off `release/1.0.x`. | → `done` when slice01 reproduces (CI green). |
| A-2 | slice02 (migrate odm's own docs) closed | ptr: slice02 `cdc-verification.md` | correctness | arc-plan | open | attested: CC closing-report (`028d21d`; 6/6; cov `odm-migrate` lib 99.56% / mapping 99.05% / legacy 93.86% line); `odm migrate docs/design` → 12 `odd` nodes committed under `nodes/2026/07/`, legacy intact; `odm check` green ("ok, 12 nodes"), green-by-construction (document nodes orphan-exempt); three slice01 flags settled **without amendment** (distinct `(type=odd,number)` space; `supersedes` parser hardened for `null`/number/`"ODD-NN"`; no multi-supersession in corpus); no `07-deferred` → `deferred→retire` interim stands; idempotent + `--dry-run` at real scale; **no** `odm-index` change. cargo rows pending CI. Branched off slice01. | → `done` when slice02 reproduces (CI green). |
| A-3 | slice03 (**schema versioning** — ODD-0020) closed | ptr: slice03 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-4 | slice04 (self-host cutover) closed | ptr: slice04 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-5 | slice05 (PM-skill population) closed | ptr: slice05 `cdc-verification.md` | correctness | arc-plan | open | | attested |
| A-6 | **Compose:** `odm migrate` imports legacy ODDs into the new model — idempotent, `--dry-run`-able, supersede-not-delete (no legacy file removed) | arc-scale demo: migrate a legacy corpus; re-run is a no-op; no deletions | serious | arc-plan / 0013 §9 | open | mechanism-complete (slice01, `4479076`): faithful mapping + idempotent (keyed on preserved `number`) + `--dry-run` + never-delete (byte-snapshot proven) + loud-on-malformed, on fixtures. To **reproduce at arc scale** at arc-close (never inherited) — on odm's own corpus once slice02 lands. | reproduce at arc scale |
| A-7 | **Compose:** the importer runs cleanly on odm's **own** `docs/design`; `check` passes on the imported graph | arc-scale demo: migrate odm's docs; `odm check` green | serious | arc-plan | open | mechanism-complete (slice02, `028d21d`): `odm migrate docs/design` imports 12 `odd` nodes; `odm check` exit 0 (green-by-construction) on the imported graph; legacy intact; idempotent. To **reproduce at arc scale** at arc-close (never inherited). | reproduce at arc scale |
| A-8 | **Compose:** odm **self-hosts** — its plan lives under `nodes/` and `odm orient`/`rollup`/`check` run on the real corpus | arc-scale demo: `odm orient` over the self-hosted plan | serious | arc-plan / 0013 §9 | open | | reproduce at arc scale. The self-hosting trigger. |
| A-9 | **Compose:** the PM skill is populated from ODD-0001 and the redundant *mechanical* framework prose is retired with a pointer to `odm check` | arc-scale demo: the skill carries the "run `odm <cmd>`" entries; the retired prose points to odm | serious | arc-plan / 0013 §11 | open | | reproduce at arc scale |
| A-10 | bubble-up findings dispositioned | ptr: arc-plan change-log | correctness | bubble-up | open | | accrues as slices close |
| A-11 | slice06 (retire redundant framework prose) closed | ptr: slice06 `cdc-verification.md` | correctness | arc-plan | open | | attested. **Class-(a) stable ID** — appended (not a renumber) per the stable-ID discipline when slice03=schema was inserted; class-(a) = A-1…A-5 **+ A-11** (6 slices). |
| A-12 | **Compose:** nodes carry a per-type `schema: <type>/vN.N` marker; `migrate` stamps `v1.0` and reads unversioned legacy as `v0.1`; a wrong-type field is a `check` finding | arc-scale demo: migrate → stamped nodes; a v0.1 legacy read; `check` flags a wrong-type field | serious | ODD-0020 / arc-plan v1.5 | open | | reproduce at arc scale. **Class-(b) stable ID** — the schema-versioning capability (slice03). |

Closes in `arc06-migrate-self-host/closing-report.md`: per-row walk + composition verdict,
independently gated. A failed class-(b) row spawns a **remediation slice**, not a re-pass.
**On A-8 reproduced, the v1.0.0 MVP-plus is self-hosting.**

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
- **Idempotence key (resolved, slice01).** ULID identity is fresh/random, so a re-run
  cannot re-mint the same id — idempotence is a **describe-or-create keyed on the preserved
  legacy `number`**: `migrate` sets the new node's `number` = legacy number (identity = fresh
  ULID), and **skips** any legacy doc whose `number` already exists as an `odd` node. (Open
  sub-question for slice01/02: whether `odd` numbers share the work-node numbering space or a
  separate one — settle against the real corpus.)
- **DocState → gate mapping (slice01).** The legacy `state` scalar (`Draft`/`Under-review`/
  `Revised`/`Accepted`/`Active`/`Final`/`Deferred`/`Rejected`/`Withdrawn`/`Superseded`, from
  the `01-draft`…`10-superseded` dirs) maps to the **`odd` gate-set position** (defined in the
  gate config). slice01 pins the exact per-state gate mapping against the `odd` gate-set;
  the state-*directory* itself is dropped (was redundant truth).

## Carry-ins from A5 (project-plan v1.6) + git

- **`CLAUDE.md` oxur-cli doc-drift → folds into slice05** (retire/reconcile prose): the
  project `CLAUDE.md` claims "CLI output uses `oxur_cli::common::output`/`oxur_cli::table`,"
  but `odm-cli` has **no** `oxur-cli` dependency (it uses `tabled` + `writeln!`). A doc-keeping
  fix — reconcile it while retiring redundant prose in slice05 (or a quick standalone fix
  sooner). Recorded so it isn't lost.
- **Two-reads-per-bare-command optimization → NOT an A6 concern** (it's an `odm-reconcile`
  perf follow, orthogonal to migrate/self-host/PM-skill). **Parked as a post-MVP backlog
  item**, not forced into this arc. Flagged here so the disposition is explicit.
- **Git:** A6 branches cut from **`release/1.0.x`** (not `main` — `main` is reset onto the
  pre-rebuild `release/0.3.x` import). cc-prompts say "branch off `release/1.0.x`."

## Method

Ledger per slice; CC implements, CDC verifies; cargo rows via CI / local 1.85+;
five-iteration cap. Slice closes bubble up to this arc-plan; the arc closes with its own
`closing-report.md` + composition check. **On arc close, the v1.0.0 MVP-plus is
self-hosting** — and per the project-plan, the A7/A8 telemetry/forecasting horizon
becomes scopable.

## Version History

### v1.5 — 2026-07-06
**Schema-versioning folded in (ODD-0020) — a new slice03 inserted before self-host.** Operator
decision (Duncan + CDC): version the file/frontmatter metadata schema with **per-type markers**
(`schema: <type>/vN.N` — reusing the `check/v1` idiom), current = **v1.0**, unversioned legacy
⇒ **v0.1** (design-docs-only). Captured as **ODD-0020**. Folded into A6: **slice03 = schema
versioning** (the marker + per-type field-validity as a `check` extension + `migrate` stamps
`v1.0` / reads legacy as `v0.1`), sequenced **before** the self-host cutover so the plan-set
enters `nodes/` already at `v1.0`; **self-host → slice04, PM-skill → slice05, retire →
slice06**. Ledger reconciled by the **stable-ID discipline** (don't disturb rows with
content): the empty placeholders A-3/A-4/A-5 relabeled (schema/self-host/PM), slice06-retire
appended as stable **A-11** (class-a), the schema compose row appended as stable **A-12**
(class-b) — A-1/A-2 + the migrate/check compose rows (A-6/A-7, real evidence) untouched.
class-(a) = A-1…A-5 + A-11 (6 slices); compose = A-6–A-9 + A-12; bubble-up A-10. Surfaced by:
the operator's schema-versioning proposal + its intersection with the imminent self-host.

### v1.4 — 2026-07-06
**slice02 closed (A-2 attested; A-7 mechanism-complete) + bubble-up propagated —
the importer met reality; odm's own ODDs are migrated.** `odm migrate docs/design`
imported odm's real ODD corpus (12 ODDs, numbers `2`/`9`–`19`) into 12 `odd` nodes
committed under `nodes/2026/07/`, with the legacy `docs/design` untouched
(supersede-not-delete) — the first real step of self-hosting. **`odm check` is
green** on the imported graph ("ok, 12 nodes"), and green *by construction*:
document nodes are orphan-exempt and non-parent-capable, and the imported nodes
carry no edges/affects/deferred, so no findings arise — no fabricated edge, no
forced green. **The three slice01 flags settled against reality, none forcing an
amendment:** (1) `odd` is a **distinct numbering space** — idempotence keys on
`(type=odd, number)`, no collision on the real corpus; (2) the real `supersedes`
shape is `null` throughout, and the parser was hardened (`de_opt_ref`) to also
accept a bare number and a `"ODD-00NN"` string ref (future-proof, with a fixture);
(3) **no multi-supersession** occurs → the single-`supersedes`-edge model (with
warn-on-multiple) is kept. No `07-deferred` ODD exists, so **`deferred → retire`**
stands as the interim mapping (revisitable). Idempotent + `--dry-run` hold at real
scale (a re-run creates 0). oxur's `crates/design/docs` is **decided out of scope**
(odm's own docs first). **No `odm-index` change.** 6/6 attested; clippy clean; no
`unsafe`; `odm-migrate` line cov ≥ 93.8%; full workspace green. **Bubble-up for
slice03 (the plan-set cutover):** `odd` + work nodes will coexist in one store —
keep the numbering spaces separated and confirm `check`/`orient` handle the mixed
corpus (the document-node orphan-exemption is what makes it green); the
**reflexive-import ordering** is the crux (import the plan describing the migration
*last*; lean on `--dry-run` + a git checkpoint); legacy retirement becomes a live
post-cutover question (both representations now coexist). *Plan-keeping:* CC
propagated this bubble-up here itself. Surfaced by: slice02 close.

### v1.3 — 2026-07-06
**slice01 closed (A-1 attested; A-6 mechanism-complete) + bubble-up propagated.**
The `migrate` importer core landed: a new `odm-migrate` crate + `odm migrate
<legacy-path> [--dry-run]` command that maps the legacy number-/state-directory
model onto the node model — `number` → fresh ULID (legacy number preserved and
used as the idempotence key), `state` → a **cumulative** `odd` gate reach (a
`Final` doc reaches `draft…final`) or, for `deferred`/`rejected`/`withdrawn`/
`superseded`, a **retirement** marker (supersede-not-delete); `supersedes`/
`superseded-by` → a single `supersedes` edge on the superseding node (ids resolved
via a `number → id` map shared with the idempotence check); title/tags/component/
created/updated carried, `author` carried into the frontmatter `extra` catch-all
(no typed field). Idempotent (re-run creates 0), `--dry-run`-able (writes nothing),
never-delete (proven by a copy-then-byte-snapshot test), and loud on malformed
(missing number / unknown state → reported skip; dangling supersedes → warning +
node still imported; bad frontmatter → skip, never a panic). Tested on synthetic
fixtures (`test-data/legacy` + `test-data/legacy-edge`), **not** `docs/design`
(slice02). **No `odm-index` change.** 7/7 attested; clippy `-D warnings` clean; no
`unsafe`; `odm-migrate` line cov ≥ 91% (lib 99.56%). **Flagged decisions (candidate
amendments, not blockers):** cumulative gate reach at `Asserted`; `deferred` grouped
with the dustbin as a retirement (a first-class `deferred` disposition is the
amendment); `author`→`extra` (a typed `author` is the amendment); single
`supersedes` edge per node (>1 → warn). **Bubble-up for slice02:** settle the
odd-vs-work numbering space (recommend `(type=odd, number)` as the key — already
what the code checks); confirm the real `supersedes` value shape (may not be bare
numbers); check for multiple supersessions per node. *Plan-keeping:* CC propagated
this bubble-up here itself. Surfaced by: slice01 close.

### v1.2 — 2026-07-06
**A6 activated (the final v1.0.0 arc); slice01-relevant questions resolved (plan-deepening).**
A6 is the active arc after A5's CI-green close. Per *plan late, plan deep*, deepened as
slice01 is drawn: resolved the **idempotence key** (describe-or-create on the preserved
legacy `number`; fresh ULID identity; skip-if-exists) and the **DocState → `odd` gate
mapping** (state scalar → gate position; state-dir dropped). Grounded the legacy model in
odm's own `docs/design` (12 ODDs, state-dirs `01`…`10`, frontmatter `{number, state,
supersedes, superseded-by, …}`) → `NodeType::Odd` nodes; `odm-migrate` doesn't exist yet
(slice01 creates it). Added a **Carry-ins from A5** section: the `CLAUDE.md` oxur-cli
doc-drift folds into slice05; the two-reads optimization is parked (non-A6); A6 branches off
`release/1.0.x`. No structural re-break; the 5-slice breakdown holds. Surfaced by: drawing up
slice01 / the A5 close bubble-up.

### v1.1 — 2026-06-26
Added the **`## Arc Ledger`** section per LEDGER-DISCIPLINE v2.0 §B (the arc ledger opens
with the arc-plan, which already exists). Pure addition — the v1.0 body is unchanged.
Surfaced by: the ledger-discipline upgrade (v1→v2.0), not a slice bubble-up.

### v1.0 — 2026-06-26
Initial arc-plan, drafted from ODD-0013 §9/§11 + the ODD-0015 A6 row + the ODD-0001
PM-skill seed, as part of the project-plan synthesis session. No slices started;
one-line altitude per *plan late, plan deep*.
