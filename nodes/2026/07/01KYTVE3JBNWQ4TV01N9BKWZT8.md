---
id: 01KYTVE3JBNWQ4TV01N9BKWZT8
number: 585462700
type: artifact
schema: artifact/v1.1
name: 'Slice 11 (Migration Fidelity) — CDC verification: Synthesis capability + L-8b'
created: 2026-07-29
updated: 2026-07-29
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice11-synthesis-l8b/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-31
edges:
  part_of: 01KYSX4R1GDJQXPPGFWXP5ZZWK
---
# Slice 11 (Migration Fidelity) — CDC verification: Synthesis capability + L-8b

> **Arc:** Migration Fidelity · **Slice:** 11 · **Verifier:** CDC (independent) · **Date:** 2026-07-29 ·
> **Method:** LEDGER-DISCIPLINE v2.0 §A CDC protocol + PM Part IV bubble-up. **Fixture + doc only — no
> live class-(b) row.** Code + test existence/correctness and the L-8b doc edits reproduced by direct
> read on `release/1.0.x`; runtime execution (`cargo test`/`clippy`/`llvm-cov`) attested-by-CC → CI.
> **Under review:** `ledger.md` (F-1…F-10), the commits `38941a8` (synthesis F-1…F-6), `d80cdb3` (close),
> `6664f9f` (L-8b content fix), `6046101` (gap disclosure) — all `release/1.0.x`.

## Verdict

**PASS — CDC-verified.** The synthesis capability reproduces in code with genuinely thorough tests, the
L-8b reconciliation landed correctly (including the *content* edits the self-caught gap almost dropped),
and **no live store mutation occurred** (`odm` HEAD unchanged — reproduced). A clean slice; the one
process wobble (a first commit that staged only the renames) was caught by CC before reporting done,
fixed, and disclosed — the failure-recovery discipline working exactly as intended.

## 1. Synthesis capability (F-1…F-6) — reproduced

