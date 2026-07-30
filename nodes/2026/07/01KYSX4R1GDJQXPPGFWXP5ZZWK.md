---
id: 01KYSX4R1GDJQXPPGFWXP5ZZWK
number: 58837411
type: slice
schema: slice/v1.1
name: Slice 11 (Migration Fidelity) — Synthesis capability + L-8b reconciliation (plan-of-record)
created: 2026-07-29
updated: 2026-07-29
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice11-synthesis-l8b/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-30
edges:
  part_of: 01KYNDTQ6SVSRZMRH0E07YJMEP
status:
  built:
    reached: 2026-07-30
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-30
  planned:
    reached: 2026-07-30
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-30
  tested:
    reached: 2026-07-30
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-30
---
# Slice 11 (Migration Fidelity) — Synthesis capability + L-8b reconciliation (plan-of-record)

> Refs: `../arc-plan.md` (s11 row; MF-7 synthesis; MF-8 L-8b) · **ODD-0025** §2.3 (synthesis is a
> separate superseding step; `supersedes`→`Vec`; concatenation hash-gated vs editorial-merge attested;
> tooling-guaranteed bidirectional lineage) · **design-notes** F1/F2 (both DECIDED 2026-07-27) ·
> `arc-release-hardening/uat-coverage-audit.md` §L-8 (the L-8b definition). `depends_on:` s02 (ODD-0025),
> s03 (fidelity core).
>
> **Capability + doc reconciliation — fixture/doc only, no live store mutation** (the s06/s09 shape). It
> builds the synthesis capability and clears the L-8b release-gate; the **live application** (fire the
> vision synthesis + re-snapshot every node this and prior slices drifted) is **s12 (reconcile run)**.

## Goal

