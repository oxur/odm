# Arc — Store-as-source-of-truth & native authoring

<!-- Name/title carries no document-role metadata, per ODD-0013 §2.1. -->

**Opened:** 2026-08-02 · **Status:** shaped (design-first; ODD-0026 pending) ·
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
| **02 · self-sourced planning nodes** | Make `migrate` / `reconcile` / `check` honor a planning node that has **no external source**: no `undeveloped`/`missing-source`/body-hash error for a store-authored node; convert the existing planning nodes to self-sourced per ODD-0026; keep schema/edge/decomposition/order validation intact. The core enabler — after this, the store *is* authoritative. | code |
| **03 · native authoring commands** | `odm node new` gains a body path (`--from-file <md>` and/or `--body`, and/or `$EDITOR`); `odm node edit <ref>` opens the store node for editing; child/slice minting is a clean one-command operation (closing the `undeveloped-stub` gap). "Edit the store file + `odm check`" remains the always-works fallback. | code |
| **04 · cutover** | After a final safety `migrate --all` + green `check`: delete the `./docs` planning subtree (git-recoverable), keep `./docs` for end-user docs, and update project memory, `CLAUDE.md`, and CC's settings to the odm-native authoring workflow. Gated on 01–03 and an explicit operator go. | code + docs |
| **05 · decomposition bookkeeping: consistency fix + deterministic auto-recompose** | Two coupled defects the 2026-08-02 store surfaced. **(a) Consistency bug:** `node decomposed` affirms *all* reverse-`part_of` children (incl. artifact/note nodes) but `check`'s decomposition-drift computes a *different* current-child set (excludes them) — so an affirmed parent reports a permanent phantom "removed N" that re-affirming cannot clear (MF `#58837400`: the 2 non-slice children `536513400`/`560811200`). Pick **one** child-set definition and use it in both `node decomposed` and `check`. **(b) Auto-recompose:** when `migrate` churns children deterministically and the resulting set is *provably the same* as the prior affirmed set (re-mint / re-read, no membership change), it re-affirms automatically — the deterministic bookkeeping odm exists to kill, not to nag the user with. A *genuine* membership change (a real new/removed child) still surfaces for the human scope-completeness judgment (the mechanical half auto-heals; the judgment half does not). | code |
| **06 · auto-extend affirmed decomposition on authored additions** | Resolves **SS5-1** — the follow-up to slice 05. In odm's model **the plan tree declares scope**, so an *already-affirmed* parent that gains a plan-tree-declared slice should have its decomposition **auto-extended** by `migrate --all` — no manual `node decomposed`. Rule (extends slice 05's `decompose` pass): current work-children ⊇ affirmed (additions only) → auto-extend (also auto-heals the MF transitional case); a work-child **removed** → still `LeftAsDrift`; a **never-affirmed** parent is never auto-affirmed (first affirm stays a human act). Uses the model-independent additions-only signal so it lands before ODD-0026. | code |

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

