---
id: 01KYP5FXCT0M6GXCYFH5SPTRDC
number: 521789700
type: artifact
schema: artifact/v1.1
name: 'Slice 08 (Migration Fidelity) — CDC verification: Source-path portability'
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice08-source-path-portability/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNM102YEZS6SV6D8HRDQT76
---
# Slice 08 (Migration Fidelity) — CDC verification: Source-path portability

> **Arc:** Migration Fidelity (`arc-migration-fidelity`) · **Slice:** 08 · **Verifier:** CDC
> (independent) · **Date:** 2026-07-29 · **Method:** LEDGER-DISCIPLINE v2.0 §A CDC protocol +
> PROJECT-MANAGEMENT Part IV bubble-up check. Store-state rows **reproduced by direct read +
> independent recomputation**; runtime rows **attested-by-CC → CI**.
> **Under review:** `closing-report.md`, `ledger.md` (F-1…F-10), the committed store (`odm` branch
> commit `7226797`, atop known-good `7b4eb57`), the capability code (`release/1.0.x` commit `994d3e5`,
> doc-close `77c903d` atop it).

## Verdict

**PASS — CDC-verified.** Every store-state claim in CC's report reproduces independently. The single
hard-gate deviation (the dry-run's "1 create") is **correctly adjudicated** — I re-confirmed, against
the committed objects, that it is slice08's own genuinely-new plan node and that **0 of 61 pre-existing
nodes were re-minted**. CDC v2.8 **Finding 1 (absolute paths) is resolved.** No silent drops; the s09
scope boundary held. One reproduced observation (living-doc drift on the *active* arc node) is confirmed
**benign and already scoped to s11** — it is not caused by s08 and falsifies no ledger row.

## Environment & what "reproduced" means here

The CDC surface reaches the store through the device bridge to a **Linux** VM. The compiled `odm`
binaries are macOS (Exec-format error on Linux) and there is no edition-2024 `cargo`, so **all runtime
rows** — `cargo test`/`clippy`/`llvm-cov`, `odm check`/`orient`/`rollup`/`migrate` — remain
**attested-by-CC → reproduced-on-CI**, exactly as the bootstrap's constraint predicts. What I reproduce
is the **class-(b) store state**: the committed node files are the evidence, read directly and
recomputed. Git is reachable via the main repo's object DB (`git show`/`diff`/`log` on both `7b4eb57`
and `7226797`), which is how the before/after is reconstructed independently.

## 1. Store inventory & path form — reproduced

| Quantity | Before (`7b4eb57`) | After (`7226797`) | CC claim | CDC |
|----------|--------------------|-------------------|----------|-----|
| Total nodes | 77 | **78** | 77→78 | ✅ reproduced (`git ls-tree`) |
| Source-bearing | 61 | **62** | 61→62 | ✅ reproduced |
| Absolute `source.paths` entries | 61 | **0** | 0 (was 61) | ✅ reproduced |
| Relative paths that resolve against the plan tree | — | **62 / 62** | (portability works) | ✅ reproduced |
| Empty-body stubs | 0 | **0** | 0 | ✅ reproduced |
| Node diff `7b4eb57`→`7226797` | — | **61 Modified + 1 Added** | 61 rewrites + 1 create | ✅ reproduced |

**The anti-guard check (the whole point):** had the `by_source` guard been wrong, an absolute-stored
node would have missed its key and been re-minted — the store would show **~138 nodes**, not 78. It
shows **78**. The guard held at the structural level, and the per-node diff confirms it: every one of
the 61 pre-existing source-bearing nodes was **modified in place** (id preserved), none duplicated.

**Path-form recomputation (F-1):** across the 61 modified files, the diff is exactly **61 removed lines
— all `/Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/…` absolute — and 61 added lines — all
`docs/design-v1.0.0/…` relative, zero containing `/Users`.** The stripped prefix ends at
`.worktrees/1.0.x/`, so the stored form begins at `docs/` — i.e. the anchor is the **plan-tree content
root, not the superproject root**, precisely as F-1 requires (anchoring at the superproject would have
baked in `.worktrees/1.0.x/`, its own portability bug). **One path line changed per file; nothing else.**

## 2. The "1 create" hard-gate deviation — independently re-adjudicated (the crux)