Give odm a real **synthesis** step — a node that *supersedes* many sources, verified by regime — and
**clear the standing L-8b pre-ship gate**. **Done when** `edges.supersedes` is a list (many-to-one) with
a tooling-guaranteed bidirectional-lineage invariant + a `check` rule; a synthesis node records its
`synthesis:` type and multi-path `source`, hash-gated for `concatenation` and attestation-verified for
`editorial-merge`; the project vision is **fixture-proven** re-cast as an editorial-merge synthesis
superseding a 1:1 `project-plan` node; and ODDs **0013, 0017, 0018 (+0013's 0019/0020 amendments)** are
moved/accepted into the state directories their authority implies — all in code + docs, `TempDir`-proven,
**with the live `.worktrees/odm` store untouched** (s12 fires it).

## Why

Two loose ends the arc committed to close. **Synthesis:** ODD-0025 §2.3 draws a hard line — merging many
docs into one is *not* migration (which is strictly 1:1, hard-gated), it's a later step that mints a new
node superseding its sources. The project node already violates the 1:1 rule today: its ~2 KB vision body
is a bespoke `replan.rs::vision_from_plan` extract of project-plan §1, not a faithful copy of the ~40 KB
`project-plan.md`. Formalizing it as a synthesis makes that honest and gives every future many→one merge a
verified home. **L-8b:** these four normative ODDs carry the wrong **authority** today. A design doc's authority is its
node's gate vector, which `mapping.rs::reach_cumulative` derives from the doc's `state:` frontmatter
(ODD-0013 §9) — and those `state:` fields are **stale**: node #13 reads **draft** though ODD-0013's
0019/0020 amendments are accepted; 0017/0018 are load-bearing but still `draft`. s10 faithfully migrated
the stale `state:`, so the staleness is now baked into the node gate vectors — a normative architecture
doc shipping with its authority reading "draft" when it is really accepted, which is *exactly* the drift
odm exists to prevent, sitting in odm's own corpus. Release-blocking before v1.0.0. **The
`01-draft/`/`04-accepted/` directory is display-only in the model** — the node reads authority from the
gate vector, never the folder (verified: no production code treats the `NN-state/` dir as authority) — so
the fix is the `state:` field; the folder move only keeps the browsable view honest.

**Capability-first** (operator preference; the s06/s07, s09/s10 rhythm): s11 builds and fixture-proves the
synthesis capability and does the L-8b doc moves; s12 fires the live application. Note a helpful property
that removes a red-window worry: moving an ODD file does **not** break the live coverage check, because
`match_odd` covers by frontmatter `number`, not path — so the moved docs stay covered; the file move only
leaves `source.paths` staleness for s12's reconcile to resolve.

## Scope

**In (code fixtures + doc-tree edits; NO `.worktrees/odm` store mutation):**

- **`edges.supersedes` → `Vec`** (F1, ODD-0025 §4). Change the model (`frontmatter.rs`
  `supersedes: Option<Supersedes>` → a list), plus the index/adapter/CLI that read it. Keep
  `SupersedesKind` {`Obsoletes`|`Updates`} — it is the orthogonal *what-it-does-to-the-target* axis
  (synthesis-type is the *how-merged* axis; §2.3). `superseded_by` stays **derived, never stored**.
- **Tooling-guaranteed bidirectional lineage + a `check` rule** (F1, hard requirement). Every synthesis
  writes the forward `supersedes` edges; the reverse is derived; a `check` rule flags any node claiming a
  supersede whose target is missing, or any lineage inconsistency — so the back-edge can never drift
  silently (it is never hand-maintained).
- **The synthesis step + its two verification regimes** (F2, §2.3). A synthesis node records
  `source.synthesis: concatenation | editorial-merge | other` and a multi-element `source.paths`.
  `concatenation` **stays hard body-hash-gated** against a **deterministic join** (define + document:
  source order, separator, per-source `normalize`). `editorial-merge` is not hash-checkable → verified by
  supersede lineage + an explicit recorded attestation. Keep as much hard-fail as each regime allows.
- **Fixture-prove the project-vision re-cast** (MF-7). Prove, on a `TempDir`, that the vision becomes an
  **editorial-merge synthesis** node superseding a **1:1 `project-plan` node** (faithful body =
  `project-plan.md`, hard-gated) — replacing the bespoke `vision_from_plan` synthesis with the modeled
  one. **Do not fire it on the live store** (s12).
- **L-8b — correct the four ODDs' authoritative state** (MF-8). The substantive fix is the **`state:`
  field, not the folder**: **update each doc's `state:` frontmatter to its true authority** — for
  **ODD-0013, 0017, 0018** (folding in 0013's **0019/0020** amendments) — **confirming the target state
  per doc against `uat-coverage-audit.md` §L-8, not guessing** (e.g. 0013 draft → accepted). Then `git mv`
  each file into the matching `NN-state/` dir so the browsable view agrees (a clean rename; history
  follows). This is a **doc-tree** change under `docs/design/` — **not a store mutation**. The corrected
  `state:` flows into each node's gate vector only when **s12 reconciles** the drift; the nodes read the
  *stale* authority until then, and that reconcile is s12's, not this slice's.

**Out:**

- **The live synthesis mint + all live reconcile → s12.** Firing the vision re-cast on `.worktrees/odm`,
  and re-snapshotting every node drifted by (a) this slice's L-8b file moves, (b) the pre-existing drift
  the s08/s10 verifications found (the active arc node; design nodes ODD-0013, ODD-0020; the ODD-0025 §4
  re-snapshot) — all land in **s12 (reconcile run)**, the arc's single live-store close, together with the
  composition / P-12 acceptance demonstration.
- **L-8a** (migrate the whole design/ODD corpus into `design`-type nodes so authority = gate vector) — a
  **post-1.0 follow-on**, explicitly *not* this arc (`uat-coverage-audit.md` §L-8a).
- No change to the 1:1 migration rule or the body-hash gate (unchanged); synthesis is the *separate*
  superseding step, never migration.

## Verification

Fixture / `TempDir`, class-(a) (fixture-attested → CI); doc-tree edits reviewed directly. **No live
class-(b) row** — the store is untouched. After the slice:

- `supersedes` is a list; a synthesis node supersedes ≥2 sources; the derived `superseded_by` and the
  `check` bidirectional-lineage rule hold (seed a broken lineage → `check` Error; valid → green).
- A `concatenation` synthesis passes/fails the deterministic-join hash gate correctly; an `editorial-merge`
  synthesis is accepted with recorded attestation + intact supersede lineage.
- The vision re-cast is fixture-proven (synthesis node + 1:1 `project-plan` node, faithful, hard-gated).
- The four L-8b ODDs carry their **corrected `state:`** frontmatter (true authority, confirmed per §L-8)
  and sit in the matching `NN-state/` dir; `git mv` history preserved. (The node gate vectors still read
  the pre-correction authority until s12 reconciles — expected, not a gap.)

CDC reproduces the code + fixtures + the doc moves structurally + on CI.

## Rollback & findings discipline

Fixture + doc only — no live store to revert. A capability gap in review is a normal fix-iteration
(five-cap). Any discovered model gap → **amend ODD-0025/0013, don't work around** (e.g. if the
`concatenation` deterministic-join needs a spec line). The L-8b moves are `git mv`s on `release/1.0.x` —
reversible in git; keep each move a clean rename so history follows.

## Exit

`ledger.md` closed; CDC-verified against code + fixtures + the doc moves. Synthesis is a real, verified
capability and the L-8b gate is cleared. On close, bubble up to `../arc-plan.md`: MF-7 (synthesis) +
MF-8 (L-8b) → capability/gate done; **s12 (reconcile run) is next and now carries the full live close** —
the vision-synthesis mint + every drifted node (s08 arc-node, ODD-0013/0020, the §4 re-snapshot, and this
slice's L-8b-moved ODDs) + the arc composition / P-12 demonstration.
