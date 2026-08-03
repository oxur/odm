# Arc — Store-as-source-of-truth & native authoring

<!-- Name/title carries no document-role metadata, per ODD-0013 §2.1. -->

**Opened:** 2026-08-02 · **Status:** slice 01 CLOSED -- ODD-0026 Accepted (2026-08-03); slice 02 CDC-verified PASS; slice 03 (native authoring -- programmatic surface) CDC-verified PASS (2026-08-03); slice 04 (cutover) next ·
**Unnumbered**, per the `arc-release-hardening` / `arc-migration-fidelity` /
`arc-llm-command-surface` precedent (the numbering scheme is under review in RH).

> **Authored the old way, on purpose.** This arc's own planning docs (this
> `arc-plan.md`, its slice docs, and ODD-0026) are authored as markdown under
> `./docs` and migrated into the store — because the capability that would let us
> author them *in* odm is exactly what this arc builds. Chicken-and-egg,
> disclosed. It is the last arc that has to be authored this way.

## Why this arc exists (the training wheels)

odm's model today is **`./docs` = source of truth, the store = a verified 1:1
mirror.** Every planning node carries `source.paths` pointing at a `./docs` file
(confirmed: the live LLM-arc node has `source.paths:
docs/design-v1.0.0/arc-llm-command-surface/arc-plan.md`, `class: arc-plan`), and
ODD-0025's hard body-hash gate proves the store body matches that source. There is
**no command to author or edit a node body** — `node new` takes only
type/name/parent (no body); the CLI's own error affordances say *"edit {file}"*.
Bodies enter the store one way: `migrate` copies them from an external `./docs`
file.

So `./docs` is the training wheels. Deleting the planning tree today would dangle
every planning node's source and remove the only authoring surface. This arc takes
the wheels off cleanly: **the store becomes the source of truth for planning, odm
gains a native authoring path, and `./docs` is freed for conventional end-user
documentation** (the usual library/tool pattern).

This is also what the 2026-08-02 `odm check` output is telling us from another
angle: arc nodes read as `undeveloped-stub` (advanced past planning with no minted
slice children) and MF shows `decomposition-drift` because `migrate` re-mints the
child set. Those are symptoms of the docs-as-source model — slices live as docs
that may or may not be minted as nodes, and migration churns the tree. Native
authoring makes minting a slice node a first-class, non-drifting operation.

## Capability

A planning node — project, arc, slice, or the artifacts under a slice — can be
**created, given a body, updated, and managed entirely through `./bin/odm`**, with
the **store node file as its source of truth** (no external `./docs` dependency).
`check` / `reconcile` / `migrate` treat planning nodes as *self-sourced*: the
body-hash fidelity gate, which exists to prove faithful migration *from an external
source*, no longer applies where there is no external source. When this arc
closes, the `./docs` planning subtree can be deleted and `odm check` stays green,
with `./docs` holding only end-user documentation.

**Design basis:** **ODD-0026** (store-as-source model — slice 01), generalizing
**ODD-0025**'s existing exemption (the project synthesis node and retired nodes
already carry no external 1:1 source) from two special cases to *all authored
planning nodes*. Reconciles with **ODD-0013** (identity/naming) and **ODD-0017**
(projection-out — the reverse direction: this arc is about *authoring in*, ODD-0017
is about *rendering out*).

**Explicitly not in scope:** the LLM-command-surface arc's read/query work (that
arc is independent and may proceed in parallel); telemetry/forecasting (A7/A8);
any change to end-user-doc *content*.

## Dependencies

- **Consumes:** the fully self-hosted store after the 2026-08-02 safety
  `migrate --all` (slice16 + Store-Lifecycle nodes now minted; drift reconciled).