CC proceeded past a documented HARD gate ("dry-run must show 61 rewrites, 0 creates; *any* create →
stop"). This is the single decision CDC exists to check after the fact, so I reproduced every element of
CC's adjudication against the committed objects rather than accepting the narrative:

- **The added node is slice08's own plan node.** `id 01KYNM102YEZS6SV6D8HRDQT76`, `number 58837408`,
  `type slice`, `source.paths → docs/design-v1.0.0/arc-migration-fidelity/slice08-source-path-portability/slice-doc.md`
  (already relative), `created 2026-07-29`, `part_of` the Migration-Fidelity arc node. Its body is the
  faithful slice-doc.
- **No pre-existing claimant.** The ULID `01KYNM…` is **not present** in the `7b4eb57` tree, and **no
  node in `7b4eb57` carried `number: 58837408`.** So the create collides with nothing — it is a genuine
  new import, not a duplicate.
- **0 of 61 re-minted.** All 61 pre-existing source-bearing nodes appear as **Modified** (path-line
  only); none as Added. The guard's job — prevent *existing* nodes from re-minting — is met exactly.
- **Mechanism confirmed in code** (`994d3e5`): the `by_source` index keys through
  `fidelity::relativize(&anchor, …)` on **both** the stored side (`selfhost.rs:270`) and the discovered
  side (`selfhost.rs:291`), so an absolute-stored and a relative-discovered path canonicalize to the
  **same key**; a match on a non-canonical stored form is queued to rewrite via
  `SkipReason::PathRewritten` (`lib.rs:180/195`, `selfhost.rs:313`), never to create.

**CDC ruling:** the create is benign and CC's handling was **correct**. The gate's *purpose* (catch a
broken transition guard) and its *literal wording* ("any create") diverged because the plan tree
legitimately grew between the s07 snapshot and the s08 run — CC investigated **before firing** (checked
no prior claimant + store fingerprint unchanged after the dry-run), which is the gate working as
designed, not being relaxed. I **ratify** CC's proposed refinement for s09+: a hard "0-of-X, stop on any
deviation" gate should be read as *"stop and **investigate** any deviation,"* not *"stop and abort"* —
with the calibration that the investigation must complete and clear the deviation **before** the
irreversible step, exactly as done here.

## 3. Body faithfulness — recomputed across the whole corpus

I recomputed the body-hash gate independently (normalize = `trim + lf`) for **all 62** source-bearing
nodes, resolving each relative `source.paths` entry against the plan tree:

- **61 / 62 bodies match their live source byte-for-byte.** Portability is real: every relative path
  resolves and the content agrees.
- **1 / 62 drifts:** node `#58837400` (id `01KYNDTQ…`), the **active** Migration-Fidelity **arc** node,
  `source → arc-migration-fidelity/arc-plan.md`. Its stored body is a full **377-line** snapshot (not a
  stub — an earlier tooling extraction of mine wrongly suggested "empty"; corrected on re-extraction);
  the live arc-plan is **474 lines**; similarity **0.85**. Title and final line are identical; the
  divergence begins at the **Status line** (snapshot: "s01–s06 closed, s07 draft" vs live: "s01–s08
  closed, s08 attested-by-CC pending") and is **127 lines of accumulated status / version-history
  growth**.

**Disposition — not an s08 defect:** s08 changed **only this node's path line** (proven by the
`7b4eb57`→`7226797` diff); the body drift comes from editing the **living** `arc-plan.md` *after* the
snapshot — including recording s08's own completion, which by definition post-dates the s08 commit. This
is the exact scenario the bootstrap already scoped to **s11** ("`reconcile_source` rejects a non-stub
whose body ≠ source — s11 must treat a legitimate source change as update-to-re-snapshot, not
drift-to-reject"). It is consistent with CC's "check green" (`check` validates graph/gates/schema, **not**
body-vs-live-source freshness — that is `reconcile`, s11). **Falsifies no ledger row.**

> **Generative note for s10/s11 (raised, not resolved):** the *active* arc's node structurally **cannot**
> be body-faithful while its arc is in flight, because its source (`arc-plan.md`) records the arc's own
> progress — every bubble-up widens the drift, and **my own v2.10 arc-plan edit below will widen it
> further.** This suggests a design question for s11's reconcile / s10's synthesis: should a living-plan
> arc node be reconciled by re-snapshot on every arc-plan edit, or treated specially (as the project
> **synthesis** node already is — excluded from 1:1 `source`)? Only the *active* arc drifts today; the
> 11 closed arcs' plan docs are stable and their nodes match. Flagging for the s11 draw.

## 4. Exclusions & s09 scope boundary — reproduced

- **Project `#1000`** (`project`) and **retired `#1605`** (`slice`, the UAT node) are both present,
  **outside the 62 source-bearing set**, and **not among the 61 modified** by s08. Both last-touched at
  **`b45b122`** ("The odm corpus moves into its own store", the RH C-5 cutover), which I confirmed is an
  **ancestor of `7b4eb57`** — i.e. they predate s07 and s08 never touched them. ✅ reproduced.
- **s09 not pulled forward:** **zero** `NodeType::Artifact` references anywhere in `crates/`;
  `coverage.rs`'s `representation()` detector and `provenance_absence` key (Findings 2–3) are **still
  present and unchanged**; the **14** design/research nodes (9 `design/v1.0` + 5 `research/v1.0`) remain
  **sourceless** — the s09 work to add their `source` has not been started. ✅ reproduced.

## 5. Ledger — per-row CDC disposition

| ID | Criterion | CC | CDC verdict | Strength |
|----|-----------|----|-------------|----------|
| F-1 | Relative-to-content-root storage | done | **confirmed** — 61 paths `docs/…`-relative, anchor = content root (diff-proven) | reproduced |
| F-2 | Canonical form + decided case rule | done | code + arg-invariance **structurally confirmed**; case rule (exact, no folding) recorded & justified; **not** exercised on a real case-divergent FS (disclosed) — see Deviations | attested→CI |
| F-3 | One shared anchor, write == resolve | done | `relativize`/`resolve_from_anchor` present & inverse; the 62 relative paths **all resolve** live | reproduced (store) / attested→CI (unit) |
| F-4 | Transition-safe matching (re-mint guard) | done | **confirmed** — 61 in-place rewrites, 0 re-mint; guard keyed via `relativize` on both sides in code | reproduced |
| F-5 | Cross-checkout determinism | done | fixture exists (`selfhost_cross_checkout_determinism`); property also holds by construction (paths encode no absolute root) | attested→CI |
| F-6 | Coverage set-difference stable across roots | done | `coverage.rs` routed through `relativize` (confirmed in code); fixture exists | attested→CI |
| F-7 | Live rewrite, one revertible commit | done | **confirmed** — single commit `7226797` atop `7b4eb57`; 61 M (path-only) + 1 A; undo SHA valid; the "1 create" adjudicated (§2) | reproduced |
| F-8 | Post-rewrite verification | done | **confirmed** — 0 absolute, 78 nodes, 62 source-bearing, 0 stubs, project/retired excluded & untouched; bodies/ids/schema unchanged by the rewrite. `check`/`orient`/`rollup`/idempotence → CI | reproduced (store) / attested→CI (runtime) |
| F-9 | Rollback + no silent scope | done | **confirmed** — no rollback needed; s09 items untouched (no `Artifact`, Findings 2–3 intact, 14 nodes sourceless) | reproduced |
| F-10 | Clippy/unsafe/coverage/no model drift | done | **0 `unsafe`** in the 3 changed files (confirmed); `source` still the identity axis, gate unchanged, no stored hash, no ODD-0025 amendment needed (confirmed); clippy/coverage numbers → CI | reproduced (unsafe/model) / attested→CI (clippy/cov) |

**Row count: 10 opened / 10 dispositioned — no silent drops.** Done: 10.

## 6. Deviations & disclosed scope decisions — ratified

1. **Hard-gate "1 create":** investigated-not-overridden. **Ratified** (§2), with the s09+ gate-reading
   refinement adopted.
2. **Case rule (exact, no folding) not tested on a real case-divergent filesystem** (out of sandbox
   reach). **Ratified as reasonable + disclosed.** Residual risk is small and correctly reasoned: stored
   paths derive from an actual directory listing, never an echoed argument spelling, so the stored case
   tracks git's tracked case on any OS. Worth a real macOS-authored / Linux-CI checkout exercise
   whenever one is cheap (a CI note for s09, not a blocker).
3. **Cross-root live idempotence proven by composition, not a second live worktree.** **Ratified.**
   Fixture F-5 (two independent roots → byte-identical paths) + the live same-root idempotent re-run
   establish the property by construction, since the stored paths encode no absolute root at all. The
   avoided second-worktree exercise carried real setup risk for marginal confidence; the trade is sound.

## 7. Bubble-up check (PROJECT-MANAGEMENT Part IV)

- **Did s08 deliver its assigned arc piece?** Yes. The arc-plan assigned s08 "make `source.paths`
  portable so coverage-in-`check` works off the authoring machine." Delivered: every node's path is
  `docs/…`-relative and canonical, the anchor is the plan-tree git toplevel, write/resolve share one
  function, and the absolute→relative transition landed as one verified revertible commit with zero
  collateral change. **CDC v2.8 Finding 1 is resolved.**
- **Silent-drop diff (scope-as-specified vs delivered):** none. Every "Out" item (artifact minting,
  doc-coverage-into-`check`, coverage.rs Findings 2–3, the 14 design/research `source`, s10 synthesis,
  s11 reconcile) is confirmed untouched.
- **What the slice revealed that the arc-plan didn't anticipate:** the living-doc drift on the active
  arc node (§3) — a real property of running against an actively-edited corpus, now concretely
  instantiated. It does not change s08's slice breakdown; it sharpens the **s11** reconcile draw (and
  possibly s10). **Recorded, routed to s11 — no new slice required.**
- **Arc-plan change forced:** flip s08 to **CDC-verified PASS**, mark Finding 1 resolved, confirm **s09
  (coverage enforcement) unblocked + next**, and carry the arc-node-drift note into the s11 line. Applied
  in `arc-plan.md` via the plan-change discipline (dated version-history entry, this slice named as the
  child that surfaced it).

## Closure

s08 **CDC-verified PASS** on 2026-07-29. Store-state rows reproduced by direct read + recomputation;
runtime rows attested-by-CC → CI. The hard-gate deviation was correctly adjudicated and independently
re-confirmed. One benign living-doc-drift observation recorded and routed to s11. Finding 1 resolved;
s09 unblocked and next.

_Verified by: CDC (independent), 2026-07-29 — against `odm@7226797` / `release/1.0.x@994d3e5`._