| Claim | Reproduced in code | Tests (non-vacuous, read) |
|-------|--------------------|---------------------------|
| **F-1** `supersedes` → `Vec` | `frontmatter.rs:703` `pub supersedes: Vec<Supersedes>`; read sites updated. (The `if let Some` at `odm-migrate/lib.rs:539` is the **legacy-doc** parser, not the node model — a `Vec` can't pattern-match `Some`, and the workspace compiles, so it is confirmed *not* a missed conversion.) | — |
| **F-2** bidirectional-lineage / cycle `check` | `Violation::SupersessionCycle` with cycle-path reporting (`commands.rs:1306/1343`); branching-safe DFS | `check.rs`: `supersession_cycle_is_flagged_once`, `a_branching_cycle_through_multiple_targets_is_flagged_once`, `a_node_superseding_three_targets_with_no_cycle_is_clean` (many-to-one, no false positive) |
| **F-3** synthesis type + multi-path | `synthesis.rs`: `SynthesisType{Concatenation,EditorialMerge}`, `source.synthesis` recorded | (below) |
| **F-4** `concatenation` hash-gated against a defined deterministic join | `deterministic_join` (`synthesis.rs:128`) — documented order/separator/per-source normalize | `concatenation_passes_on_a_faithful_join`, `concatenation_fails_on_a_tampered_body`, `deterministic_join_is_order_sensitive_and_normalizes_each_source` |
| **F-5** `editorial-merge` = lineage + attestation | `Attestation` required; typed error "an editorial-merge synthesis requires a recorded attestation" | `editorial_merge_requires_an_attestation`, `editorial_merge_is_accepted_with_lineage_and_attestation` |
| **F-6** vision re-cast | `vision_synthesis.rs` | `vision_becomes_an_editorial_merge_synthesis_over_a_1to1_project_plan_node` — asserts the 1:1 faithful `project-plan` body, `synthesis == editorial-merge`, a **recorded** attestation, the supersede edge (`kind == Updates`), clean lineage, **and store-untouched** |

Zero `unsafe` in `synthesis.rs`. `clippy`/`fmt`/coverage → CI.

## 2. L-8b (F-7) — content landed (the critical check)

Because CC self-reported that the first commit staged only the file renames, the load-bearing question
was whether the **`state:` content edits** actually reached the final state. Reproduced by direct read of
`release/1.0.x`:

- **ODD-0013, 0017, 0018 all sit in `04-accepted/` with `state: Accepted`**; none remain in `01-draft/`.
- 0013 is a real acceptance edit, not a flag flip: `state: Accepted`, `version: 2.5`, `updated: 2026-07-29`,
  with version-history growth.
- Git history confirms the recovery: `38941a8` landed the renames without the content; `6664f9f` = *"Fix
  L-8b: the state/version-history content edits never landed in 38941a8"*; `6046101` discloses it. The
  final tree carries the content. **The self-caught gap was genuinely closed.**

**0017/0018 → Accepted — judgment ratified.** The audit (`uat-coverage-audit.md` §L-8) named all three
ODDs for "move/accept," so `Accepted` aligns with its directive; CC's disclosed judgment (the audit
didn't name the *exact* state) is sound, and `Accepted` (vs `Final`) is the conservative pre-ship choice.
*Minor observation (not a finding):* 0018 is a `research` doc, so `Accepted` reads slightly oddly for a
non-decision document — a small state-vocabulary question for research vs. decision docs, disclosed, out
of L-8b's scope (which is: get the normative docs out of a lying `draft`).

## 3. The self-caught process gap — assessed (positive)

CC's first commit (`38941a8`) captured the `git mv` renames but not the `state:`/version-history content
edits (a misread `RM` porcelain code). A routine post-commit check caught it **before** reporting done;
CC fixed it (`6664f9f`) and disclosed it in the ledger, closing report, arc-plan, and the report. This is
the **let-it-crash / recover-cleanly** discipline working: a real near-miss (an unstaged content edit is
exactly the silent-drop this framework guards against) surfaced by the doer's own verification, named
plainly rather than absorbed. Counts as a Safety-II positive, not a defect — the content is in the final
state, verified above.

## 4. No live mutation (F-8) — reproduced

`odm` branch HEAD is unchanged at `2fc25f5` (s10 iteration 1's tip) — **no s11 store commit**. The synthesis
is in-memory/fixture only (the vision test asserts store-untouched); no live vision mint; L-8a not started.
The live application (fire the vision synthesis + reconcile every drifted node) remains s12, as scoped.

## 5. Ledger — CDC disposition (F-1…F-10)

F-1…F-6 confirmed in code + non-vacuous tests (reproduced structurally / attested→CI on execution). F-7
**confirmed** — content landed, `git mv` history preserved, 0017/0018 judgment ratified. F-8 **reproduced**
(no live mutation). F-9: no ODD amendment owed — ODD-0025 §2.3 already specifies the deterministic join's
*shape* (order/separator/per-source normalize); CC made it concrete in `synthesis.rs` (documented), which
implements the model rather than changing it. F-10: 0 `unsafe` (reproduced); clippy/coverage → CI.
**10 rows, no silent drops.**

## 6. Findings / observations

1. **The concrete deterministic join lives only in `synthesis.rs`** (code + doc comment) — **LOW /
   optional.** §2.3 delegated the specifics, so this is compliant; consider echoing the concrete
   separator/order into ODD-0025 §2.3 for discoverability (the join is a fidelity contract). Not a
   blocker; same spirit as the s09 §4 note.
2. **0018 (research) → `Accepted`** — observation, no action (§2 above). Disclosed judgment.

## 7. Bubble-up check (PM Part IV)

- **Did s11 deliver?** Yes — synthesis is a real, verified capability (list-supersede, cycle-safe check,
  two regimes, the vision re-cast) and the L-8b release-gate is cleared (the four normative ODDs now carry
  their true `Accepted` authority).
- **Silent-drop diff:** none. Every "Out" item (the live vision mint + all drift reconcile → s12; L-8a
  post-1.0) confirmed untouched. The one *near*-drop (the L-8b content) was self-caught and closed.
- **What it revealed:** the staging-porcelain misread is a reusable caution for any doc-move slice (a
  `git mv` + a content edit needs both staged) — worth a line in a future L-8a/move slice.
- **Arc-plan change:** flip s11 → **CDC-verified PASS** (v2.18); MF-7/MF-8 → done; **s12 (reconcile run)
  is next and carries the arc's full live close** — fire the vision synthesis; reconcile every drifted
  node (the s08 active-arc node, design nodes ODD-0013/0020, the ODD-0025 §4 re-snapshot, and this slice's
  L-8b-moved 0013/0017/0018, whose gate vectors still read the pre-L-8b authority until reconciled); the
  composition + P-12 acceptance demonstration.

## Closure

s11 **CDC-verified PASS** on 2026-07-29. Synthesis capability reproduced with thorough tests; L-8b content
verified landed (self-caught gap closed); no live mutation. s12 (the arc-close live reconcile + P-12) is
next. After s12, the arc closes and the self-host DoD is demonstrable.

_Verified by: CDC (independent), 2026-07-29 — against `release/1.0.x` (`38941a8`…`6046101`); `odm@2fc25f5`
unchanged._