- **Unblocks:** deletion of the `./docs` planning subtree (this arc's slice 04);
  and, going forward, authoring *every* future arc/slice in odm rather than the old
  way — including the LLM-command-surface arc's own slices.
- **Adjacent (coordinate):** LLM-arc **slice 09** (frontmatter-fidelity check) and
  **slice 10** (name/title normalizer) both touch `migrate`/`check`; this arc
  changes what `check`/`migrate` expect of a planning node's `source`. Land the
  model (slice 01) before those slices harden their `check` rules, so they build on
  the settled contract.

## Slice breakdown (shaped, plan-late)

| Slice | Scope | Kind |
|---|---|---|
| **01 · ODD-0026 — the store-as-source model** | Resolve the four open design forks (A/B/D/E; C is settled) and author + accept **ODD-0026**. Design/decision unit; the deliverable is the accepted ODD, from which slices 02–04 derive their acceptance criteria. *Authored by CDC + operator (a decision, not an implementation).* | design |
| **02 · self-sourced planning nodes** | Make `migrate` / `reconcile` / `check` honor a planning node that has **no external source**: no `undeveloped`/`missing-source`/body-hash error for a store-authored node; convert the existing planning nodes to self-sourced per ODD-0026; keep schema/edge/decomposition/order validation intact. The core enabler — after this, the store *is* authoritative. **CC-closed 2026-08-03** (`origin: authored` + `Source::authored` + `Violation::InconsistentAuthoredSource`; `self_host`/`reconcile` never churn an authored node; `migrate --to-authored` — explicit, one-time, deliberately not part of `--all`; 9/11 ledger rows done, 2 attested (ODD amendment stubs not yet folded in); real-corpus leg deferred to CDC). | code |
| **03 · native authoring commands** | `odm node new` gains a body path (`--from-file <md>` and/or `--body`, and/or `$EDITOR`); `odm node edit <ref>` opens the store node for editing; child/slice minting is a clean one-command operation (closing the `undeveloped-stub` gap). "Edit the store file + `odm check`" remains the always-works fallback. **Scoped 2026-08-03 to the programmatic surface** (`node new`/`node set`/`node set-body` via files+flags -- the automation path); interactive `$EDITOR` split + section-anchored edits -> slice 07. **CC-closed 2026-08-03**: the metadata partial (JSON/TOML, `deny_unknown_fields` enforces the author-vs-odm boundary by the type's own shape); `node new` now mints every node `origin: authored` (not just body/metadata-carrying ones); `node set`/`node set-body` new; 10/10 ledger rows done (2 with a disclosed real-corpus-leg caveat). | code |
| **04 · cutover** | After a final safety `migrate --all` + green `check`: delete the `./docs` planning subtree (git-recoverable), keep `./docs` for end-user docs, and update project memory, `CLAUDE.md`, and CC's settings to the odm-native authoring workflow. Gated on 01–03 and an explicit operator go. | code + docs |
| **05 · decomposition bookkeeping: consistency fix + deterministic auto-recompose** | Two coupled defects the 2026-08-02 store surfaced. **(a) Consistency bug:** `node decomposed` affirms *all* reverse-`part_of` children (incl. artifact/note nodes) but `check`'s decomposition-drift computes a *different* current-child set (excludes them) — so an affirmed parent reports a permanent phantom "removed N" that re-affirming cannot clear (MF `#58837400`: the 2 non-slice children `536513400`/`560811200`). Pick **one** child-set definition and use it in both `node decomposed` and `check`. **(b) Auto-recompose:** when `migrate` churns children deterministically and the resulting set is *provably the same* as the prior affirmed set (re-mint / re-read, no membership change), it re-affirms automatically — the deterministic bookkeeping odm exists to kill, not to nag the user with. A *genuine* membership change (a real new/removed child) still surfaces for the human scope-completeness judgment (the mechanical half auto-heals; the judgment half does not). | code |
| **06 · auto-extend affirmed decomposition on authored additions** | Resolves **SS5-1** — the follow-up to slice 05. In odm's model **the plan tree declares scope**, so an *already-affirmed* parent that gains a plan-tree-declared slice should have its decomposition **auto-extended** by `migrate --all` — no manual `node decomposed`. Rule (extends slice 05's `decompose` pass): current work-children ⊇ affirmed (additions only) → auto-extend (also auto-heals the MF transitional case); a work-child **removed** → still `LeftAsDrift`; a **never-affirmed** parent is never auto-affirmed (first affirm stays a human act). Uses the model-independent additions-only signal so it lands before ODD-0026. | code |
| **07 · interactive authoring ergonomics** | `odm node edit <ref> --body`/`--meta` (open ONLY the stripped body, or ONLY the metadata partial, in `$EDITOR`; odm re-attaches the seam on save) + `node edit <ref> --section "## Heading" --from-file` (heading-anchored partial body edits). Human ergonomics layered on slice 03's programmatic surface; needs an editor-spawn dependency + markdown-section parsing. **Plan-late** -- off the SS-4/SS-6 critical path; split from slice 03 on sizing (v1.11), drawn when near. | code |

**Sequencing (updated — operator priority 2026-08-02): `05 → 06 → 01 → 02 → 03 → 04`.**
Slice **05 ran first** (independent of the model; cleared the consistency bug).
**06 runs next** — before the store is re-migrated + committed — so `migrate --all`
becomes hands-off (no manual `node decomposed` for authored additions), which is the
whole point of the re-migrate. The rest keep their order (`01 → 02 → 03 → 04`), each
load-bearing for the next. (05/06's numbers are higher than all of them by the
append convention, but their order is set here, not by the number — odm's own
*identity ≠ order* principle, dogfooded: the first two slices we run are #05 then
#06.) 04 is the point of no return (though git-recoverable) and does not run without
a green final migrate and your explicit go.

**Slice 05: CDC-verified PASS (with notes) — 2026-08-02.** Consistency fix (a)
**reproduced** — MF `#58837400` drift cleared (affirmation now the 16 work slices,
0 drift); the seam (F-5) is correctly implemented. Auto-recompose (b) is delivered
and correct-by-design but **inert on real data** — migrate preserves ids, so the
identity-churn trigger never fires (**SS5-1**, elevated to slice 01 / ODD-0026). Two
caveats: the code + the MF affirmation are **uncommitted** (attested → CI), and the
`odm` store worktree needs a deliberate reconcile before proceeding (**SS5-2**). See
`slice05-decomposition-bookkeeping/cdc-verification.md`.

**Slice 06: CDC-verified PASS — 2026-08-02** (`slice06-auto-extend-decomposition/cdc-verification.md`; code `7cd5e01`). Resolves SS5-1: an already-affirmed parent auto-extends on authored additions (also auto-heals the MF transitional shape), while removals and never-affirmed parents stay conservative. The sharp edge — a *vanished* work-child vs a *present non-work* id — is correctly distinguished (CC caught it), so removals never get swallowed. Structural rows reproduced; **F-2's real leg is now CLOSED — reproduced on the real corpus (2026-08-02):** `migrate --all` auto-extended MF hands-off (`RECOMPOSE`: `0 re-affirmed, 1 auto-extended, 0 left as drift`), `odm check` **0 errors**, only benign `undecomposed-parent` warnings remain. SS6-1: additions-only blesses any addition in migrate context (accepted; → ODD-0026).

**Sizing:** 01 is a design slice (its diff is ODD-0026). 02 and 03 are medium
(distinct subsystems: the check/migrate/reconcile contract vs. the CLI authoring
surface). 04 is small-but-careful (a deletion + config, with a full verify). 05 is
medium — the consistency fix is small once the one child-set definition is chosen;
the auto-recompose (with the provably-same-set guard) is the substance. 06 is small —
one new outcome on slice 05's existing pass.

## Arc ledger (opening / composition rows)

Per `LEDGER-DISCIPLINE.md` §B — the class-(b) "slices compose into the capability"
rows, stated up front from the capability statement. Class-(a) per-slice-closed and
class-(c) bubble-up rows accrue as slices close.

| ID | Criterion | Verify | Sev | Status |
|---|---|---|---|---|
| **SS-1** | ODD-0026 accepted, recording the model + the fate of `source.paths` and the body-hash gate for planning nodes | `docs/design/04-accepted/0026-*.md` present, status Accepted | serious | **done** (2026-08-03, slice 01) |
| **SS-2** | A planning node with **no external source** passes `check` (no `undeveloped-stub` / missing-source / body-hash error) | fixture: an authored node, `check` clean | serious | **done — fixture + real end-to-end CLI test** (slice 02, 2026-08-03) |
| **SS-3** | Existing planning nodes converted to self-sourced per ODD-0026; a full `migrate --all` + `check` is green with the `./docs` plan tree still present | `migrate --all` then `check` exit 0 | serious | mechanism **done**, fixture-proven; **real-corpus leg deferred to CDC** (slice 02 — `.worktrees/odm` not checked out in the implementation worktree) |
| **SS-4** | A node body can be **created and updated via `./bin/odm`** (no hand-edit of store files required) | `node new --from-file` + `node edit` demonstrated; the created/edited body reads back via `node show` | serious | **done (programmatic surface) — fixture + real end-to-end CLI round-trip** (slice 03, 2026-08-03); **real-corpus leg CLOSED 2026-08-03** -- live `node set-body` on bootstrap #543468700, `store commit` 701668b, `check` 0 errors |
| **SS-5** | The body-hash fidelity gate **still applies to genuinely-migrated content** (end-user/legacy docs with an external source) — no regression | fixture: a migrated node with a corrupted body still fails `check` | correctness | **done — regression-fixtured** (slice 02, 2026-08-03) |
| **SS-6** | **DoD — reproduced at arc scale:** a fresh context authors a new arc + slice end-to-end using only `./bin/odm`, with no `./docs` planning tree and no hand-edited store files | demonstration transcript | **serious** | open |
| **SS-7** | With the `./docs` planning subtree deleted, `check` / `orient` / `list` / `rollup` are green and unchanged; end-user `./docs` is untouched | delete on a branch, `check` exit 0 | serious | open |
| **SS-8** | Decomposition bookkeeping is deterministic: (a) `node decomposed` and `check` use **one** child-set definition — a parent whose current children set-equal its affirmed set reports **0** drift (MF `#58837400` clears); (b) `migrate` auto-recomposes provably-identity child churn with no manual re-affirm | fixture: the MF artifact-child case → 0 drift after affirm; a re-mint `migrate` leaves `check` green with no manual step | serious | (a) **done** — slice 05 CDC-verified; (b) delivered, inert (SS5-1) |
| **SS-9** | `migrate --all` is **hands-off** for authored additions: an already-affirmed parent gaining a plan-tree-declared work-child is **auto-extended** (no manual `node decomposed`), while a work-child *removal* and a *never-affirmed* parent still behave conservatively | fixture set + real: reset → `migrate --all` → MF 0 drift with no manual step | serious | **DONE — reproduced on the real corpus** (2026-08-02): `RECOMPOSE` = `0 re-affirmed, 1 auto-extended, 0 left as drift`; `odm check` 0 errors |

**SS-6 is the arc's reason to exist** and, like every DoD row, is closed by a
fresh/independent context, not by the implementer.

## The design forks (resolved in slice 01 / ODD-0026)

Stated here so the arc's design basis is visible; slice 01 carries the detail and
my recommendations. Four were open choices (A, B, D, E), now **resolved in ODD-0026 (Accepted)**; **C is already settled** and
is listed only to make the cutover scope explicit.

- **A -- `source.paths` for planning nodes. RESOLVED (ODD-0026 sec. 2.1): KEEP provenance.**
  Authored nodes carry `origin: authored` + a no-external-path `source` (`class: authored`); migrated
  nodes keep the full `source` record. ~~Earlier lean: drop it~~ **overturned** -- `source` is enduring
  provenance, not migration scaffolding, and the schema is clarified so "authored" reads as a value.
- **B — the body-hash gate.** Applies only to migrated nodes (external source) and
  is N/A for authored nodes, vs. kept as a self-checksum. *Lean: N/A for authored*
  — the gate proves faithful *migration*; there is nothing to migrate. Git is the
  tamper record for the store branch.
- **C — the design corpus (ODDs 0011–0025). SETTLED, not a fork.** They are
  design/planning artifacts already migrated into the store (repeatedly), so they
  are store-as-source like every other planning node; `docs/design/` deletes in the
  cutover (slice 04). Listed only so the cutover scope is explicit.
- **D — end-user `./docs`.** Outside the odm planning store entirely (conventional,
  versioned with code) vs. tracked as a separate non-planning corpus. *Lean:
  outside* — that is the point of freeing `./docs`.
- **E — the authoring/update contract (drives slice 03).** *Lean:* `node new
  --from-file` (you still write markdown; odm ingests it as the body and owns
  placement/frontmatter/identity) + `node edit <ref>` (`$EDITOR` on the store
  file). The human ergonomics barely change; the source of truth moves into the
  store.

## Version History

### v1.13 -- 2026-08-03 (slice 03 CDC-verified PASS)

CDC-verified slice 03 PASS (`slice03-native-authoring/cdc-verification.md`). Structural rows reproduced by reading CC's working-tree code (uncommitted/staged; HEAD is the drawing commit `3f527dc`): the metadata partial + the boundary enforced by the type's shape (`deny_unknown_fields`, no field for odm-owned keys) -- stronger than a validation pass; `node new` minting `Origin::Authored` + verbatim body; `node set` gating on `SETTABLE_FIELDS`; `node set-body` preserving the frontmatter (only `updated` bumps). F-7 SS-4 round-trip read in full (`new -> set -> set-body -> show`, `check` green throughout, no hand-edit). The two self-caught regressions verified as correct contract updates (planned origins now seeded directly), not weakened tests. SS-4 done (programmatic surface). Carried: the `status`-intent = `Asserted` interpretation is reasonable + flagged, worth a one-line confirm in ODD-0026 sec. 2.5; F-3/F-7 real-corpus leg deferred to the operator's rebuilt binary. Surfaced by: slice 03 CDC verification. **Next:** slice 04 (cutover), on operator go.

### v1.12 -- 2026-08-03 (slice 03 CC-closed: programmatic native authoring implemented)

CC implemented slice 03 — the programmatic authoring surface. `odm-cli/src/metadata.rs`:
`MetadataPartial { part_of, tags, status }`, dual JSON/TOML I/O into one canonical form, the
author-vs-odm boundary (ODD-0013 v2.6) enforced **by the type's own shape** (`deny_unknown_fields`
— no field for any odm-owned key exists to be set, correctly or otherwise). `node new` now mints
**every** node `origin: authored` + `Source::authored` (slice 02's model, reused unmodified) — not
conditionally on body/metadata being given — gains `--metadata`/`--content`/`--from-file`/`--body`/
`--json`; a bare `node new` mints an authored stub. New `node set <ref> <field> <value>`
(`name`/`tags`/`part_of`/`status`; `status` reuses `Status::set_gate` at `Asserted` evidence rather
than inventing a second status model — flagged as an interpretive choice for ODD-0026 §2.5's
under-specified "status intent"). New `node set-body` (whole-body replace, frontmatter untouched
except the `updated` bump every other mutator already does). SS-4 proven end-to-end
(`new` → `set` → `set-body` → `show`, `check` green throughout, no hand-edit) — fixture-level;
the real-corpus leg deferred to CDC (`.worktrees/odm` not checked out here, the standing blocker).
**Two regressions self-caught via the full test suite, not by CDC**: `node new`'s unconditional
`origin: authored` broke a pre-existing JSON-shape test and a pre-existing rollup-provenance
fixture that both drove the *real* `node new` command expecting `Origin::Planned` — both updated to
the new, correct contract. 10/10 ledger rows done. See
`slice03-native-authoring/{ledger,closing-report}.md`. Surfaced by: CC (this session). **Next:**
slice 04 (cutover) — gated on slice 03 (done) and an explicit operator go; slice 07 (interactive
authoring ergonomics) remains open, plan-late, off the SS-4/SS-6 critical path.

### v1.11 -- 2026-08-03 (slice 03 open set drawn; scoped to the programmatic surface; slice 07 split out)

Slice 03's open set is drawn (`slice03-native-authoring/`). Planning it against ODD-0026 sec. 2.5's full fork-E contract surfaced a **sizing split**: the contract spans two distinct concerns -- (a) the **programmatic** surface (`node new --metadata/--content`, `node set`, `node set-body` via files + flags; the canonical metadata partial with dual JSON/TOML I/O; validate-before-write; the author-vs-odm boundary from ODD-0013 v2.6) and (b) the **interactive** surface (`node edit --body/--meta` in `$EDITOR`, `node edit --section` heading-anchored edits). The programmatic half is the automation/LLM path -- it is what lets odm author its own docs and what slice 04's cutover depends on, and it closes **SS-4** (create AND update, no hand-edit) and enables **SS-6** on its own. The interactive half is human ergonomics, needs a new editor-spawn dependency (none present today) + markdown-section parsing, and is off the SS-4/SS-6 critical path. Keeping both in one slice risked overflowing a single context (PROJECT-MANAGEMENT Part I sizing). **Slice 03 is scoped to (a); (b) splits out to a new plan-late slice 07** (interactive authoring ergonomics), drawn when near. Grounded in the current CLI (`node new` takes only type/name/parent and mints `Origin::Planned`; no `set`/`edit`/`set-body` yet; `serde_json`+`toml` already deps). No re-sequencing of 03 -> 04. Surfaced by: slice 03 planning + the accepted ODD-0026 fork-E full surface. **Next:** CC implements slice 03.

### v1.10 -- 2026-08-03 (slice 02 CDC-verified PASS; ODD amendments folded in)

CDC-verified slice 02 PASS (`slice02-self-sourced-nodes/cdc-verification.md`) -- structural rows reproduced by code read at `803c738`; the F-7 no-regression test read in full (a drifted migrated node still hard-fails alongside an untouched authored sibling); the `InconsistentAuthoredSource` consistency check confirmed wired into the CLI (`commands.rs:1760`). Two things stronger than the ledger asked: the check is bidirectional (not a suppression), and CC surfaced + disclosed the `--to-authored`-vs-`--all` workflow hazard (converting a node freezes its `./docs` edits). **F-8/F-9 folded in (now done):** the ODD-0013 amendment landed in `docs/design/04-accepted/0013-odm-architecture-design.md` (now **v2.6**: `origin: authored`, the authored `source` shape, the author-vs-odm field boundary) and the ODD-0025 amendment in `0025-migration-fidelity-model.md` (now **v1.4**: §2.0 authored point, §2.1 gate-scope-narrowed, §2.2 authored source). Still open: the `--to-authored`/`--all` separation is model-adjacent and lives only in code + slice docs (recommend a note in ODD-0026 when convenient); and F-6/SS-3's real-corpus leg (operator runs `migrate --to-authored --dry-run` -> real -> `migrate --all` -> `check` on the live store). Surfaced by: slice 02 CDC verification + amendment fold-in. **Next:** slice 03 (native authoring commands, SS-4).

### v1.9 -- 2026-08-03 (slice 02 CC-closed: self-sourced planning nodes implemented)

CC implemented slice 02 — **the core enabler**: an authored node (`origin: authored`, `source: {
class: authored }`, no external paths) is now a first-class, checked, migrate-safe citizen of the
store. Schema: `Origin::Authored`; `Source`'s migration-only fields became optional (`None` on an
authored node); `source.migrated_from` preserves a converted node's provenance; a new structural
check (`Violation::InconsistentAuthoredSource`) enforces the two-way `origin`/`source.class`
agreement. `self_host` gained a fourth identity-matching tier (`by_coordinate_authored`) — without
it, an authored node's empty `source.paths` made it invisible to the existing matching, so the
first `migrate --all` after this landed would have **re-minted every authored node as a duplicate**
the moment its still-present `./docs` file was seen; caught by reading the matching logic before
writing code, not after. `reconcile` gained an explicit authored-skip; `mapping.rs` (design/
research) needed no change, confirmed safe by construction. **`odm migrate --to-authored`**: a new,
explicit, `--dry-run`-able, idempotent conversion pass for the existing corpus (sub-decision (i) —
re-classify + preserve provenance) — **deliberately not folded into `--all`**, since converting a
node stops further `./docs` edits from reaching it (self-sourced now), a real workflow change the
operator should trigger knowingly, not absorb as a side effect of their ordinary `migrate --all`
habit. **F-7 no-regression fixtured**: a genuinely-migrated node with a drifted body still hard-fails
`repair`, proven alongside an untouched authored sibling in the same run. ODD-0013/ODD-0025
amendment stubs written in the slice directory (F-8/F-9), code-complete but not yet folded into the
accepted ODDs — a separate operator/CDC action. **9/11 ledger rows done**, 2 attested (the
amendments); the real-corpus leg of SS-3 (F-6) is **deferred to CDC** — `.worktrees/odm` is not
checked out in the implementation worktree, the same blocker every slice this session has hit. One
self-caught iteration: the first version of the new check ran over index-backed frontmatters that
don't carry `source` at all (caught by a real end-to-end CLI test, not a unit test) — moved to
`content_validity`, the store-loaded pass `check_source_paths` already uses for the identical
reason. Full workspace `make check` green. See `slice02-self-sourced-nodes/{ledger,closing-report}.md`.
Surfaced by: CC (this session). **Next:** slice 03 (native authoring commands) — SS-4.

### v1.8 -- 2026-08-03 (slice 01 CLOSED; ODD-0026 Accepted; SS-1 done)

**Slice 01 closed.** ODD-0026 was accepted by the operator and promoted to `docs/design/04-accepted/0026-store-as-source-model.md` (v1.1, `state: Accepted`); the slice's close set (`closing-report.md`, `cdc-verification.md`) is written and its ledger walked (D1-1 `reconciled`, D1-2..D1-9 `attested`). Arc ledger **SS-1 -> done**. One plan change bubbled up (recorded in v1.7 and applied here): **fork A was corrected** -- the "## The design forks" section's stale "*Lean: drop it*" is replaced with the resolved decision (KEEP `source`/provenance; authored = `origin: authored`, no external path). ODD-0026 sec. 3 adds two deliverables to **slice 02**: the ODD-0013 schema amendment (`origin: authored`, the authored `source` shape, the author-vs-odm field boundary) and the ODD-0025 amendment (authored nodes bypass the migration body-hash gate) -- both now in slice 02's open set. No re-sequencing: `02 -> 03 -> 04` stands. Surfaced by: slice 01 close. **Next:** slice 02 (self-sourced planning nodes) -- open set drawn.

### v1.7 — 2026-08-03 (slice 01 opened; ODD-0026 authored as Draft; forks A-E resolved, A corrected)

**Slice 01's open set was completed and ODD-0026 authored** at `docs/design/01-draft/0026-store-as-source-model.md` (Draft; decisions ratified by the operator 2026-08-02/03, prose pending acceptance). All five forks are now recorded. **Fork A was CORRECTED:** the earlier "drop external source" lean (A1) was overturned by the operator -- `source`/provenance is KEPT as enduring metadata; an authored node carries `origin: authored` + a no-external-path `source` (`class: authored`), and the node schema gains explicit "authored is a provenance *value*, not the absence of the field" language so the misread cannot recur. B (body-hash N/A for authored, migration gate unchanged), C (design corpus settled), D (end-user `./docs` outside), E (odm owns the seam; canonical metadata partial with dual JSON/TOML I/O; author-vs-odm field boundary; validate-before-write; create + channel-addressed-edit surface). ODD-0026 sec.3 specifies the amendments this arc's later slices apply: ODD-0013 (add `origin: authored`, the authored `source` shape, the boundary) and ODD-0025 (authored nodes bypass the migration body-hash gate by construction). Slice-01 `ledger.md` rows D1-2..D1-9 are now `attested` (recorded in the ODD); **D1-1 stays open** until the operator accepts ODD-0026 -- on acceptance it promotes to `04-accepted/`, D1-1 closes `reconciled`, and slice 01 closes. Authored in the plan tree per the operator ("do this in the old place; hard switch after native authorship lands") -- ODD-0026 migrates into the store on the next `migrate --all`. Surfaced by: slice 01. **Next:** operator accepts ODD-0026, then slice 02 (self-sourced planning nodes).

### v1.6 — 2026-08-02 (SS-9 closed: hands-off migrate reproduced on the real corpus)

The acceptance anchor met on the live store. A real `reset → migrate --all` (rebuilt
binary) auto-extended MF with **no manual `node decomposed`** — `RECOMPOSE`:
`0 re-affirmed, 1 auto-extended, 0 left as drift` — and `odm check` returned **0
errors** (10 benign `undecomposed-parent`/`undeveloped-stub` warnings, all
legitimately-open decomposition states left un-affirmed on purpose). F-2's real leg
closes; **SS-9 done**. Slices 05 + 06 are both live-proven; `migrate --all` is now
fully hands-off for authored additions. Store is clean, attributable, ready for
`odm store commit`.

### v1.5 — 2026-08-02 (slice 06 CDC-verified PASS)

CC implemented slice 06 (`7cd5e01`); CDC-verified PASS —
`slice06-auto-extend-decomposition/cdc-verification.md`. Six structural rows
reproduced by code read (the `AutoExtended` outcome; the additions-only rewrite; the
sharp-edge `is_none_or(is_work)` that keeps a *vanished* work-child as a removal while
dropping a *present non-work* id; never-affirmed skipped; idempotent; the RECOMPOSE
output). The seam is refined not removed — CC renamed and re-asserted slice 05's seam
test to the new auto-extend behavior and added a removal companion (`node unlink
part_of`) proving removals still drift. **F-2's real leg — MF at 0 drift, hands-off,
on a real `migrate --all` — is deferred with a clear re-entry to the operator's next
migrate cycle** (no macOS binary in the CDC sandbox); it is the arc's acceptance
anchor. Finding **SS6-1** (additions-only blesses any addition in migrate context) is
an accepted property routed to ODD-0026, not a defect. Surfaced by: CC (impl +
honest deferral of the real leg + the sharp-edge catch) + CDC reproduction.

### v1.4 — 2026-08-02 (slice 06 added + drawn: hands-off migrate)

Added **slice 06** (auto-extend affirmed decomposition on authored additions) and
ledger row **SS-9**, resolving **SS5-1**. Slice 05's seam correctly refused to invent
a completeness judgment, but the operator's real friction is the genuine-addition
case, and a manual `node decomposed` after every `migrate` is the micromanagement odm
exists to kill. Slice 06 implements the resolution: in odm's model the plan tree
declares scope, so an *already-affirmed* parent auto-extends to include plan-tree-declared
additions (also auto-healing the MF transitional affirmation), while a work-child
*removal* and a *never-affirmed* parent stay conservative. Model-independent
additions-only signal so it lands **before** ODD-0026; the "authoring declares scope"
principle it embodies is folded into slice 01's ODD. Resequenced `05 → 06 → 01 → …`:
06 lands before the store is re-migrated + committed, so the re-migrate is hands-off.
CDC owns a share of SS5-1's origin — the gap was flagged in slice 05's verification but
parked as an ODD question instead of fixed; the operator called it, and it's now a
slice. Surfaced by: operator (from the live manual-affirm friction) + SS5-1.

### v1.3 — 2026-08-02 (slice 05 CDC-verified PASS with notes)

CC implemented slice 05; CDC-verified PASS (with notes) — `cdc-verification.md`.
**Reproduced:** F-1 (one shared `decomposition_children` helper, both call sites) and
F-2 (MF affirmation = the 16 work slices, 0 drift). **Attested → CI:** F-3/F-5/F-6
(tests present incl. the seam test `migrate_all_leaves_a_genuinely_new_slice_as_drift_not_auto_affirmed`;
`make check` green; no toolchain in the CDC sandbox). Two findings carried:
**SS5-1** — auto-recompose (b) is correct but **inert** (migrate preserves ids,
confirmed by the store's same-ULID churn); and the operator's real friction is the
*genuine-addition* case, which the seam correctly leaves to a human affirm — elevated
to **slice 01 / ODD-0026** as a model question (does authoring-in-scope make additions
auto-affirmable without crossing the seam?). **SS5-2** — the `odm` store worktree is
in a large uncommitted, internally-inconsistent state (13 staged-deleted-but-present
`2026/08/` nodes, 9 modified nodes, config, untracked mints) accreted from this
session's migrate runs; it needs a deliberate reconcile-and-commit (or reset +
one clean `migrate --all`) **before** more commits/migrates. Surfaced by: CC (both
findings, honestly bubbled) + CDC reproduction.

### v1.2 — 2026-08-02 (slice 05 resequenced first + drawn)

Operator priority: **slice 05 runs first** (`05 → 01 → 02 → 03 → 04`) so the
decomposition-bookkeeping fix lands and is tested in the imminent `migrate --all`
cycle. Slice 05's open set was written CC-ready (`slice05-decomposition-bookkeeping/`),
grounded in the pinned code diagnosis: `check_decomposition` (recompose.rs) filters
children `is_work()` while `node decomposed` (commands.rs) affirms all reverse-`part_of`
children — the divergence that makes MF's "removed 2" permanent. Fix (a) = one shared
work-child definition used by both paths; fix (b) = migrate auto-recompose for
provably-identity churn, with the completeness-judgment seam preserved. Surfaced by:
operator. Ledger row SS-8 (arc) → slice-05 ledger F-1…F-6.

### v1.1 — 2026-08-02 (slice 05 added: decomposition bookkeeping)

Added **slice 05** (decomposition consistency fix + deterministic auto-recompose)
and ledger row **SS-8**, after the 2026-08-02 safety `migrate --all` surfaced two
coupled defects. (a) A consistency bug: `node decomposed` affirms all reverse-`part_of`
children while `check`'s decomposition-drift excludes artifact/note children — a
permanent phantom "removed 2" on MF `#58837400` that survives re-affirming (proven
by the arithmetic: affirmed 18 = 16 slices + 2 artifacts; check's current = 16
slices). (b) The operator's point: `migrate` should auto-recompose provably-identity
child churn rather than making the user hand-run `odm node decomposed` after every
deterministic mint — with the seam that the *mechanical* child-set half auto-heals
but the *judgment* half (scope complete?) still needs a human affirm when membership
genuinely changes. Surfaced by: operator, from the MF decomposition-drift bug.
**Placement adjustable** — could instead live in LLM-arc slice 09 (check-hardening)
or as standalone odm-hardening; parked here because it is the same "mechanize the
deterministic bookkeeping" charter this arc embodies.

### v1.0 — 2026-08-02 (arc shaped)

Opened when the LLM-arc reconciliation surfaced that odm cannot author planning-doc
bodies (verified: `node new` has no body arg; error affordances say "edit {file}";
every planning node's `source.paths` points into the `./docs` tree). Operator
green-lit taking the "training wheels" off as its own arc, authored the old way.
Design-first: slice 01 is ODD-0026 (the store-as-source model), generalizing
ODD-0025's project/retired source-exemption to all authored planning nodes. Slices
02–04 implement, then cut over. Arc ledger SS-1…SS-7 opened.
