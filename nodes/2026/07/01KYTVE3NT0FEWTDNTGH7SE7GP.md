---
id: 01KYTVE3NT0FEWTDNTGH7SE7GP
number: 510494700
type: artifact
schema: artifact/v1.1
name: Slice 11 closing report — Synthesis capability + L-8b reconciliation
created: 2026-07-29
updated: 2026-07-29
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice11-synthesis-l8b/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-31
edges:
  part_of: 01KYSX4R1GDJQXPPGFWXP5ZZWK
---
# Slice 11 closing report — Synthesis capability + L-8b reconciliation

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 11 · **Feeds:** MF-7, MF-8 ·
> **Realizes:** ODD-0025 §2.3 (synthesis, `supersedes`→`Vec`, bidirectional lineage), `uat-coverage-
> audit.md` §L-8 (L-8b) · **Assignment:** `cc-prompt.md` · **Ledger:** `ledger.md` (F-1…F-10) ·
> **Implemented by:** CC · **Date:** 2026-07-29
> **Branch:** `release/1.0.x` only — `38941a8` (code F-1…F-6 + the L-8b bare renames), `d80cdb3` (this
> ledger/report/bubble-up), `6664f9f` (F-7's actual content — a self-caught correction, see Deviations).
> **No `odm` branch commit this slice** — fixture/doc only, per the hard rule; the live application is
> s12. **Evidence class:** fixture-attested (LEDGER-DISCIPLINE v2.0 §B class-(a)) throughout; there is
> no class-(b) row — nothing in this slice touches the live store.

## What shipped

**The model + check rule (F-1, F-2):** `edges.supersedes` changed from `Option<Supersedes>` to
`Vec<Supersedes>` (ODD-0025 §2.3 — a synthesis may supersede many sources), with every read site
updated: the index adapter and build (already list-shaped for every other edge kind, so this was
additive), graph edge-building, `check`'s field-validity/dangling-edge rules, and the CLI's JSON schema,
`show` text, and `supersede` command. `check_supersession` — the self-supersede and cycle detector — was
rewritten as an explicit-stack DFS, since a node can now supersede many targets and the relation is a
general directed graph, not a single-successor chain; the old implementation's `loop` over one `succ: Id`
per node could not have been mechanically patched to handle branching correctly. `superseded_by` remains
what it always was: not a field anywhere in the model, so "derived, never stored" holds by construction,
not by discipline.

**The synthesis capability (F-3, F-4, F-5):** a new `odm_migrate::synthesis` module builds a synthesized
node's frontmatter in memory (no I/O) — the forward `supersedes` edges attached unconditionally, a
`Source` record (extended with new `synthesis`/`attestation` fields) carrying the regime and the
multi-element `source.paths`. `concatenation` is hash-gated against a join that is now **concretely
defined**, not just named: caller-supplied source order, a fixed separator, per-source `trim+lf`
normalize (reusing the existing migration-fidelity normalization, not a new one). `editorial-merge`
requires an explicit, recorded `Attestation` — never a silent pass — with the lineage half of the
invariant guaranteed structurally by F-2's `check` rule rather than re-verified inside the builder.
`other` performs no automatic verification, exactly as ODD-0025 §2.3 allows.

**The project-vision re-cast, fixture-proven (F-6):** a `TempDir` test demonstrates the full mechanism
the vision re-cast needs — a `self_host`-minted, faithful 1:1 `project-plan` node, then a synthesis node
that supersedes it via `editorial-merge` with a recorded attestation, verified clean under `check`'s
lineage rule. This replaces `replan::vision_from_plan`'s role as *the whole mechanism* with its role as
*the text-extraction step feeding a modeled synthesis* — `vision_from_plan` itself is unchanged; only
what happens to its output changes. The live numbering/identity-continuity policy (does the synthesis
inherit the project's existing number?) is explicitly left to s12, disclosed in the test's own header.

**L-8b — the four ODDs' authoritative `state:` corrected (F-7):** ODD-0013's `state: Draft` → `Accepted`
(the cc-prompt's own worked example — its already-`Accepted` amendments 0019/0020 presupposed it was
settled). ODD-0017 and ODD-0018 — named by `uat-coverage-audit.md` §L-8 as "load-bearing but `01-draft`"
without an explicit target state — were confirmed by citation evidence (not guessed): both are the
design authority multiple *closed, built* arcs already depend on (0017: arc02, arc03, arc06, arc08,
arc-llm-command-surface, arc-release-hardening; 0018: arc07-two-clock-telemetry, arc08-forecasting),
the same pattern L-8 names generally. All three corrected to `Accepted` and `git mv`-relocated
`01-draft/` → `04-accepted/` (clean renames, history preserved), each with a dated Version-History entry
naming the L-8 citation and the s11-corrects/s12-reconciles split.

## Ledger — per-row walk

| ID | Criterion | Status | Disposition |
|----|-----------|--------|-------------|
| F-1 | `supersedes` is a list | **done** | Model + every read site updated; ≥2-target and round-trip fixtures pass; no stored `superseded_by` anywhere (confirmed by grep). |
| F-2 | Bidirectional-lineage + `check` rule | **done** | `check_supersession` rewritten (explicit-stack DFS); 6 new multi-target fixtures (clean, dangling-among-many, self-supersede-among-many, branching-cycle, 3-way fan-out) plus the 3 pre-existing 1-target tests, all green. |
| F-3 | Synthesis records type + multi-path source | **done** | `Source.synthesis`/`.attestation` added; `build_synthesis` populates both; fixture-proven. |
| F-4 | `concatenation` hard-gated, deterministic join defined | **done** | Join concretely defined (order/separator/normalize) and documented in code; pass/fail/order-sensitivity fixtures. |
| F-5 | `editorial-merge` verified by lineage + attestation | **done** | `MissingAttestation` rejected; lineage always attached, verified structurally by F-2; fixture-proven. |
| F-6 | Project-vision re-cast fixture-proven | **done** | `TempDir` fixture: 1:1 project node + editorial-merge synthesis superseding it, `check`-clean, no live write. |
| F-7 | L-8b: correct the four ODDs' `state:` | **done** | 0013/0017/0018 corrected + `git mv`'d; 0017/0018's target confirmed by citation evidence, disclosed. |
| F-8 | No live store mutation | **done** | `.worktrees/odm` untouched, still `2fc25f5`; no design-corpus migration (L-8a) started. |
| F-9 | No model drift; amend not work around | **done** | 1:1 migration + hash gate byte-unchanged; no ODD amendment needed (§2.3 already scoped the join's specifics to implementation). |
| F-10 | Clippy/unsafe/coverage | **done** | Clippy + fmt clean; 0 `unsafe`; new/changed lines all covered; `commands.rs`'s file-wide 89% (pre-existing, unrelated shortfall) disclosed, not chased. |

**Rows: 10. Done: 10. Deferred: 0. No-op: 0.** No silent drops: every "Out" item (the live vision mint,
all live reconcile, L-8a) is confirmed untouched (F-8).

## Verification

| Check | Result |
|-------|--------|
| `cargo test --workspace --all-features` | 0 failed, checked after every code checkpoint |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | clean (only the pre-existing, unrelated `proc-macro-error2` notice) |
| `cargo fmt --check` | clean |
| `unsafe` in every file this slice touched (24 changed + 2 new) | none |
| `llvm-cov --workspace` | `synthesis.rs` 99.13%, `check.rs` 97.20%, `frontmatter.rs` 96.64%, `graph.rs` 96.52%; `commands.rs` 89.03% file-wide (disclosed, see F-10) |
| `.worktrees/odm` status | clean, still `2fc25f5` — untouched |
| L-8b renames | `git status` shows clean `R` (rename) status for all 3 files — history preserved, no content diff outside the frontmatter/Version-History edit |

## Deviations / findings (flagged, per the working agreement)

### The L-8b doc moves landed in the same commit as the code, undisclosed in the commit message

`38941a8`'s message describes F-1…F-6 (the synthesis capability + lineage check) but does not mention
F-7 (the L-8b `git mv`s). Both are in the **same** commit: the L-8b renames were already staged (via
`git mv`, which auto-stages) from an earlier step in this session, and the subsequent `git add` +
`git commit` for the code swept them in too. This session's git discipline is new-commits-only (never
amend), so the correction is this disclosure, not a rewritten commit.

### `38941a8` staged only the bare rename — F-7's actual content landed late, in `6664f9f`

More serious than the message-wording issue above: `38941a8` turned out to contain only the *pure
relocation* of the three L-8b files — `git show 38941a8:<path>` still read `state: Draft` and the
original version number, with no Version-History entry. The content edits (the actual point of F-7) had
been applied to the working tree correctly, but `git status`'s two-letter `RM` code — rename staged in
the index, **plus** a further unstaged modification on top — was misread as "fully staged" from its
one-line summary. This meant the ledger and this report, as first written, described `done`/`reproduced`
evidence for content that did not yet exist in any commit.

**Caught and fixed before reporting the slice done to the operator**: a routine post-commit `git status`
check (part of this slice's own closing verification, not an external catch) surfaced the unstaged `M`,
and the missing content landed in `6664f9f` — verified by `git show 6664f9f:<path>` for all three files
now matching exactly what the ledger always described. No functional gap remains; the final state is
correct. **Process lesson, recorded because it generalizes**: after any `git mv` that is meant to carry
co-located content edits, check `git status`'s two-letter code specifically (not just its one-line
rename summary) before treating the move as complete — `RM`/`AM`-shaped statuses mean "staged, but with
more on top," not "fully staged."

### ODD-0017/0018's L-8b target state: confirmed by evidence, not literally named in the audit

`uat-coverage-audit.md` §L-8 explicitly names ODD-0013's correction (`draft`→`accepted`) but only
describes 0017/0018 generally ("load-bearing but `01-draft`"), without stating their target state. Rather
than assume `Accepted` by analogy to 0013, this was checked: both are cited as the design authority by
multiple already-closed, already-built arcs (listed above), which is the same "authority already acted
upon while still labeled draft" pattern the audit's finding describes. This is a disclosed judgment call,
not a literal instruction followed — flagged so the operator can correct it if the intended target
differs (e.g. if either was meant to stay at a lower rung like `revised`).

### No ODD amendment for the deterministic-join definition

ODD-0025 §2.3 already anticipated that `concatenation`'s deterministic join needed concrete definition
("fixed order, separator, per-source `normalize`") without specifying the values — reading this as an
implementation fill-in rather than a model gap, F-4 defines and documents the concrete choices in code
(`synthesis::JOIN_SEPARATOR`, doc comments) rather than amending the ODD. Flagged in case this reading is
wrong and the operator would prefer the concrete join recorded in the ODD itself, not just in code.

### `docs/design/index.md` left stale, disclosed (not a new finding)

The legacy-generated index (`docs/design/index.md`, "do not edit manually") still lists ODD-0013/0017/
0018 at their old `01-draft/` paths and `Draft` states. This file has been excluded from doc-coverage and
established as inert, unmaintained infrastructure since s10 (F-11) — this slice doesn't regenerate it,
consistent with that established handling, not a fresh gap.

## Bubble-up to `../arc-plan.md` (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

**Did s11 build synthesis and clear L-8b?** Yes, at the capability/doc level this slice was scoped to:
`edges.supersedes` is a verified-lineage list; a synthesis node records its regime and multi-path source,
verified per-regime (hash-gated or attested); the project-vision re-cast is fixture-proven as the modeled
mechanism it should have always been; and the three normative ODDs whose `state:` had drifted from their
real authority are corrected and relocated. **MF-7 and MF-8 move to done at the capability/gate level** —
the live application (actually firing the vision synthesis, and reconciling every node whose source the
model now disagrees with) is s12's, as scoped from the start.

**What this slice revealed that the arc-plan didn't anticipate:**

1. **A single-target `supersedes` model quietly assumed a chain, not a graph** — `check_supersession`'s
   original cycle detector was correct for the single-target case but could not have been safely patched
   for branching; it needed rebuilding, not extending. Worth naming for any future edge-kind change: a
   `Vec`-ification of a previously-scalar edge is not always purely additive at the *validation* layer,
   even when it is at the storage layer.
2. **L-8b's two under-specified targets (0017, 0018) show the audit's own "load-bearing" language is
   itself evidence-checkable** — a document is load-bearing exactly when other, already-shipped work
   depends on it, which is a fact `grep`-able across arc-plans, not a judgment call requiring guesswork.
   Future L-8-shaped findings can use the same citation-evidence method rather than defaulting to "ask
   the operator" for every under-specified target.
3. **The commit-bundling deviation (F-7 landing silently inside F-1…F-6's commit) is a process finding
   worth carrying forward**: `git mv`'s auto-staging behavior means a later, unrelated `git add` can sweep
   in already-staged renames without the committer noticing at commit-message-writing time. A checklist
   habit — `git status` immediately before writing any commit message, not just before `git add` — would
   have caught this before it happened rather than after.

**The slice-scale silent-drop diff:** scope-as-specified vs. scope-as-delivered — no drops. Every "In"
item (F-1 through F-7) landed; every "Out" item (the live vision mint, all live drift reconcile, L-8a) is
confirmed untouched (F-8).

**Recommended arc-ledger update:**

- **MF-7** ("synthesis lands as supersede lineage; project vision re-cast; bidirectional guaranteed") →
  **done at the capability level** — the model, the check rule, and the vision-recast mechanism are all
  fixture-proven; the live mint is s12's.
- **MF-8** ("L-8b: ODD-0013/0017/0018 (+0019/0020) reconciled to authoritative states") → **done at the
  doc-tree level** — all three corrected and relocated; the corresponding **node** gate vectors still
  read the pre-correction authority until s12 reconciles (expected, disclosed, not a gap).
- **s12 (reconcile run) is next**, now carrying the full live close: the vision-synthesis mint, the s08
  active-arc-node body drift, ODD-0013/ODD-0020's body drift (s10 F-6), the ODD-0025 §4 re-snapshot (s10
  iteration 1 F-20), and this slice's 3 L-8b-moved ODDs' `source.paths`/gate-vector reconcile — plus the
  arc composition / P-12 acceptance demonstration.