**Slice 06: CC-closed — 2026-08-02.** `AutoExtended` outcome added to `odm_migrate::decompose::auto_recompose`;
an already-affirmed parent whose current work-children are a superset of its affirmed work-children
(additions only) is rewritten to the current set, dropping any stale non-work ids in the same step (the
MF-transitional-shape auto-heal). 6 of 7 ledger rows done (module fixtures + real `odm-cli` `migrate --all`
CLI-pipeline tests); F-2's real-store leg (MF `#58837400` at 0 drift) deferred to CDC — see
`slice06-auto-extend-decomposition/closing-report.md`. Full workspace `make check` green. It resolves SS5-1
and makes the re-migrate hands-off, pending CDC's independent reproduction + the real-store confirmation.

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
| **SS-1** | ODD-0026 accepted, recording the model + the fate of `source.paths` and the body-hash gate for planning nodes | `docs/design/04-accepted/0026-*.md` present, status Accepted | serious | open |
| **SS-2** | A planning node with **no external source** passes `check` (no `undeveloped-stub` / missing-source / body-hash error) | fixture: an authored node, `check` clean | serious | open |
| **SS-3** | Existing planning nodes converted to self-sourced per ODD-0026; a full `migrate --all` + `check` is green with the `./docs` plan tree still present | `migrate --all` then `check` exit 0 | serious | open |
| **SS-4** | A node body can be **created and updated via `./bin/odm`** (no hand-edit of store files required) | `node new --from-file` + `node edit` demonstrated; the created/edited body reads back via `node show` | serious | open |
| **SS-5** | The body-hash fidelity gate **still applies to genuinely-migrated content** (end-user/legacy docs with an external source) — no regression | fixture: a migrated node with a corrupted body still fails `check` | correctness | open |
| **SS-6** | **DoD — reproduced at arc scale:** a fresh context authors a new arc + slice end-to-end using only `./bin/odm`, with no `./docs` planning tree and no hand-edited store files | demonstration transcript | **serious** | open |
| **SS-7** | With the `./docs` planning subtree deleted, `check` / `orient` / `list` / `rollup` are green and unchanged; end-user `./docs` is untouched | delete on a branch, `check` exit 0 | serious | open |
| **SS-8** | Decomposition bookkeeping is deterministic: (a) `node decomposed` and `check` use **one** child-set definition — a parent whose current children set-equal its affirmed set reports **0** drift (MF `#58837400` clears); (b) `migrate` auto-recomposes provably-identity child churn with no manual re-affirm | fixture: the MF artifact-child case → 0 drift after affirm; a re-mint `migrate` leaves `check` green with no manual step | serious | (a) **done** — slice 05 CDC-verified; (b) delivered, inert (SS5-1) |
| **SS-9** | `migrate --all` is **hands-off** for authored additions: an already-affirmed parent gaining a plan-tree-declared work-child is **auto-extended** (no manual `node decomposed`), while a work-child *removal* and a *never-affirmed* parent still behave conservatively | fixture set + real: reset → `migrate --all` → MF 0 drift with no manual step | serious | fixture set **delivered, attested** (slice 06 closed by CC); real-store leg (MF 0 drift) **deferred to CDC** — no `.worktrees/odm` access from the implementation worktree |

**SS-6 is the arc's reason to exist** and, like every DoD row, is closed by a
fresh/independent context, not by the implementer.

## The design forks (resolved in slice 01 / ODD-0026)

Stated here so the arc's design basis is visible; slice 01 carries the detail and
my recommendations. Four are open choices (A, B, D, E); **C is already settled** and
is listed only to make the cutover scope explicit.

- **A — `source.paths` for planning nodes.** Drop external source entirely (node
  file is the source; mark `origin: authored`) vs. repoint self-referentially.
  *Lean: drop it* — generalizes ODD-0025's existing no-source case.
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

### v1.5 — 2026-08-02 (slice 06 CC-closed: hands-off migrate implemented)

CC implemented slice 06: `Outcome::AutoExtended` on `odm_migrate::decompose::auto_recompose`, additions-only
against the affirmed **work-children** set (reusing slice 05's `decomposition_children`, not modifying it),
rewriting the affirmation to the current work-child set when nothing affirmed is missing — which auto-heals
the MF transitional shape (stale non-work ids drop out of the comparison) in the same step as folding in a
genuine addition. **Attested (module fixtures + real `odm-cli` `migrate --all` CLI-pipeline tests):** F-1
(auto-extend on a clean addition), F-3 (a genuine work-child removal — surfaced via `odm node unlink … part_of
…`, since `migrate` never deletes nodes — still drifts), F-4 (a never-affirmed parent untouched), F-5
(idempotent), F-6 (`RECOMPOSE` output + summary distinguish auto-extended; no `--json` surface exists yet for
`migrate --all` to extend), F-7 (no regression; full workspace `make check` green). **F-2 split**: its fixture
half (the MF transitional shape) is attested; its real-store half (reset → `migrate --all` on the actual `odm`
corpus → 0 drift) is **deferred to CDC** — the `.worktrees/odm` orphan-branch checkout is not present in the
implementation worktree, so this is the still-open acceptance-anchor leg. One finding worth folding into
**ODD-0026**: the affirmed-side work-child filter has a sharp edge — an id still present in the corpus but
non-work-typed is safe to drop as stale, but an id **absent from the corpus entirely** must not be filtered the
same way, or a genuine removal reads as stale cruft (caught by re-running slice 05's own removal fixture before
trusting the change, not by external review). The pre-existing slice-05 integration test
(`migrate_all_leaves_a_genuinely_new_slice_as_drift_not_auto_affirmed`) tested exactly the scenario slice 06
flips; renamed to `migrate_all_auto_extends_an_affirmed_parent_for_an_authored_addition` and its assertions
updated to the new contract rather than left stale. See
`slice06-auto-extend-decomposition/closing-report.md` for the full walk. Surfaced by: CC (this session).

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
