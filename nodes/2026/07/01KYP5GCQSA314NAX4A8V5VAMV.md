---
id: 01KYP5GCQSA314NAX4A8V5VAMV
number: 500359600
type: artifact
schema: artifact/v1.1
name: 'CDC Verification — Arc 02 / Slice 04: Derived order & satisfaction'
created: 2026-06-25
updated: 2026-06-25
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc02-graph-gates-derived-order/slice04-derived-order-and-satisfaction/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKX6HS98MA47EGSK8G
---
# CDC Verification — Arc 02 / Slice 04: Derived order & satisfaction

> Independent verification of CC's closed ledger (impl `547b5f2`; closed `1bb8d8e`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran;
> **attested** = CC's evidence, not reproduced here. The heart of the design.

## Environment constraint (disclosed)

Cowork sandbox has no 1.85+ toolchain; cargo-gated rows route to CI / local.

## Row dispositions

**Row count:** 16 opened (incl. the H-15/H-16 status-wiring rows I folded in last
turn — CC picked them up from the uncommitted slice-doc edit), 16 addressed. No
silent drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
no `unsafe` (graph/core/cli); `odm-graph` still domain-agnostic (⊬ `odm-core`);
**H-15** typed `Status` wired into `Frontmatter` (`status: Status`,
`skip_serializing_if = is_empty`); satisfaction `DEFAULT_THRESHOLD = Reproduced`
with `is_soft = level < threshold`; CLI exposes `next`/`blocked`/`path`. → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
H-1…H-14 (topo/next/blocked/path, satisfaction, min-propagation, threshold,
soft-satisfied surfacing, staleness), H-16 (arc01 `unknown_keys_preserved` 2→1);
clippy; coverage line 96.59% / region 95.93%. → **PENDING CI.**

## Rulings on CC's flagged items

1. **Tears-without-rationale / "empty tears at CLI level."** **Accepted as
   deferred — folds into the CLI-mutators gap below.** The `Tear` *type* enforces a
   rationale (slice02); there is simply no `odm tear` command yet, so derived-order
   runs with an empty tears list.
2. **The disclosed arc01 `unknown_keys_preserved` 2→1.** **Accepted** — expected and
   planned (H-16); status moved unknown→known.
3. **`serde_norway` over the stale serde_yaml family.** **Accepted** (consistent
   with slice03).

## Headline finding — CLI graph-mutators gap (mine to own)

The CLI can now *create nodes* (`new`/…/`supersede`) and *query the graph*
(`next`/`blocked`/`path`/`check`) — but it has **no `link`/`unlink`, `set-gate`, or
`tear`** commands. So today you can query a graph you cannot fully *construct* or
*advance* through odm's own CLI: edges and gate transitions require hand-editing
frontmatter. 0013 §7 lists all of these as intended commands; they fell through a
crack — I listed `link`/`unlink` in §7 but omitted them from slice05's ledger
without flagging it, and `set-gate`/`tear` are odm-core ops without a CLI surface.

**Why it matters:** this is a **self-hosting prerequisite.** Post-A3 we migrate the
plan *into* odm — which requires wiring `depends_on`/`consumes` edges and setting
gates via the CLI, not by hand. It must exist before self-hosting.

**Recommendation:** a dedicated **CLI graph-mutators slice** — `link`/`unlink`,
`set-gate`, `tear` (+ confirm `new --parent` for `part_of`) — placed either as
**arc02 slice07** (a "wire the CLI to the engine" capstone) or at the **head of
arc03**. Needs your placement call. Tracked, not dropped.

## Minor

A stale code comment in `frontmatter.rs` still lists `status` as an example of a
*not-yet-modeled* unknown key — but status is now the typed field. One-line
touch-up (desired_facts remains the live example). Non-blocking.

## Verdict

Arc 02 / Slice 04 **CDC-verified on structure; all flags accepted; the consuming
half of evidence-leveled satisfaction is in; cargo rows pending CI.** Open: the
CLI-mutators gap (a self-hosting prerequisite) needs a slice + placement. On CI
green, slice 04 closes and slice 05 (recomposition integrity) opens.

CDC: planning thread, 2026-06-22. Iterations used: 1.
