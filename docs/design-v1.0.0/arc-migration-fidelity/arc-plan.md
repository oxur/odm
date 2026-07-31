# Arc — Migration Fidelity: faithful, verifiable, repeatable migration — plan-of-record

> **Named arc, canonical number deferred** (operator call, 2026-07-27) — same §2a treatment as
> Release Hardening / arc-store-home / LLM-command-surface. **v1.0.0 release-blocking:** it sits
> on the **P-12** DoD path (odm cannot credibly claim to self-host while its own corpus is 44
> empty stubs and 6 of 11 arcs) and it **subsumes the standing L-8b pre-ship gate**. Directory:
> `arc-migration-fidelity/` (provisional; settles with the numbering).
>
> **Unblocked by ODD-0024 (G-1 closed):** the id scheme is decided (ULID retained), so the
> minting freeze that gated this arc's node-creation slices is lifted.
>
> `depends_on:` A6·slice04 (the self-hosted corpus this repairs); ODD-0024 (minting unfrozen).
> **Refs:** `reconciliation-audit-2026-07-27.md` (the discovery); ODD-0013/0020 (the node model
> the fidelity checks amend); **ODD-0025** (the fidelity model, Accepted — s02's deliverable);
> `arc-migration-fidelity/design-notes.md` (the decision log this plan draws on).
>
> **Status:** s01–s13 **closed** (s13 2026-07-30); s04–s13 CDC-verified PASS (s13 2026-07-31). Coverage is **enforced live and fully green**: `odm check` exit 0, 0 errors — the doc-coverage
> gap disclosed at s13's close (14 `uncovered-doc` errors for s11/12/13's own artifact-family docs) is
> **closed** as of `e06fffe`, fired by a new `migrate --all` (composes self-host + design/research
> reconcile + `--artifacts` + `--notes` + `--vision` into one idempotent pass — built the same day the gap
> was disclosed, prompted by an operator question about why nothing composed the five separate
> derivations). `388/388` docs covered. On the committed `odm` corpus (commit `e06fffe`, atop `e1e94bf`,
> atop `26bea1d`, atop `2fc25f5`, atop the s10 mint `355404a`, atop known-good `7226797`) — mint-all
> (268 `artifact` + 31 `note` nodes), **all** design/research nodes now source-bearing and reconciled to
> current sources (the 2
> disclosed-drifted ODD-0013/0020 + the moved-and-drifted ODD-0017/0018 + ODD-0025 itself, all fired live
> by s13), all `source.paths` portable (CDC v2.8 Finding 1, durably enforced by the `absolute-source-path`
> rule). s11 built synthesis (`supersedes`→`Vec` + bidirectional lineage + `concatenation`/
> `editorial-merge` regimes) and cleared L-8b (ODD-0013/0017/0018 corrected + relocated) at the
> capability/doc level. s12 built the **reconcile capability** (re-snapshot mode, moved-source
> re-discovery, the living-plan-node policy, the promoted vision-apply path, ODD-0025 §4 resolved),
> fixture-only. **s13 fired it all live**: every drifted node reconciled (0 remain), ODD-0017/0018's
> paths corrected and resolving, and the **vision minted** — `#1001` a faithful 1:1 `project-plan` node,
> `#1000` re-cast as the attested editorial-merge synthesis superseding it, clean lineage. **A third,
> post-close bug** (the operator's own investigation, not CC's verification, caught it): `#1000`'s
> `supersedes` edge is invalid on a `project`-type node per ODD-0020 §2's work/document field split —
> a latent conflict with ODD-0025 §2.3 nothing before s13 had exercised through `check`. Resolved
> same-day: `check_field_validity` now exempts a `source.synthesis`-bearing work node; **ODD-0020 → v1.4**.
> No live data changed. Two real bugs
> in the live-invocation wiring were caught by the dry-run/idempotence gates and fixed before commit (see
> `slice13-live-reconcile/closing-report.md`). **MF-7 done; MF-9 fidelity true on the live corpus** (doc-
> coverage caveat aside). **The arc-close is next**: MF-9 composition + the P-12 self-host acceptance
> demo + the **final reconcile-and-freeze** of the living-plan tail (CDC s13 Finding CDC-F1 — two
> still-edited plan nodes, `arc-plan.md`/slice10 `ledger.md`; the doc-coverage gap is already closed as of
> `e06fffe`). MF-9/P-12 must not be claimed before that freeze. *Plan late, plan deep.*

## Capability

Give odm a **complete, general, verifiable migration capability** — one that brings *every*
documentation file into the store faithfully, proves it did, and can be re-run across many
projects. Four properties define "faithful and verifiable," each enforced by a check rather than
trusted:

1. **1:1 verbatim bodies, hard-gated.** A migrated node's body **is** its source body. Migration
   hashes `normalize(source_body)` and `normalize(node_body)` (`normalize` = trim + CRLF→LF); a
   mismatch is a **hard error** that fails the migration. The importer performs **no body
   transformation** (no synthesised `# {name}` H1, no header injection) — the old transform
   behaviour is the thing that produced the 44 stubs.
2. **A `source` record + preserved `author`/`version` on every migrated node.** A `source` sub-map
   records paths (a list), class, normalization, and the migrating tool + version — computed at
   migration, no hashes stored (content is allowed to change — Version-History sections do). Legacy
   `author`/`version` are preserved as **typed fields** (not dropped, not git-derived). *(0013
   reserves `provenance` for **derived** lineage; this stored record is the distinct `source` axis —
   ODD-0025 §2.0/§2.2, Accepted.)*
3. **Frontmatter fidelity, schema-mapped.** The fields the source actually had are checked to map
   correctly onto the node's fields (e.g. legacy `state: Final` → the cumulative gate reach),
   over a versioned schema mapping. Only originally-present fields are checked.
4. **No file left behind, enforced.** A doc-coverage check — the inverse of `orphan` — asserts
   every `.md` in the migration source has a node. Its absence is *why* ~211 supporting docs went
   silently uncovered; wiring it into `check` makes that class of hole loud, permanently. **No
   scope cap** — the importer walks every arc/slice dir that exists (the `MAX_MVP_ARC` A1–A6 cap
   that caused the hole is removed, v1.6); `number` is a non-structural human handle, so named arcs
   just get one assigned.

Synthesis (merging several docs into one) is **not** migration: it is a separate, later step that
mints a **new** node **superseding** its sources (`edges.supersedes` → a `Vec`, with the
reverse `superseded_by` derived and **tooling-guaranteed** on every synthesis). Concatenation
stays hash-gated; editorial/conceptual merges are verified by lineage + attestation.

Applied first to odm's own `1.0.x` corpus — taking self-hosting from skeleton (44 stub bodies, 6
of 11 arcs, ~211 uncovered docs, 0 source records) to **100%**. Built general so the next project
runs the same capability.

## Exit criteria (arc acceptance — the composition check)

- **Coverage:** every `.md` under `1.0.x/docs/*` maps to a node; the doc-coverage check is green
  (no file left behind). Design/research nodes may be top-level *or* contained — both valid (F7).
  The arc's **own** report/verification artifacts are minted as `artifact` nodes (F10 mint-all,
  ODD-0025 §2.6) so the enforced check does not flag them in perpetuity.
- **Body fidelity:** every migrated node's `normalize(body)` hash equals its source's; the hard gate
  is green across the whole corpus; **no stub bodies remain**.
- **Source record:** every migrated node carries a `source` sub-map, and preserved `author`/`version`
  where the source had them (ODD-0025 §2.2).
- **Representation:** all real arcs (the 6 previously-excluded + A1–A6) and all slices are
  represented; scope is **uncapped** (`MAX_MVP_ARC` removed); named arcs carry an assigned `number`
  handle; supporting-doc children are minted.
- **Frontmatter fidelity:** the schema-mapping check is green over originally-present fields.
- **L-8b cleared:** ODD-0013/0017/0018 (+ the 0019/0020 amendments) reconciled to states that
  reflect their authority.
- **Reflexive:** `odm check` green; `orient`/`rollup` reproduce the hand-maintained truth; **P-12
  (odm self-hosts) is satisfiable at project close** against a *faithful* corpus, not a skeleton.

## Slice breakdown (one-line altitude; load-bearing order)

| Slice | Scope | Mints nodes? | Depends on |
|-------|-------|--------------|------------|
| **s01 — coverage-discovery** ✅ CLOSED 2026-07-27 (CDC-verified) | Build the read-only doc-coverage detector (inverse `orphan`) + sibling detectors; run over all `1.0.x/docs` → the **exact gap inventory**. | no (read-only) | — |
| **s02 — model** ✅ CLOSED 2026-07-27 (ODD-0025 Accepted) | The model ODD: `source` sub-map (§2.0); `author`/`version` typed fields; `supersedes`→`Vec`; the `artifact` node type; frontmatter-fidelity mapping; F4/F7/F10 resolved. → **ODD-0025**. | no | s01 |
| **s03 — migration-fidelity core** ✅ CLOSED 2026-07-27 (CDC-verified) | 1:1 verbatim import + **hard body-hash gate** + `source`/`author`/`version` typing + **no-transform**. Both importers + odm-core typing. | no (fixture-only) | s02 |
| **s04 — scope + repair capability** ✅ CLOSED 2026-07-28 (CDC-verified) | **Cap removed** (`self_host` imports **all** arc/slice dirs; named arcs get a non-structural `number` handle; `coverage.rs`'s shared predicate updated). **Update-in-place repair** of stub nodes (ODD-0025 §2.8 — real body + `source`, preserve id/edges/status, via `persist` overwrite — no `delete`). **Schema-minor bump executed** (`v1.0→v1.1`, forward-compat proven). Fixture-verified; no live mutation. | no (fixture-only) | s03 |
| **s05 — source-based identity** ✅ CLOSED 2026-07-28 (CDC-verified) | Retire `number` as a **correctness key** (F12 — the number problem's root): key `self_host` idempotence + coverage matching on **`source.paths`**, not `(type, number)`; **backfill `source`** onto the already-faithful non-stub nodes (body unchanged, hash-gate-confirmed) so *every* node carries one; make the named-arc handle **name-derived + stable** (cosmetic display only). `number` becomes a pure label nothing keys on — the position-based fragility dissolves permanently. Fixture-verified. | no (fixture-only) | s04 |
| **s06 — live-run capability** ✅ CLOSED + CDC-verified PASS 2026-07-28 | **Unified the two `source`-backfill paths** (v2.1 finding): a single `reconcile_source()` — gated, project-excluding, and (self-identified extension) retired-excluding — now backs both `self_host`'s `to_populate` transition and `repair()`. **Extended `odm migrate`'s self-host path**: `repair()` now runs before `self_host()` in `self_host_inner`, default-on, no new flag; `repair()`'s only callers were previously tests. **No new verb** (`self-host` folded into `migrate`, C-5); reuses `--dry-run`. **Fixture-verified end-to-end; no live mutation.** `context.json` re-pointing stays with s07 (structurally a live-store operator statement, not a migration artifact — disclosed in the closing report). | no (fixture-only) | s05 |
| **s07 — live repair run** ✅ CLOSED + CDC-verified PASS 2026-07-28 | Fired the s06 flow on the **live** `.worktrees/odm` corpus behind the full snapshot → dry-run → adjudicate → fire → verify protocol: commit `7b4eb57` (`odm` branch, atop known-good `e2ab628`) repairs the 44 stubs, gated-backfills `source` on the faithful nodes (none existed beyond the stubs — the corpus's one non-stub slice was the retired tombstone), imports the 6 previously-excluded arcs + their 11 slices, stamps `v1.1`. **Opened with** the CDC v2.5 doc-comment fix (`release/1.0.x` commit `b901b12`). **`context.json` deliberately left unchanged** — `migrate` has no code path that writes it; re-pointing operator focus is a separate act this slice doesn't make on the operator's behalf. **Verified:** `check` green, 0 stubs, 0 uncovered arc-plans/slice-docs, 0 `BodyHashMismatch` on re-verify, `orient`/`rollup` byte-stable, fully re-run-idempotent, project + retired node confirmed untouched. **Four pre-existing findings disclosed** (not fixed here): absolute `source.paths`, a report-clarity gap in `representation()`'s named-arc heuristic, a stale `provenance_absence` detector (checks the pre-s02 key name), and 8 genuine (non-blocking) `check` warnings about arcs with no slice subdirectories yet. **Deferred to s08 as scoped:** the 14 design/research nodes' `source` + the ~211 loose-doc coverage. | **yes (live)** | s06 |
| **s08 — source-path portability** ✅ CLOSED 2026-07-28 + CDC-verified PASS 2026-07-29 | Stores `source.paths` **relative to the repo content root** — `docs/…`, never the checkout/worktree root — via a shared `anchor_for`/`relativize`/`resolve_from_anchor` trio (`fidelity.rs`) anchored on the docs tree's **git toplevel**, one function for both writing and resolving so they can't drift. `self_host`'s `by_source` matching canonicalizes whatever is stored (absolute or relative) and the freshly-discovered path to the same key — the re-mint guard, fixture-proven (`selfhost_transition_rewrites_absolute_source_paths_to_relative`) — plus cross-checkout determinism and coverage stability across two independent roots (6 new tests total). `coverage.rs` updated the same way (a necessary addition beyond the literal file list, driven by its own cross-root criterion). **Live corrective re-migration** rewrote all 61 committed nodes' paths absolute→relative as one commit (`7226797` atop `7b4eb57`) — bodies/ids/schema byte-identical, project + retired untouched, 0 re-mint; the dry-run's one "create" (slice08's own newly-drawn plan node) was investigated and confirmed legitimate before firing, not silently overridden. Closes **CDC v2.8 Finding 1**. | **yes (live)** | s07 |
| **s09 — coverage enforcement (capability)** ✅ CLOSED 2026-07-28 + CDC-verified PASS 2026-07-29 | Built the enforcement machinery **fixture-only, no live mutation**: `NodeType::Artifact` (+ `artifact/v1.1` — schema is one global axis, not `v1.0`, flagged + justified; per-type validity inherited structurally); `crates/odm-migrate/src/artifact.rs::mint_artifacts` reaches the **artifact** doc family (mint-all incl. `coverage-report.md`, §2.6), containment resolved via a directory→id index built from arc/slice nodes' own `source.paths` (correct for numbered *and* named arcs/slices, no re-derivation); `mapping::backfill_source` reaches the **design/research** family (the 14 nodes' `source`, F7 — stub-replace/faithful-keep/drift-reject, mirroring `reconcile_source`); **`odm check`** gained a doc-coverage Error rule (`commands.rs` (c3)), config-gated on a new, currently-absent `[coverage] scan_root` key — confirmed still inert against `.worktrees/odm` (0 errors, unchanged). Fixed `coverage.rs`'s `representation()` "8/12" gap (`resolve_arc_dir_numbers` replays the named-arc collision key) + the stale `provenance_absence` detector (retargeted to typed `source().is_none()` — CDC v2.8 Findings 2–3, both resolved). 20 new tests; `cargo test`/`clippy -D warnings` clean throughout; 0 `unsafe`; no ODD edited. **Live activation is s10.** | no (fixture-only) | s02, s03, s08 |
| **s10 — coverage live run** ✅ CLOSED 2026-07-29 + CDC-verified PASS (incl. iteration 1) | Fired s09's capability on the **live** `.worktrees/odm` corpus behind the s07 snapshot → dry-run → adjudicate → fire → verify protocol: one commit (`355404a` atop `7226797`) — 254 `artifact` nodes mint-all (incl. `coverage-report.md`), 12/14 design/research nodes backfilled with `source` (2 correctly left untouched — drifted since original migration, routed to s12), 4 never-migrated ODDs imported, `[coverage] scan_root = "docs"` activated same-commit (no red window). **Result: `odm check` exit 0, 371/371 docs covered, 0 uncovered, 0/0 representation gap.** No collateral (project/retired last-touch unchanged); fully idempotent; `orient`/`rollup` byte-stable. **Grew by five operator-directed items, all disclosed** (`slice10-coverage-live-run/closing-report.md`): `docs/design/index.md`+templates excluded from coverage; `docs/dev/**` minted as `NodeType::Note` (31, new general capability); every creation path now stamps git-derived `created`/`updated` instead of "today" (RH F-20, previously flagged, now wired in); `check_decomposition` fixed to count only work-type children (a mint-scale false-positive it would otherwise have produced); `odm list` now tree-nests a slice-/arc-attached `artifact` instead of showing it as orphaned reference material. **Iteration 1 (same day, CDC finding):** CDC reproduced the live store and found 4 of the just-minted nodes (#22–25) carried **absolute** `source.paths` — CDC v2.8 Finding 1 recurring on the one design/ODD import seam never routed through `relativize`, masked by the coverage matcher's own relativizing tolerance. Fixed the seam (`release/1.0.x` `ff68186`), added an **unconditional, durably-enforced** `check` rule (`absolute-source-path`) so the class of bug — not just this instance — is caught immediately at any future recurrence, proved the rule against the real regression (4→0) before fixing it, then fired the 4-node path-string-only correction as one revertible commit (`2fc25f5` atop `355404a`, `release/1.0.x` `3eddf38` the fix mechanism). Two items explicitly skipped-and-flagged, not silently dropped: the optional ODD-0025 §4 wording companion (would drift node #25's own body) and a stale `ROLLUP.md` unrelated to this regression (`slice10-coverage-live-run/closing-report.md` §"Iteration 1"). | **yes (live)** | s09 |
| **s11 — synthesis + L-8b** ✅ CLOSED 2026-07-29 + CDC-verified PASS 2026-07-29 | `edges.supersedes` → `Vec<Supersedes>` (ODD-0025 §2.3) + a rewritten `check_supersession` (explicit-stack DFS, since a node can now supersede many targets — a branching graph, not a chain); a new `odm_migrate::synthesis` module: `concatenation` hash-gated against a now-concretely-defined deterministic join (order/separator/per-source `trim+lf`), `editorial-merge` requiring an explicit recorded attestation + the lineage F-2's `check` rule guarantees. **Project-vision re-cast fixture-proven**: a `TempDir` test shows the vision as an editorial-merge synthesis superseding a faithful 1:1 `project-plan` node, replacing `vision_from_plan`'s bespoke body-patch with the modeled mechanism — no live write. **L-8b cleared**: ODD-0013 `state: Draft`→`Accepted` (the cc-prompt's own example); ODD-0017/0018 also corrected to `Accepted`, their target confirmed by citation evidence (both cited as settled authority by multiple already-closed, already-built arcs) since the audit didn't name their target explicitly — disclosed judgment call. All three `git mv`'d `01-draft/`→`04-accepted/`, clean renames, history preserved. **Fixture/doc only — `.worktrees/odm` untouched** (still `2fc25f5`); the live vision mint + every drifted node's reconcile (incl. these 3 ODDs') is **s13**'s (renumbered — see the s12 row: s12 became the reconcile *capability*, fixture-only, with the live close split out to a new s13, mirroring the s09/s10 capability/live-run split earlier in this arc). `release/1.0.x` commits `38941a8` (code + the L-8b bare renames) → `d80cdb3` (ledger/report/bubble-up) → `6664f9f` (F-7's actual content — a self-caught correction: `38941a8` staged only the rename, not the state/version-history edits; caught via `git status` before reporting done; disclosed in `slice11-synthesis-l8b/ledger.md`). | no (fixture/doc-only) | s02, s03 |
| **s12 — reconcile capability** ✅ CLOSED 2026-07-29 + CDC-verified PASS 2026-07-29 | **Re-split from the originally-planned single "reconcile run" slice** (renumbered: capability here, live close now **s13** — the same split this arc already used for s09/s10). Re-snapshot mode, both node families: a drifted non-stub is updated in place (`id`/`edges`/`status` preserved) instead of skipped (design/research: `mapping::backfill_source`'s `Drifted`-skip → re-snapshot, reported in a new `reconciled` bucket) or hard-rejected (work-tree: `selfhost::reconcile_source` gained `force_resnapshot`, `repair()`'s own policy unchanged). **Moved-source re-discovery**, both families: new `mapping::reconcile_source` (design/research) + `selfhost::reconcile` (work-tree) re-find a node whose stored path no longer resolves by identity (`number`/`(type, number)`) and rewrite `source.paths` to the new, s08-relative location — covers the exact live shape ODD-0013/0017/0018 are now in after s11's L-8b moves. **Living-plan-node policy decided** (reconcile-to-current, no special exclusion — ODD-0025 §2.9 new) and fixture-proven against repeated source edits. **Vision-apply path promoted**: `synthesis::apply_project_vision` lifts s11's inline fixture mechanism into reusable library code. **ODD-0025 §4 resolved**: `artifact/v1.0`→delivered `artifact/v1.1`, safe now that this slice's own reconcile mechanism exists to absorb node #25's resulting drift. Fixture/doc only — `.worktrees/odm` untouched, still `2fc25f5`. One commit `3daf893` (`release/1.0.x`). | no (fixture-only) | s02, s03, s11 |
| **s13 — live reconcile + vision mint** ✅ CLOSED 2026-07-30 + CDC-verified PASS 2026-07-31 | Fired s12's capability on the **live** `.worktrees/odm` corpus behind the snapshot → dry-run → adjudicate → fire → verify protocol: 2 re-snapshotted (ODD-0013/0020, s10 legacy body drift) + 3 source-reconciled (ODD-0017/0018 moved+drifted, ODD-0025 drifted-only) + 2 source-reconciled (arc-plan.md, slice10's slice-doc — living-plan-node drift) + 3 ordinary self-host creates (slice11/12/13's own plan nodes). Fired the vision mint: `#1001` a faithful, hash-clean 1:1 `project-plan` node (verbatim, git-derived dates); `#1000` re-cast as the editorial-merge synthesis superseding it, attestation recorded, `supersedes` lineage clean. **Two real bugs caught pre-commit**: `vision()`'s first draft required a `source` field the project structurally never carries (fixed to read `project-plan.md` fresh); `build_synthesis` doesn't stamp schema, so the re-cast silently dropped `#1000`'s schema until an unrelated upgrade pass patched it back — caught by the pre-commit idempotence re-run, fixed, store reset to `2fc25f5` and re-fired clean. One commit `26bea1d` (`odm` branch) atop `2fc25f5`; four commits on `release/1.0.x` (wire + 2 fixes + coverage). **Disclosed deviation**: `check` is exit 1 (14 pre-existing `uncovered-doc` errors for s11/12/13's own artifact docs — confirmed identical at `2fc25f5` before this slice, not a regression; routed to the arc-close). **yes (live)** | s12 |

*Sizing note:* the heaviest are **s05**/**s06** (identity + live run) and **s07** (children-mint + check-wiring); the live-mutation slices are
**s06**/**s07** — split at slice-activation if an open set won't fit
one context. s04 was split from its live run (s05) at activation (2026-07-27) precisely so the
destructive op is fixture-proven before it fires.

## Arc ledger (composition rows — opens here, closes in `closing-report.md`)

> LEDGER-DISCIPLINE v2.0 §B. Class-(b) rows are **reproduced at arc scale** (an end-to-end run on
> odm's corpus). Status ladder: `asserted < attested < reproduced < reconciled`; a `done` row
> reaches ≥ `reproduced`. All rows open **planned**.

| ID | Criterion | Verify | Significance | Status |
|----|-----------|--------|--------------|--------|
| MF-1 | Doc-coverage check exists and is green on `1.0.x/docs` — every `.md` has a node | run the check on the corpus | serious (no file left behind) | **done** — s01's detector; s09 built the enforced `check` rule; **s10 fired it live**: `[coverage] scan_root = "docs"` active in the committed `.worktrees/odm/config.toml`, `odm check` exit 0, `odm migrate docs --coverage` reads **371/371 covered, 0 uncovered** — the whole `docs/` tree, not a narrowed scope, made achievable by s10's index/template exclusion + note mint-all (`slice10-coverage-live-run/closing-report.md`); reproduced by direct read of commit `355404a`. **s10 iteration 1:** CDC found 4 of the newly-minted nodes carried absolute `source.paths` (masked from `check` by the coverage matcher's own tolerance) — fixed + `check` now carries its own unconditional `absolute-source-path` rule, so this specific gap can never again pass silently; reproduced by direct read of commit `2fc25f5` |
| MF-2 | Body-hash gate green across all migrated nodes; zero stub bodies remain | re-migrate + gate; count stubs = 0 | serious | **done** — s07's live run: 0 stub bodies on the committed `odm` corpus (was 44); a second `migrate` run surfaces 0 `BodyHashMismatch` (`slice07-live-run/closing-report.md`); s08's path rewrite (`slice08-source-path-portability/closing-report.md`) touched no body — still 0 stubs, still 0 `BodyHashMismatch` on re-verify; reproduced by direct read of commit `7226797` |
| MF-3 | Every migrated node carries a `source` sub-map (+ preserved `author`/`version`) | grep/`check` over the store | correctness | **done** — s07's live run made every eligible plan-node (project/arc/slice) source-bearing (project + the retired tombstone correctly excluded by design, ODD-0025 §2.3); `author`/`version` N/A (frontmatter-less plan corpus). s08 made those `source` sub-maps portable (`docs/…`-relative). s10 fired the design/research backfill live (12 of 14), disclosing 2 drift cases (ODD-0013/0020) `backfill_source`'s hard gate correctly declined to backfill over. **s13 fired the reconcile live**: those 2 re-snapshotted, plus ODD-0017/0018 (moved+drifted) and ODD-0025 itself (drifted) source-reconciled, plus the s08-found active-arc-node drift (`arc-plan.md`) and slice10's own slice-doc reconciled — **all** eligible nodes now source-bearing *and* current, 0 drifted remaining (recomputed against current sources); reproduced by direct read of commit `26bea1d` (`slice13-live-reconcile/closing-report.md`) |
| MF-4 | Frontmatter-fidelity check green over originally-present fields | run the check | correctness | planned — mapping specified in ODD-0025 §2.4 |
| MF-5 | All arcs (incl. the 6 previously-excluded) + all slices represented | count dirs vs nodes = 0 gap | serious | **done** — s07's live run: all 12 plan-tree arcs + their slices now have nodes; doc-coverage set-difference over `source.paths` = 0 uncovered arc-plans/slice-docs (`slice07-live-run/closing-report.md`); s08 confirmed this stays true after the path rewrite (source-based matching is unaffected by the path *form* change, `slice08-source-path-portability/closing-report.md`) and additionally proved it's now **portable** — the same 0-uncovered result holds from any checkout root, not just the authoring machine; reproduced by direct read of commit `7226797`. **Caveat resolved, was disclosed as a gap:** `coverage.rs`'s `representation()` heuristic could not resolve a named arc's number from its directory name alone, so `--coverage`'s summary line read "8/12" even though the exact criterion (doc-coverage) was 12/12 — **s09 fixed this** (`resolve_arc_dir_numbers` replays the named-arc collision key, CDC v2.8 Finding 2, `slice09-coverage-enforcement/closing-report.md`); a named arc **with** a node now reads represented. **s10 confirmed live**: post-mint `--coverage` reads **0 arc dir(s) + 0 slice dir(s) unrepresented** (was the historical "8/12" undercount pre-s09); reproduced by direct read of commit `355404a` |
| MF-6 | Supporting-doc children minted; doc-coverage wired into `odm check` | `check` fails on a seeded uncovered doc | serious (loud-hole guard) | **done** — s09 built **mint-all** (`mint_artifacts`, ODD-0025 §2.6, no exemption incl. `coverage-report.md`) and the enforcing `check` rule; **s10 fired it live**: 254 `artifact` nodes minted (commit `355404a`), `[coverage] scan_root` active, `odm check` exit 0. Widened beyond the original criterion's `.md` scope by s10's operator-directed additions: `docs/dev/**` also covered via 31 minted `NodeType::Note` nodes, and `docs/design/index.md`/templates correctly excluded rather than counted as gaps (`slice10-coverage-live-run/closing-report.md`); reproduced by direct read of commit `355404a` |
| MF-7 | Synthesis lands as supersede lineage; project vision re-cast; bidirectional guaranteed | inspect edges; seed + verify | correctness | **done** — s11 built the `Vec`-typed `supersedes` + bidirectional-lineage `check` rule + both synthesis regimes; s12 promoted the vision-apply mechanism to reusable library code. **s13 fired the vision mint live**: `#1001` a faithful, hash-clean 1:1 `project-plan` node (verbatim `project-plan.md`, 426/426 lines byte-identical); `#1000` re-cast in place as the editorial-merge synthesis superseding it, with a recorded attestation and a clean `supersedes` edge (`check`'s lineage rule green); reproduced by direct read of commit `26bea1d` (`slice13-live-reconcile/closing-report.md`) |
| MF-8 | L-8b: ODD-0013/0017/0018 (+0019/0020) reconciled to authoritative states | inspect states | pre-ship gate | **done** — s11 corrected the doc-tree states (ODD-0013/0017/0018 → `Accepted`, `git mv`'d into `04-accepted/`). **s13 reconciled the corresponding node gate vectors live**: ODD-0013/0020 re-snapshotted, ODD-0017/0018 re-discovered by identity and their `source.paths` rewritten to the new `04-accepted/`-relative location (both resolve); reproduced by direct read of commit `26bea1d` (`slice13-live-reconcile/closing-report.md`) |
| MF-9 | **Compose:** odm self-hosts *faithfully* — `check`/`orient`/`rollup` green on a corpus with real bodies, full coverage, source records | project-scale reproduce at arc close | serious (P-12) | **done** — s13 fired the reconcile + vision mint live: 0 drifted nodes remain, the vision is live, `orient`/`rollup` byte-stable, fully re-run-idempotent. **The disclosed doc-coverage gap is now closed too**: a new `migrate --all` (built the same day, prompted by the operator asking why nothing composes the five separate derivations into one idempotent pass) fired live — 2 nodes reconciled (this session's own in-flight edits to `arc-plan.md` and ODD-0020, both living-plan-node drift) + 14 artifact nodes minted (the s11/12/13 gap). `odm check`: **exit 0, 0 errors** (was 14); `388/388` docs covered (was 374/388). Re-run verified idempotent (0/0/0 every sub-step) before commit. **Composition + the P-12 acceptance demonstration are the arc-close's job** — the corpus they'll demonstrate against is now fully faithful and fully covered. Reproduced by direct read of commit `e06fffe` (`odm` branch) |

## Version History

### v2.25 — 2026-07-31 — s13 CDC-verified PASS (live reconcile + vision mint)

**CDC verification: PASS** (`slice13-live-reconcile/cdc-verification.md`, 10 rows). Reproduced by direct
git read against the **shipped** store `odm@e06fffe` (387 nodes) — not the `26bea1d` CC's ledger closed
against; the two operator-directed follow-ups (`e1e94bf` heading/`wrong-type-field` fix, `e06fffe`
`migrate --all` gap mint) are part of HEAD (CDC-F2, LOW: the slice docs close against `26bea1d`, the
arc-close report should narrate the true chain). Independent recompute of the §2.1 gate vs current
sources: **383 faithful / 2 drifted / 0 moved / 1 synthesis / 1 sourceless / 387 total**. The 2 drifts
(`#58837400` arc-plan node, `#509907700` slice10 ledger) are the **by-design living-plan tail** — sources
edited after the reconcile, §2.9-sanctioned, `check`-invisible. Vision reproduced byte-1:1 (`#1001` ==
`project-plan.md`, SHA `ae2dcfe895a9`) under an attested editorial-merge synthesis (`#1000`, `supersedes`
→ `#1001`). Doc-coverage gap **reproduced closed**: 0/20 arc-fidelity slice1x docs uncovered, 385/386
scanned covered (1 benign retired-tombstone fallback). ODD-0020 v1.4 synthesis carve-out ratified (keyed on
`source.synthesis`, not `node_type`). CDC disclosed its **own** corrected scan error (whole-file vs
body-after-fence) and confirmed CC's three honest close-disclosures (self-caught schema drop, `check`
undercount, `wrong-type-field` regression). **CDC-F1 (routed, not a defect):** the living-plan tail must be
closed by the arc-close **final reconcile-and-freeze** before MF-9/P-12 is claimed. **MF-7 done; MF-9
fidelity true on the live corpus pending the freeze. The arc-close is next.**

### v2.24 — 2026-07-30 — `migrate --all` built and fired live; the disclosed doc-coverage gap closed

**Same investigation, same day, one more step.** After the v2.23 correction, the operator asked why
`odm migrate` had no single command to idempotently reingest everything — self-host, design/research
reconcile, `--artifacts`, `--notes`, and `--vision` had always been five separate invocations, and it
was exactly that gap (nothing reminding the operator `--artifacts` needed a re-run) that let s11/12/13's
own artifact docs sit uncovered for three slices.

**Built `migrate --all <docs-root>`** (`release/1.0.x@3d4d8fd`): composes all five into one pass,
resolving `docs_directory`/`dev_directory` from the store's `config.toml` rather than requiring them
typed out, with a new `[legacy]` fallback section for onboarding projects that still have a pre-split
odm.toml (`odm store init` now ports pre-split settings forward automatically when it finds them —
confirmed via `main` branch's own original `odm.toml` that nothing was lost for *this* repo at the RH
C-5 cutover; the `docs_directory` narrowing from `"./docs"` to `"./docs/design"` was a deliberate
rebuild-era taxonomy change, not a loss). `--vision`'s step is best-effort: a plan set with no
Definition-of-done section yet is skipped, not a reason to fail the other four steps.

**Fired live**: dry-run adjudicated (2 reconciles — this session's own in-flight edits to `arc-plan.md`
and ODD-0020, both living-plan-node drift — + 14 artifact mints, exactly the disclosed gap), fired as one
commit `e06fffe` atop `e1e94bf`, re-run verified idempotent (0/0/0 every sub-step) before that commit.
**`odm check` is now exit 0, 0 errors** (was 14). `388/388` docs covered (was 374/388). **MF-9 moves to
done** — every disclosed s13 deviation is now resolved, not just tracked.

### v2.23 — 2026-07-30 — Post-close correction: s13's own verification undercounted `check` by one finding

**Same day as s13's close.** The operator, reconciling `find docs -name '*.md' | wc -l` (388) against
`odm node list --all`'s footer (373), asked a question whose investigation surfaced that s13's closing
evidence for F-6/F-7 was wrong, not just incomplete: `odm check` was reporting **15** errors post-fire,
not the 14 recorded — the 15th a genuine, undisclosed regression (`[wrong-type-field]` on `#1000`:
`supersedes` is document-only per ODD-0020 §2, but `project` is a work type). A latent conflict between
ODD-0025 §2.3 (project re-cast as synthesis, needs `supersedes`) and ODD-0020 §2 (work nodes can't carry
it) that no prior slice had exercised on the same node through `check` — s11's synthesis fixtures never
ran `NodeType::Project` through `content_validity`.

**Resolved, operator-approved** (of three options presented — a `check.rs` carve-out, reconsidering the
re-cast's edge shape, or reverting the live fire — chose the carve-out as smallest and most consistent
with the already-Accepted model): `check_field_validity` now exempts a work node carrying
`source.synthesis` from the `supersedes`/`affects` checks, keyed on that field rather than on
`NodeType::Project`. **ODD-0020 → v1.4** records the decision; a new fixture test proves both
directions. No live store data changed — `26bea1d` was always correct under the *intended* model, only
the validator's rule was wrong (`release/1.0.x@83acedb`). `check` on the committed store now shows
exactly 14 errors — the pre-existing `uncovered-doc` gap the original evidence claimed, confirmed for
real this time.

**A related finding, same investigation, resolved same day:** `#1000` and `#1001` both triggered a
`[no-vision]` warning. Two causes: `apply_project_vision`'s body never carried the literal
`# Vision` heading L-3b/`orient` require (fixed via a shared `synthesis::vision_body()` helper; the
live `#1000` refreshed in place, `odm@e1e94bf`); and L-3b itself checked every `NodeType::Project`
node — safe before the vision mint, broken once it created a second (`#1001`, whose verbatim 1:1
body can never carry an injected heading) — fixed by exempting a superseded project node
(`release/1.0.x@e4508e2`). `check` on the committed store: 14 errors (unchanged), 8 warnings (was
10). Full account: `slice13-live-reconcile/ledger.md`'s "Post-close correction" section.

**Named here because this is exactly the kind of thing LEDGER-DISCIPLINE's closure-discipline exists to
catch and not bury**: CC's own "done, verified" claim for F-6/F-7 was false by one finding at the moment
it was written, caught only because the operator asked a question CC's verification hadn't. Tracked as a
correction to the child slice's already-closed record, not a silent rewrite of it.

### v2.22 — 2026-07-30 — s13 closed: live reconcile + vision mint fired (CDC verification pending)

**Fired s12's reconcile capability + the new vision mint on the live `.worktrees/odm` corpus** — the
arc's final live mutation. One commit `26bea1d` (`odm` branch) atop known-good `2fc25f5`, behind the
established snapshot → dry-run → adjudicate → fire → verify protocol. 8 nodes reconciled in place (2
re-snapshotted: ODD-0013/0020; 3 source-reconciled: ODD-0017/0018 moved+drifted, ODD-0025 drifted; 2
source-reconciled: `arc-plan.md`, slice10's slice-doc — living-plan-node drift), 4 created (slice11/12/13's
own plan nodes + the new `#1001` 1:1 `project-plan` node). `#1000` re-cast as the editorial-merge vision
synthesis superseding `#1001`, attested, clean lineage. `release/1.0.x` gained four commits: `595f24f`
(the thin CLI wiring — `--vision` flag, `selfhost::reconcile`/`mapping::reconcile_source` wired into
`migrate`), `dffd3d4` and `2533330` (two real bugs the dry-run and pre-commit idempotence checks caught
before either reached the live store — see below), `d949786` (coverage for the new code's two previously
untested branches).

**Two bugs found and fixed before commit, exactly where the protocol says they should be caught:**
(1) `vision()`'s first draft required the project node's own `fm.source()`, which the project
structurally never carries (ODD-0025 §2.3 excludes it from source-population, same as a retired node) —
the dry-run failed outright; fixed to read `project-plan.md` fresh off disk, the way every other migrated
node is sourced. (2) `build_synthesis` never stamps schema, so the re-cast `#1000` silently lost its
existing schema marker — invisible until an unrelated `migrate` upgrade pass patched a schema back in on
the next run; caught by the pre-commit idempotence re-run (required by F-6), fixed by stamping the
synthesis frontmatter explicitly, and the store was reset to `2fc25f5` and re-fired clean rather than
hand-patched. Neither fixture test suite caught either bug — both only show up against a corpus with real
history a from-scratch `TempDir` fixture can't reproduce, which is the concrete case for why this arc
insists on a live-run slice, not just capability fixtures.

**One disclosed deviation, confirmed not a regression:** `odm check`'s `absolute-source-path` rule is 0
findings, but `check` overall is exit 1 — 14 pre-existing `uncovered-doc` errors for slice11/12/13's own
artifact-family docs, uncovered since s10's last `--artifacts` mint-all run. Checked out `2fc25f5` in a
disposable worktree and ran `check` against that exact pre-fire state: identical 14 errors, identical exit
1 — not something s13 caused. Routed to the arc-close (a fresh `--artifacts` run before the P-12 demo).

**MF-3, MF-7, MF-8 move to done** (all previously "capability-level"/"mechanism-ready" language now
backed by the live-committed corpus). **MF-9 moves to "fidelity true on the live corpus, doc-coverage
caveat open"** — composition and the P-12 acceptance demonstration are the arc-close's job now that the
corpus they demonstrate against is ready. **The arc-close is next**: MF-9 composition + the P-12 self-host
acceptance demo + a fresh `--artifacts` run to close the F-7/doc-coverage gap. Full detail:
`slice13-live-reconcile/ledger.md` + `closing-report.md`.

### v2.21 — 2026-07-29 — s12 CDC-verified PASS (reconcile capability)

**CDC verification: PASS** (`slice12-reconcile-capability/cdc-verification.md`). Fixture/doc slice —
reproduced structurally on `release/1.0.x`, runtime rows attested-by-CC -> CI: `reconcile_source` now
re-snapshots a drifted non-stub in place (id/edges/status preserved, gate re-runs against the current
source) instead of skip/reject; moved-source re-discovery re-finds a relocated file by its identity key
(not a stored path) and rewrites `source.paths`; `synthesis::apply_project_vision` promoted to reusable
library code — all with non-vacuous tests (re-snapshot, directory-moved, repeated-edit living-plan case,
idempotence). The living-plan-node policy is decided + recorded as a well-formed **ODD-0025 §2.9**
(reconcile-to-current, no exclusion; the gate stays migration-time-only), resolving the s08 open question;
**§4 resolved** to `artifact/v1.1` (the s09 LOW follow-up), with node #25's self-referential drift from the
ODD edit disclosed + routed to s13. No live mutation (`odm` HEAD unchanged at `2fc25f5`); §2.9 is the one
cited model addition. No findings.

**s13 (live reconcile + vision mint) is next** — fire the capability on the live corpus (re-snapshot the
arc node, ODD-0013/0017/0018/0020, and #25; mint the vision synthesis), snapshot->dry-run->fire. After s13
CDC-closes, the **arc-close**: MF-9 composition + the P-12 self-host acceptance demonstration + the arc
closing-report + project bubble-up. **Surfaced by: slice 12.**

### v2.20 — 2026-07-29 — s12 closed: reconcile capability built (fixture-only); re-split into s12/s13

**Slice 12 is closed** (`slice12-reconcile-capability/closing-report.md`; 10/10 ledger rows done,
fixture-attested throughout — no live class-(b) row, `.worktrees/odm` untouched, still `2fc25f5`).
Built the capability the arc's "faithful *and* repeatable" charter needed: a drifted non-stub node is
now re-snapshotted in place (`id`/`edges`/`status` preserved) instead of skipped (design/research,
`mapping::backfill_source`) or hard-rejected (work-tree, `selfhost::reconcile_source`'s new
`force_resnapshot` flag) — new `mapping::reconcile_source` and `selfhost::reconcile` handle the
already-sourced case for each family, including moved-source re-discovery by identity (the exact shape
ODD-0013/0017/0018 are now in after s11's L-8b moves). The **living-plan-node policy** is decided
(reconcile-to-current, no special exclusion) and fixture-proven against a source that keeps changing
between runs — recorded in a new **ODD-0025 §2.9**. The project-vision re-cast is promoted from s11's
inline fixture mechanism into reusable library code (`synthesis::apply_project_vision`). **ODD-0025 §4
resolved**: `artifact/v1.0` → the delivered `artifact/v1.1`, safe to amend now that this slice's own
reconcile mechanism exists to absorb node #25's resulting drift (deferred in s10 iteration 1 for
exactly the opposite reason).

**Re-split, mirroring the s09/s10 capability/live-run pattern**: the arc-plan originally carried a
single "s12 — reconcile run" row assuming capability + live fire + composition all in one slice. The
actual cc-prompt this session received scoped s12 to **capability only** (fixture-only, no live store
mutation), with a new **s13** carrying the live reconcile + vision mint, and the arc-close (MF-9
composition + P-12 demo) following s13. Applied here: the old s12 row is replaced with the real s12
(capability, closed) + a new s13 row (live close, next); every "s12" forward-reference in s11's own row
is corrected to "s13".

One commit, `3daf893` (`release/1.0.x`). **MF-9 moves to "mechanism ready"** — composition and the P-12
acceptance demonstration remain gated on s13's live-reconciled corpus. **s13 (live reconcile + vision
mint) is next**; the arc-close (MF-9 composition + P-12 demo + the arc `closing-report.md` + the
bubble-up to `project-plan.md`) follows it.

### v2.19 — 2026-07-29 — s11 CDC-verified PASS (synthesis + L-8b)

**CDC verification: PASS** (`slice11-synthesis-l8b/cdc-verification.md`). Fixture/doc slice — reproduced
structurally on `release/1.0.x`, runtime rows attested-by-CC -> CI: `supersedes`->`Vec` (the `if let Some`
at `lib.rs:539` is the legacy-parser side, confirmed by compilation, not a missed conversion); the
branching-safe supersession-cycle `check`; the `synthesis` module's two regimes (concatenation
deterministic-join hash gate; editorial-merge attestation) — all with non-vacuous tests; the vision re-cast
fixture asserts the 1:1 faithful `project-plan` body, the recorded attestation, the supersede edge, clean
lineage, and store-untouched. **L-8b content verified landed:** ODD-0013/0017/0018 carry `state: Accepted`
in `04-accepted/` (the self-caught staging gap — `38941a8` missed the content, fixed in `6664f9f`,
disclosed in `6046101` — genuinely closed). No live mutation (`odm` HEAD unchanged at `2fc25f5`).
0017/0018->Accepted judgment ratified (the audit named all three for accept). Two LOW/optional notes: echo
the concrete deterministic join into ODD-0025 §2.3 for discoverability; `Accepted` reads oddly for the 0018
research doc (disclosed, out of L-8b scope). The self-caught staging gap is a Safety-II positive — the
recover-cleanly discipline working.

**s12 (reconcile run) is next** — the arc's full live close: fire the vision synthesis; reconcile every
drifted node (s08 active-arc node, ODD-0013/0020, the §4 re-snapshot, and s11's L-8b-moved
0013/0017/0018); the composition + P-12 acceptance demonstration. **Surfaced by: slice 11.**

### v2.18 — 2026-07-29 — s11 closed: synthesis capability + L-8b cleared (fixture/doc-only)

**Slice 11 is closed** (`slice11-synthesis-l8b/closing-report.md`; 10/10 ledger rows done, fixture-
attested throughout — no live class-(b) row, since nothing in this slice touches `.worktrees/odm`).
`edges.supersedes` is now a `Vec<Supersedes>` (ODD-0025 §2.3), every read site updated, with a rewritten
`check_supersession` guaranteeing bidirectional lineage over a branching relation (a node can supersede
many targets, not a single-successor chain) — 6 new multi-target fixtures. A new `odm_migrate::synthesis`
module implements both verification regimes ODD-0025 §2.3 names: `concatenation` hash-gated against a
now-concretely-defined deterministic join, `editorial-merge` requiring an explicit recorded attestation
plus the lineage the `check` rule guarantees. The project-vision re-cast is fixture-proven: a `TempDir`
test shows the vision as an editorial-merge synthesis superseding a faithful 1:1 `project-plan` node,
replacing `vision_from_plan`'s bespoke body-patch with the modeled mechanism.

**L-8b cleared**: ODD-0013's `state: Draft`→`Accepted` (folding in its already-`Accepted` 0019/0020
amendments, the cc-prompt's own worked example). ODD-0017/0018 also corrected to `Accepted` — their
target wasn't literally named by `uat-coverage-audit.md` §L-8, so it was confirmed by citation evidence
(both are the design authority multiple already-closed, already-built arcs depend on) rather than
guessed, disclosed as a judgment call. All three `git mv`'d `01-draft/`→`04-accepted/`, clean renames,
history preserved.

**Three `release/1.0.x` commits**: `38941a8` (code F-1…F-6 + the L-8b bare renames, message describing
only the code — a disclosed minor deviation), `d80cdb3` (this bubble-up), and `6664f9f` — a **self-caught
correction**: `38941a8` turned out to have staged only the bare file *rename* for the three L-8b docs,
not their `state:`/version/Version-History content edits (`git status`'s two-letter `RM` code — staged
rename, plus an unstaged modification on top — was misread as fully staged). Caught via a routine
post-commit `git status` check before reporting the slice done, fixed in `6664f9f`, verified to now match
exactly what `ledger.md`/`closing-report.md` always described. **`.worktrees/odm` untouched throughout**,
still `2fc25f5`.

**MF-7 and MF-8 move to done at the capability/doc level.** The corresponding node gate vectors (for
ODD-0013/0017/0018) and the live vision synthesis itself are **s12**'s — s12's scope now explicitly
includes reconciling these 3 newly-moved ODDs alongside the previously-routed drift cases (the s08
active-arc-node, ODD-0013/0020's legacy body drift, the ODD-0025 §4 re-snapshot). **s12 (reconcile run)
is next — the arc's full live close**, and with it the composition / P-12 acceptance demonstration.

### v2.17 — 2026-07-29 — s10 CDC-verified PASS (incl. iteration 1); portability regression fixed + now enforced

**CDC verification: PASS** (`slice10-coverage-live-run/cdc-verification.md`), post-iteration-1. Reproduced
by direct read of the committed store: the live run (`odm@355404a`) — 291 adds (254 artifact + 4 imported
ODDs + 31 note + 2 plan) + 12 design/research backfills, **zero collateral** to the 62 pre-existing nodes,
project/retired last-touch unchanged, gate flipped (`[coverage] scan_root="docs"`), git-derived dates real,
31 notes subdir-tagged + uncontained, the 2 drifted design nodes genuinely drifted (0.81/0.64 similar) and
correctly declined -> s12. **371/371 coverage independently reconciled** — my source-only set-difference
flagged 9, all resolved (8 structural-fallback-covered, 1 scan time-skew), so the green is genuine.

**The iteration-1 finding, verified fixed:** CDC's independent reproduction caught 4 imported ODDs (#22-25)
carrying **absolute** `source.paths` (s08 Finding-1 regressed on the `mapping::build_node` seam, masked by
the coverage matcher's relativize-tolerance). Iteration 1 reproduced clean: **0 absolute remain** (was 4),
the 4 rewritten path-string-only (bodies/ids/schema/dates byte-identical, `2fc25f5` atop `355404a`), the
seam fixed at mint time, and — the durable win — an **unconditional `absolute-source-path` check rule**
(also catching `.worktrees/`-anchored paths) that makes portability an enforced invariant, proven against
the real regression (4->0) before firing.

**The five operator-directed additions ratified** (disclosed, fixture-first, operator-authorized); named
process cost: F-11/F-12/F-14 were capability landed in a live-run slice, so CDC verified their live
outcomes extra-carefully rather than inheriting fixture attestation — they hold.

**Routed to s12:** the 2 drifted design nodes (0013/0020) **and** the ODD-0025 §4 re-snapshot (editing §4
would drift node #25's own body — it must be a reconcile, not an edit; this also carries the s09 §4 LOW
follow-up). **s11 (synthesis + L-8b) is next.** **Surfaced by: slice 10 + its CDC verification.**

### v2.16 — 2026-07-29 — s10 iteration 1: CDC found + fixed a source-path portability regression

**CDC verification of v2.15's s10 close found a real, live regression**, not a fixture gap: 4 of the
nodes s10 had *just* minted (#22 store-home, #23 command-surface, #24 id-scheme, #25 migration-fidelity)
carried **absolute** `source.paths` — the exact bug CDC v2.8 Finding 1 named and s08 fixed corpus-wide,
reintroduced on the one seam s08's fix never touched: the design/ODD import path (`mapping::build_node`)
stored `source_path` verbatim, unlike `artifact.rs`/`notes.rs`/`backfill_source`, which already routed it
through `fidelity::relativize`. `odm check` reported green at the v2.15 close only because the
doc-coverage matcher relativizes stored paths *before* comparing — the s08 transition-tolerance masked
exactly the class of regression it was built to *tolerate through*, not to *catch*. This falsified s10's
own F-10 ledger row ("no model drift").

**Fixed, and made durable, same day, as iteration 1 of s10's five-iteration budget**
(`slice10-coverage-live-run/cc-prompt-iteration1.md`, `ledger.md` F-16…F-21,
`closing-report.md` §"Iteration 1"):

1. **The seam** — `mapping::build_node` now relativizes before storing (`release/1.0.x` `ff68186`).
2. **The durable invariant** — `odm_core::check` gained an **unconditional** `absolute-source-path` Error
   rule (no config gate, unlike `[coverage] scan_root` — there is no legitimate absolute case). This is
   the real point: the regression is now structurally impossible to pass `check` silently, not merely
   fixed as a one-time data correction. Proved against the *real* regression before fixing it: the
   freshly-built guard, run against the still-broken live store, flagged **exactly** the 4 known nodes and
   no others.
3. **The live fix** — a new `mapping::canonicalize_source_paths` (`backfill_source`'s complement, for a
   node that already has a *malformed* `source` rather than none at all; `release/1.0.x` `3eddf38`) fired
   the 4-node correction as one revertible commit (`2fc25f5` atop `355404a` on `odm`), behind the same
   snapshot → dry-run → adjudicate → fire → verify protocol as every other live-mutation slice.
   `git diff` confirms exactly one `source.paths` line changed per file — same ids, bodies, schema, dates.

**Two items explicitly skipped and flagged, not silently dropped:** the optional ODD-0025 §4 wording
companion the cc-prompt allowed folding in *if trivial* — declined because it would introduce fresh body
drift on node #25, one of the exact 4 nodes this iteration's hard gate required stay byte-identical; and
an unrelated, pre-existing `ROLLUP.md` staleness discovered incidentally during verification, reverted
rather than fixed here to keep the iteration's diff scoped.

**Arc-plan updated:** status header now cites `2fc25f5` as the live tip (atop `355404a`, atop `7226797`);
MF-1 and MF-3 rows note the durable enforcement / the correction respectively; the s10 slice-table row
records the patch. **s10 remains CDC-verification-pending** — it flips to CDC-verified PASS only once CDC
reproduces, against the committed store and code: 0 absolute `source.paths` remain, and the
`absolute-source-path` rule is present and unconditional. **s11 (synthesis + L-8b) is unaffected and
remains next** once s10's CDC verification completes.

### v2.15 — 2026-07-29 — s10 (coverage live run) closed; coverage enforced live; MF-1/MF-6 done

**Slice 10 is closed** (`slice10-coverage-live-run/closing-report.md`; 15/15 ledger rows done — 10 as
originally scoped, 5 operator-directed additions, all disclosed — fixture-attested code + live,
direct-read-of-the-committed-store evidence for the outcome). Fired s09's capability on `.worktrees/odm`
in one commit (`355404a`, atop known-good `7226797`), behind the full snapshot → dry-run → adjudicate →
fire → verify protocol: 254 `artifact` nodes mint-all, 12 of 14 design/research nodes backfilled with
`source` (2 correctly left untouched — see below), 4 never-migrated ODDs imported, `[coverage] scan_root
= "docs"` activated same-commit. **Result:** `odm check` exit 0, `odm migrate docs --coverage` reads
**371/371 covered, 0 uncovered, 0/0 representation gap** — the *whole* `docs/` tree. No collateral to the
62 pre-existing nodes/project/retired (last-touch unchanged, still `b45b122`); fully idempotent;
`orient`/`rollup` byte-stable across 2 runs.

**Scope grew by five operator-directed items, each discovered live and disclosed, not silently
absorbed:**

1. `docs/design/index.md` + `templates/*.md` excluded from doc-coverage outright (legacy-importer
   infrastructure, never meant to be ingested) — without this, 0-uncovered was unreachable.
2. `docs/dev/**` (31 files) minted as `NodeType::Note`, tagged by immediate subdirectory, deliberately
   uncontained — a new, **general** `odm-migrate` capability (the legacy `Config::dev_directory` field
   recurs across every pre-1.0 odm project), not a one-off for this repo.
3. Every node-creation/reconcile path now stamps **git-derived** `created`/`updated`
   (`odm_store::worktree::first_commit_date`/`last_commit_date`, RH F-20's own mechanism) instead of "the
   day this ran" — a regression the operator had previously flagged and that had never been wired into
   the actual creation paths (only the separate, manually-invoked `--replan` step used it before).
   **Known residual, accepted, not chased further:** this repo's own git history may not reach back past
   its extraction from the Oxur monorepo; `--follow` testing found no rename-tracking discrepancy within
   this repo's own history, so any remaining gap is a cross-repository question outside `git log`'s reach
   here, not a bug in the new wiring.
4. `check_decomposition` (stub / undecomposed-parent / drift) now counts only `is_work()` children, not
   any `part_of` child — the mint attaching artifacts to arcs that had already affirmed `decomposed:
   complete` was reading as decomposition drift on 4 real arcs, which would have blocked a green `check`
   outright. Fixed and reproduced (0 drift findings post-mint, confirmed against a pre-fix reproduction
   in the same session).
5. `odm list` now tree-nests an `artifact` `part_of` a slice **or an arc** under that row, coloured a
   cyan-leaning blue in slice's own saturation/luminosity family, instead of showing it as "orphaned"
   reference material — caught by the operator reviewing the actual live `node list` output. Pure display
   change, no store mutation. Cross-checked against the full live corpus: 250 of 251 artifact-family files
   nest exactly where expected; the one exception is explained, not defective (a retired, sourceless
   tombstone slice predating the source model).

**A new, disclosed finding for s12:** 2 design/research nodes (ODD-0013, ODD-0020) are **drifted**, not
backfilled — both actively amended throughout this rebuild, so their live bodies no longer match their
current legacy source. `backfill_source`'s hard body-hash gate correctly declined to silently backfill
`source` over an unverifiable body (fixed this slice to skip-and-continue per drifted node rather than
abort the whole batch on the first one — a narrow, disclosed refinement of the s09 gate's granularity,
not a weakening of it). Routed to **s12**'s reconcile run, alongside the s08-found active-arc-node drift
— s12 should expect more drifted nodes to surface as the corpus ages, not treat these 2 as exhaustive.

**MF-1 and MF-6 → done** (enforced live, not just capability-complete). **MF-3 → done for the achievable
criterion** (12/14; the 2 drifted nodes are a structural, disclosed exception routed to s12, not a
silent gap). **MF-5's** live representation gap confirmed 0/0. **s11 (synthesis + L-8b) is now unblocked
and next.** **CDC verification status:** pending — this entry records CC's close; an independent CDC
pass (live, direct-read-of-the-committed-store) has not yet run.

### v2.14 — 2026-07-29 — s09 CDC-verified PASS; `artifact/v1.1` ratified; ODD-0025 §4 amendment (LOW) queued

**CDC verification: PASS** (`slice09-coverage-enforcement/cdc-verification.md`). Reproduced structurally
on `release/1.0.x` (`012fad5`…`381acba`) — a fixture-only slice, so runtime rows are attested-by-CC → CI
and there is no live class-(b) row: the `artifact` type + methods, `mint_artifacts` (+ the directory→id
containment index), `mapping::backfill_source`, the config-gated `odm check` doc-coverage Error rule, and
both CDC v2.8 Findings (2: `resolve_arc_dir_numbers`; 3: `provenance_absence` → `source().is_none()`,
stale helpers removed) all confirmed in code; the 20 new tests exist and are non-vacuous. **No live
mutation reproduced:** `odm` HEAD unchanged at `7226797`, no ODD edited, and `[coverage]` absent from the
live `config.toml` (the rule is inert live — the no-red-window seam s10 activates).

**`artifact/v1.1` ratified.** CC stamped the shared global `SchemaVersion::CURRENT` (`{1,1}`), not the
type-local `v1.0` ODD-0025 §4 names — correct, because the schema minor is one global generation counter,
not a per-type axis, and hand-stamping `v1.0` would bypass `SchemaMarker::current`. **LOW follow-up
(spec-keeping):** ODD-0025 §4 still literally reads `artifact/v1.0` and should get a one-line amendment to
match delivery (§4 is the amendment-spec section; leaving it diverged is a small spec-softening). Not a
blocker for s09 or s10.

**MF-1/MF-6** capability is now CDC-verified (live-green pending s10). **s10 (coverage live run) unblocked
and next**, its open set correctly scoped (it adds the confirmed-inert `[coverage] scan_root` key after
minting). **Surfaced by: slice 09.**

### v2.13 — 2026-07-28 — s09 (coverage enforcement, capability) closed; CDC verification pending

**Slice 09 is closed** (`slice09-coverage-enforcement/closing-report.md`; 10/10 ledger rows done,
fixture-attested, 0 deferred, 0 no-op). The enforcement **capability** the v2.12 split carved out now
exists in code: `NodeType::Artifact` (schema stamps the shared `artifact/v1.1`, not a type-local
`v1.0` — flagged, justified in a doc comment, no per-type version table to build one from); a new
`crates/odm-migrate/src/artifact.rs::mint_artifacts` reaches the artifact doc family (mint-all incl.
`coverage-report.md`, containment resolved via a `source.paths`-keyed directory index that needs no
arc/slice-numbering re-derivation, correct for numbered *and* named arcs alike); a new
`mapping::backfill_source` reaches the design/research family (mirrors `reconcile_source`'s
stub-replace / faithful-keep-and-gate / drift-hard-reject policy); `odm check` gained a doc-coverage
`Error` rule, config-gated on a new `[coverage] scan_root` key absent from the live store today —
confirmed (not just asserted) still inert by running `odm check` against `.worktrees/odm` and diffing
(unchanged). **CDC v2.8 Findings 2 and 3, both elevated from s08, are resolved**: `representation()`
now resolves a named arc's number via the same s05 name-derived key `self_host` mints it with
(closing the "8/12" undercount); `provenance_absence` retargeted from a scan for a `provenance:` key
that was renamed to `source:` back at s02 (and is derived-only, never stored, per 0013) to the typed
`source().is_none()` check. 20 new tests; `clippy -D warnings` clean throughout; 0 `unsafe`; no ODD
edited — no model line was needed, only code against the model s02's ODD-0025 already specifies.

No silent drops: every s09-scoped "In" item landed; every "Out" item (the live mint-all, the live
design/research backfill, flipping the check on live, synthesis/L-8b, the arc-close reconcile run)
is confirmed untouched — `.worktrees/odm` stays clean, HEAD unchanged from s08's `7226797`.

**s10 (coverage live run) is now unblocked and next.** MF-1 and MF-6 move from "planned" to
"capability done, live-pending"; MF-3's design/research residual scope likewise has its backfill
mechanism built, live-pending. MF-5's report-clarity caveat (the "8/12" line) is resolved outright,
not just deferred.

**CDC verification status:** pending — this entry records CC's close; an independent CDC pass against
the code + fixtures + CI (no live store to read this slice, per LEDGER-DISCIPLINE v2.0 §B class-(a))
has not yet run.

### v2.12 — 2026-07-29 — s09 split into capability (s09) + live run (s10); downstream renumbered

**Approved by the operator (option A).** s09-as-scoped bundled a new `NodeType` (`artifact`), the
discovery reach for two doc families, the `check`-wiring, the two `coverage.rs` fixes, **and two live
mutations** (mint ~211 `artifact` nodes + backfill the 14 design/research nodes) — an arc's worth of
work in one context, plus a correctness hazard (the enforcing check goes red on the live corpus if it
activates before the mint). Split along the arc's own capability-then-live-run seam (the s04/s05,
s06/s07 precedent):

- **s09 — coverage enforcement (capability)** — the machinery, fixture-only, no live mutation. Open set
  drawn (`slice09-coverage-enforcement/{slice-doc,ledger,cc-prompt}.md`).
- **s10 — coverage live run** (NEW) — fire the mint + backfill + flip the check live, behind the s07
  snapshot → dry-run → adjudicate → fire → verify protocol. Open set drawn (`slice10-coverage-live-run/`).
- **Renumber:** old **s10 (synthesis + L-8b) → s11**; old **s11 (reconcile run) → s12**. The slice table
  and the MF-ledger rows (MF-1/MF-3/MF-6) are updated to the s09-capability / s10-live split.

**Mapping note for prior entries:** references to "s11" (reconcile) and the "s10/s11" living-doc-drift
design question in **v2.11** and earlier use the *pre-renumber* numbering — under this entry the
reconcile run is **s12** and the coverage live run is **s10**. Prior history entries are left as written.

**Surfaced by:** the s08 CDC verification's sizing assessment + the operator's option-A decision.

### v2.11 — 2026-07-29 — s08 CDC-verified PASS (independent reproduction); arc-node drift routed to s11

**CDC verification: PASS** (`slice08-source-path-portability/cdc-verification.md`). Reproduced — not
merely attested — against the committed objects (`odm@7226797` vs known-good `7b4eb57`; code
`release/1.0.x@994d3e5`): 78 nodes / 62 source-bearing / **0 absolute `source.paths`** (was 61) / 0 stubs;
all 62 relative paths resolve against the plan tree; the `7b4eb57`→`7226797` diff is exactly **61
modified (one path line each) + 1 added**, ids/bodies/schema otherwise byte-stable; project (`#1000`) +
retired (`#1605`) untouched (last-touched `b45b122`, pre-s07). The **re-mint guard held**: 0 of 61
re-minted, keyed via `relativize` on both stored and discovered sides in code. **The dry-run's "1
create" was independently re-adjudicated** as slice08's own genuinely-new plan node (`#58837408`, no
prior ULID/number claimant) — CC's investigate-before-firing was correct; its s09+ gate-reading
refinement ("stop and *investigate* any deviation," clearing it before the irreversible step) is
ratified. Runtime rows (`check`/`orient`/`rollup`/`clippy`/`llvm-cov`) stay attested-by-CC → CI (the
macOS binaries won't run on the CDC Linux bridge).

**One reproduced observation, routed — not an s08 defect.** The active arc node (`#58837400`,
`source → arc-plan.md`) shows body-hash drift vs the live arc-plan (127 lines of status/version-history
growth; 0.85 similar; a full 377-line body, not a stub). s08 changed only its path line; the drift is the
living-doc reconcile case already scoped to **s11** — and, structurally, the *active* arc's node can
never be body-faithful while its arc is in flight (this very entry widens the drift). **Open design
question for s10/s11:** reconcile a living-plan arc node by re-snapshot on every edit, or treat it
specially like the project synthesis node (excluded from 1:1 `source`)? Only the active arc drifts; the
11 closed arcs match. (Surfaced by: slice 08.)

**s08 → CDC-verified PASS.** CDC v2.8 Finding 1 confirmed resolved. **s09 (coverage enforcement)
unblocked and next.**

### v2.10 — 2026-07-28 — s08 (source-path portability) closed; CDC v2.8 Finding 1 resolved

**Slice 08 is closed** (`slice08-source-path-portability/closing-report.md`; 10/10 ledger rows done, 0
deferred, 0 no-op; attested-by-CC, CDC reproduction pending). Delivered exactly what v2.8/v2.9 scoped:
`source.paths` is now repo-content-root-relative and canonical (`docs/…`, never `.worktrees/1.0.x/…`),
anchored on the plan tree's own git toplevel via one shared function (`fidelity::relativize`/
`resolve_from_anchor`) used for both writing and resolving; `self_host`'s `by_source` matching
canonicalizes whatever is stored — absolute or relative — to the same key as the freshly-discovered
path, so the live rewrite (one commit, `7226797` atop `7b4eb57`) matched and corrected all 61
previously-absolute nodes in place with **zero** re-mints and **zero** collateral change to any
body/id/schema (confirmed: the project and retired nodes are still last-touched at the pre-s07 cutover
commit, `b45b122`). One additional node — slice08's own plan node, genuinely new since s07 — was
imported in the same pass. **What implementing it revealed:** (1) the dry-run's hard "0 creates" gate
showed 1 create; investigated rather than either blindly stopping or blindly firing, it turned out to be
slice08's own plan-set directory (drawn after s07 closed) — the transition guard was proven *working*
by the fact all 61 pre-existing nodes were correctly matched-and-rewritten, not re-created; (2)
`coverage.rs` needed the same anchor treatment as `selfhost.rs` despite not being named in the
cc-prompt's file list — required by the ledger's own F-6 cross-root-coverage-stability criterion, caught
by reading the ledger literally before writing code; (3) the two report-clarity findings s07 disclosed
(`representation()`'s named-arc gap, the stale `provenance_absence` key) were confirmed still present,
unchanged, still correctly deferred to s09. MF-2/MF-3/MF-5 (already `done` as of s07) stay done — s08
strengthens *how* that outcome is stored (a portable identity key) without changing what was proven true
about the corpus's content. **s09 (coverage enforcement) is unblocked and next.**

### v2.9 — 2026-07-28 — source-path spec tightened; relativization split into its own slice (s08); renumber

Operator confirmed **relative** `source.paths`: the determinism-across-machines property that makes
odm's migration *repeatable* forbids absolute — two collaborators ingesting the same files must get
identical results, which only relative paths can guarantee. Spec **tightened** (beyond "relativize"):
relative to the **repo content root** (`docs/…`, **not** the checkout/worktree root — else
`.worktrees/1.0.x/` gets baked in, its own portability bug); **canonicalized** (forward slashes; a
decided case rule for macOS↔Linux CI); **one shared anchor** (the docs tree's git toplevel) for write
*and* resolve, so it round-trips cross-worktree (store on the `odm` branch, docs on `1.0.x`) and
cross-checkout. Because the fix carries a **live corrective re-migration** (rewriting the 61 committed
nodes), it becomes **its own slice — s08, source-path portability** — ahead of coverage enforcement,
which would otherwise be built on absolute paths (same "capability, then live run, unbundled" shape as
s06/s07). **Downstream renumbered: old s08/s09/s10 → s09/s10/s11** (coverage enforcement / synthesis +
L-8b / reconcile run). The s07 CDC verification (v2.8) is unchanged; the living-doc reconcile note it
recorded now lands in **s11**.

### v2.8 — 2026-07-28 — s07 CDC-verified PASS; absolute-`source.paths` finding elevated into s08

**CDC verification: PASS** (`slice07-live-run/cdc-verification.md`; 11/11 rows). Reproduced — not merely
attested — on the committed store (`7b4eb57`): 0 stubs (was 44), 61 source-bearing (was 0), source-less
= **exactly** the project + the retired tombstone + the 14 design/research nodes; all 12 arcs
represented (independent coverage set-difference = 0 uncovered); the **body-hash gate re-computed
independently** (no stored hash), 60/61 byte-faithful — the 61st being this arc-plan's own node,
benignly stale because the s07 close re-edited `arc-plan.md` *after* the migration snapshot (matched at
CC's verify time). My s06 v2.5 doc-comment finding is resolved verbatim. **CDC ratifies MF-2/MF-3/MF-5
→ done** (already flipped in v2.7; now independently reproduced).

**One of CC's four disclosed findings elevated.** Finding 1 (absolute `source.paths`) is more than a
routine follow-up: `source.paths` is the s05 *identity key*, so machine-/worktree-absolute paths (a)
reintroduce a cross-checkout re-run-**duplicate** hazard — the very fragility s05 killed, via the path
instead of the number — and (b) don't resolve outside the authoring machine (demonstrated: they fail
even inside the device's own VM), breaking the committed self-hosted store's portability and any
coverage-in-CI. Made **s08's lead item** (relativize + rewrite the 61 nodes) ahead of the
coverage-enforcement work that would otherwise build on absolute paths. Findings 2–3 (coverage-report
correctness) fold into the same s08 `coverage.rs` pass; Finding 4 (8 `check` warnings) is real
plan-decomposition debt correctly surfaced, not a defect — no action. **New s10 note:** living plan
docs drift from their migrated snapshots the instant they're re-edited, and `reconcile_source` *rejects*
a non-stub whose body no longer matches source — s10's arc-close reconcile must treat a legitimate
source-doc change as an update-to-re-snapshot, not a drift-to-reject.

### v2.7 — 2026-07-28 — s07 (live repair run) closed; the arc's first live mutation landed

**Slice 07 is closed** (`slice07-live-run/closing-report.md`; 11/11 ledger rows done, 0 deferred, 0
no-op; attested-by-CC, CDC reproduction pending — this is the arc's first live, class-(b) row, so CDC
reproduces by **direct read of the committed store**, not by re-running fixtures). Opened with the CDC
v2.5 doc-comment fix (`release/1.0.x` `b901b12`); captured a before-manifest (60 nodes, 0 source, all
`v1.0`, 1 retired, sha256 fingerprint, known-good SHA `e2ab628`); dry-ran and adjudicated (44
reconciled / 17 created / 46 skipped, exact against the corpus once traced by hand — the slice-doc's
"≈47" was a slightly-loose estimate, not a discrepancy); fired for real and committed the entire
`.worktrees/odm` worktree as **one** commit (`7b4eb57`, atop `e2ab628`, revert = `git reset --hard
e2ab628…`). **Verified on the committed store:** 0 stub bodies (was 44), 61 source-bearing plan nodes
(was 0), all 12 plan-tree arcs represented (0 uncovered arc-plans/slice-docs), 0 `BodyHashMismatch` on
re-verify, `check` exit 0, `orient`/`rollup` byte-stable, fully re-run-idempotent, project and the
retired tombstone both confirmed byte-for-byte untouched. `context.json` deliberately left unchanged
(`migrate` has no code path that writes it). **No gate failed; no rollback was needed.**

**What the live run revealed (four disclosed, non-blocking findings, not fixed here):** (1)
`source.paths` stores absolute, machine-specific paths (`odm-cli/src/migrate.rs::resolve()`,
pre-existing since s01/s03 — fixtures never surfaced it since `TempDir` paths are absolute too, just
ephemeral); (2) `representation()`'s named-arc heuristic gap (disclosed at s05) now visibly produces a
confusing "8/12" in `--coverage`'s summary line even though the exact doc-coverage criterion is 12/12
— a report-clarity gap, not a real one; (3) the `provenance_absence` detector still checks for a
literal `provenance:` key, which ODD-0025 §2.0 renamed to `source:` at s02 — it now reports all 77
nodes "provenance absent" forever, directly contradicting the correct "61 source-bearing" figure one
line above it in the same report; (4) `check`'s 8 new warnings are genuine, pre-existing
plan-decomposition facts (4 newly-visible named arcs genuinely have no slice subdirectories yet in the
plan tree) made visible for the first time now that those arcs have nodes — not migration defects.
Findings 1–3 are recommended follow-ups for whoever next touches `coverage.rs`'s rendering or
`resolve()` (s08 is the natural home for 2–3, since it already touches coverage). **MF-2, MF-3
(plan-node scope), and MF-5 move `planned → done`** — reproduced live, per LEDGER-DISCIPLINE v2.0 §B;
MF-3's design/research-node scope, MF-1, and MF-6 stay **planned**, explicitly handed to **s08**.

### v2.6 — 2026-07-28 — s07 (live repair run) drawn; scope refined to plan nodes, design/research → s08

CDC drew the s07 open set (`slice07-live-run/{slice-doc,ledger,cc-prompt}.md`) and ground-truthed the
live store to pin the run: 60 nodes / **0 source** / all `schema: */v1.0` / **44 stubs (6 arc + 38
slice** — every present arc is a stub) / 6 arcs / `context.json → 01KWXM…`; plan tree has **12** arcs
(6 to import) and 49 slice-docs. **Scoping refinement (folded into the s07 row above):** `discover()`
structurally produces only project/arc/slice PlanNodes, and `validate` does **not** require `source`
(confirmed in `odm-core/src/check.rs` — only `author`/`version` are type-restricted on work nodes), so
s07's `source`/stub outcome is **arc/slice-scoped** and the 14 design/research nodes + ~211 loose docs
stay source-less **without failing `check`** — their `source` + coverage-into-`check` is **s08**. The
plan's earlier "every node source-bearing" / "populate author/version" framing corrected accordingly
(author/version N/A on the frontmatter-less plan corpus). No scope *loss* — a precision fix at draw so
CC runs a live mutation against an accurate target. s07 is the class-(b) slice where **MF-2/MF-3/MF-5
reproduce at arc scale** (§B) — they flip `planned → done` only at s07 close, on the committed store.

### v2.5 — 2026-07-28 — s06 CDC-verified PASS; doc-only comment finding carried onto s07

**CDC verification: PASS** (`slice06-live-run-capability/cdc-verification.md`; 10/10 rows). Reproduced
by close code-read on the live machine: `reconcile_source()` is the one gated, project- **and**
retired-excluding policy both `repair()` and `self_host`'s transition route through (no second copy to
drift); the gate runs even under `--dry-run`; `self_host_inner` runs `repair()` then `self_host()`,
default-on; no `unsafe`; live store not checked out. Cargo-run behaviors (drift/dry-run/idempotence,
clippy, coverage) attested → CI. **One CDC finding, doc-only (fails no row):** `self_host_inner`'s doc
comment calls the repair→import order a load-bearing correctness invariant ("reordering would
reintroduce exactly the gap s06 closed"), but CC's own closing-report F-3 disproves it — the Preferred
unification makes `self_host()` alone reconcile a coordinate-matched node identically, so the order is
for composability/report-attribution, not per-node correctness (which rests on `reconcile_source`,
exercised both ways). Carried onto s07 as a one-line comment correction at its opening, not a reopen
of s06. s06 flips to **CDC-verified PASS**; MF-2/MF-3/MF-5 stay **planned** (their live-corpus outcome
is s07's, per §B).

### v2.4 — 2026-07-28 — s06 (live-run capability) closed; both CDC findings resolved

**Slice 06 is closed** (`slice06-live-run-capability/closing-report.md`; 10/10 ledger rows done, 0
deferred, 0 no-op; attested-by-CC, CDC reproduction pending). Delivered exactly what v2.1–v2.3 scoped:
a single `reconcile_source()` is now the **one** gated, project-excluding `source`-population policy,
backing both `repair()` and `self_host`'s coordinate→source transition (closing the v2.1 finding at
its root, not just at the one call site CDC named); `odm migrate`'s self-host path now runs `repair()`
**then** `self_host()` by default, no new flag, folding the reconcile count into the existing report
shape (closing the v2.2 finding — `repair()` now has a real caller). **Self-identified extension
beyond the literal CDC finding:** the same unification pass found `self_host`'s transition carried no
guard against a **retired** node either (a tombstone sharing a coordinate with a live plan directory)
— `repair()` had this by accident of its own pre-check; `reconcile_source` now excludes retired nodes
for both callers, closing the same *class* of gap the CDC finding named, not just its one instance.
**What implementing it revealed:** (1) given the "Preferred" unification choice, `self_host()` alone
(no `repair()` call) now reconciles a coordinate-matched node identically to `repair()`-then-
`self_host()` — the specified order is implemented exactly as directed and is the right shape for API
composability (`repair()` needed a real caller), but it is not, in this implementation, load-bearing
for *per-node correctness* the way the plan's framing assumed; disclosed rather than silently
smoothed over. (2) `context.json` re-pointing, mentioned in `slice-doc.md`'s prose but absent from
`cc-prompt.md`'s Task list and structurally a live-store operator statement rather than a migration
artifact, stayed with **s07** as the slice-breakdown table already specified — followed the operative
`cc-prompt.md` over the summary prose, flagged the inconsistency rather than silently picking one.
MF-2/MF-3/MF-5 stay **planned**, now pointing at `slice04`/`slice05`/`slice06`'s closing reports as
fixture-verified baseline evidence — the live-corpus outcome each asserts is **s07**'s job.

### v2.3 — 2026-07-28 — s06 entry point corrected: extend `odm migrate`, not a new command

Drawing the s06 open set (CDC), the command inventory (`arc-llm-command-surface/`) settled the CLI
question v2.2 left open: `self-host` was **removed** as a spelling (C-4) and **folded into `migrate`**
(C-5) — "one verb." So s06 does **not** add a command; it **extends `odm migrate`'s self-host path**
(`self_host_inner`, `odm-cli/src/migrate.rs`) to run `repair()` before `self_host()`, both already
`Mode`/`--dry-run`-aware. Confirmed against the code: `odm migrate` dispatches to `self_host()` only;
`repair()`'s sole callers are tests. slice-doc / ledger / cc-prompt drawn to match. No scope change —
a naming/shape correction folded in at draw so CC doesn't build against a stale "add a command" framing.

### v2.2 — 2026-07-28 — s06 split into live-run capability + live run (s07); repair had no CLI entry

Drawing s06 surfaced that the live run has **no way to be invoked**: `selfhost::repair` (built s04,
generalized s05) is a library function with **no CLI command** — the only wired migration entry is
`odm migrate <plan>` (which runs `self_host`, not `repair`). So s06 becomes the **live-run
capability**: (a) the v2.1 two-path fix (route `self_host`'s `to_populate` transition through the same
gated, project-excluding logic as `repair()`), and (b) **wire the full repair/reconcile flow into an
invocable, `--dry-run`-able CLI command** — both fixture-verified, no live mutation. The actual live
mutation is a new **s07 — live repair run**, deliberately its own slice so the capability is
CDC-verified before anything touches the real store (the "build then run" gate, applied to the arc's
highest-stakes action). Downstream renumbered: old s07/s08/s09 → **s08/s09/s10**. Not needless
deferral — the op literally cannot be run until it is wired.

### v2.1 — 2026-07-28 — s05 CDC-verified PASS; two-path source-backfill finding onto s06

**CDC verification: PASS** (`slice05-source-identity/cdc-verification.md`; 12/12 rows). Reproduced by
CDC: idempotence keys on `source.paths` (primary) with the `by_coordinate` one-time transition; the
named-arc handle is name-derived (FNV-1a slug hash, deterministic collision-probe, recomputable);
coverage matches `source.paths` exact; `number` keys nothing for correctness; the live store is
untouched (0 `source:`, 60 nodes, all `v1.0`). `repair()`'s unification of stub-repair + faithful
backfill under **one gated path** is endorsed, and CC's F-5 test drives the drift branch with two
independent bodies (better than s03/s04's primitive-only). **One CDC finding, carried onto s06:** a
*second* `source`-backfill path exists — `self_host`'s coordinate→source transition (`to_populate`)
adds `source` **ungated** and **without the project exclusion** `repair()` enforces. Invisible in
s05's fixtures, but at the live run it would stamp a 1:1 `source` onto the **synthesis project node**
(the exact case F-6 prevents, via a path F-6 doesn't cover) and could stamp `source` over a drifted
body without surfacing it. A spec gap the CDC slice-doc didn't anticipate (it fails no s05 row).
**s06 must unify/guard the two paths before the live mutation** (s06 row updated). MF-2/3/5 stay
planned.

### v2.0 — 2026-07-28 — s05 (source-based identity) closed; number retired as a correctness key

**Slice 05 is closed** (`slice05-source-identity/closing-report.md`; 12/12 ledger rows done, 0
deferred, 0 no-op; attested-by-CC, CDC reproduction pending). Delivered exactly what v1.9 scoped:
`self_host` idempotence now keys on **`source.paths`**, with a one-time coordinate→source transition
that backfills `source` onto a pre-slice05 legacy node in place; the s04 v1.8 named-arc re-run-
duplicate hazard is now structurally impossible (source-matching alone would prevent it, and the
new name-derived, collision-handled `named_arc_number(slug, taken)` additionally removes the number
*shift* that caused it); `repair` is generalized past the stub filter — a faithful non-stub node
lacking `source` gets one as a body-unchanged no-op, a drifted one surfaces `BodyHashMismatch` rather
than being silently backfilled, and the project node stays excluded (its body is a synthesis, ODD-0025
§2.3); `coverage.rs`'s doc-coverage matcher gained an exact `source.paths` check as primary, closing
CC's s04-disclosed named-arc coverage gap without any model change. **What implementing it revealed:**
(1) the stub-repair and non-stub-backfill cases collapse into one code path under one hash-gate call,
not two branches with separate error handling — computing the candidate body first (source text for a
stub, the existing body otherwise) and always gating it against the source is simpler *and* stronger
than a two-policy design; (2) the F-7 coverage-matching piece needed no split-escape to s07 as
v1.9 reserved — it turned out cheap once F-1 was in place, since every node already carried
`source.paths` from s03 onward, so the fix was in the matcher, not the model; (3) a genuinely
vanishing edge case is named for the record: `named_arc_number`'s collision-bump is deterministic
given a fixed slug set in a fixed processing order, but a *new* slug that collides with an
already-claimed slot could in principle still shift a later arc's number depending on insertion
order — not a live risk, and source-matching absorbs it regardless, but worth a one-line explanation
if a live run ever shows two cosmetic numbers disagreeing across runs. MF-2/MF-3/MF-5 stay
**planned**, now pointing at both `slice04-scope-repair-capability/closing-report.md` and
`slice05-source-identity/closing-report.md` as fixture-verified baseline evidence — the live-corpus
outcome each asserts is s06's job.

### v1.9 — 2026-07-28 — source-based identity inserted as s05 (retire number-as-a-correctness-key); live run → s06

Operator decision (2026-07-28): **retire `number` as a correctness key now, before any live work** —
the number kept causing problems precisely because a *derived* value was doing an *identity* job. A
new **s05 — source-based identity** is inserted ahead of the live run: `self_host` idempotence +
coverage matching key on **`source.paths`** (the stable identity s03 gave every node), not
`(type, number)`; the already-faithful non-stub nodes get a one-time **`source` backfill** (body
unchanged, hash-gate-confirmed) so *every* node carries one; and the named-arc handle becomes
**name-derived + stable**. Once `source` is the key, `number` is a pure display label nothing keys on,
so the position-based handle fragility (v1.8 finding) **dissolves permanently** rather than being
patched. Resolves the v1.8 pre-mint requirement at its root and promotes design-notes F12 from
"future refinement" to **done-now**. Downstream renumbered: old s05/s06/s07/s08 → **s06/s07/s08/s09**.
*(Should have been done when F12 first surfaced; deferring it is what let the number keep biting —
recorded so the lesson sticks.)*

### v1.8 — 2026-07-28 — s04 CDC-verified PASS; named-arc handle scheme flagged onto s05

**CDC verification: PASS** (`slice04-scope-repair-capability/cdc-verification.md`; 12/12 rows).
Reproduced by CDC on the live machine: the cap is **deleted** (grep clean), `schema.rs CURRENT = v1.1`
with forward-compat, the live store is untouched (0 nodes carry `source:`, all 60 still `v1.0`), and
`repair()` preserves id/edges/status/number by **cloning the frontmatter and mutating only the delta**
(a stronger guarantee than the slice-doc asked for). The s03 schema-bump carry-forward is **closed**
(executed here; blast radius smaller than feared — only 7 test literals, found by grep). **One
load-bearing finding, carried onto s05:** the named-arc `number` handle is **position-based**
(`named_arc_number(index) = 1900 + index·100`, from the CDC slice-doc spec) — collision-free and
correct for a one-shot, but (1) unstable under *adding* a named arc (a re-run then mints a **duplicate**
node, since idempotence keys on `(type, number)`), and (2) not recomputable from a name (so coverage
can't resolve named arcs — CC's disclosed gap). Both dissolve once the number stops being a correctness
key: **s05 must, before minting named-arc nodes on the live store, either make the handle name-derived
or move idempotence onto `source.paths` (F12 — now load-bearing, not optional).** CC's disposition of
the coverage gap (defer to s06's source-based matching, ODD-0025 §5) is concurred. MF-2/3/5 stay
planned — live outcomes s05 produces.

### v1.7 — 2026-07-28 — s04 (scope + repair capability) closed; ready for the live run

**Slice 04 is closed** (`slice04-scope-repair-capability/closing-report.md`; 12/12 ledger rows
done, 0 deferred, 0 no-op). Delivered exactly what v1.6 restructured this slice to: the
`MAX_MVP_ARC`/`arc_in_scope` cap is deleted (not raised) — `self_host` now imports every arc/slice
directory it finds, numbered or named; named arcs get a deterministic, collision-free `number`
handle (`NAMED_ARC_BASE = arc_number(8) + NAMED_ARC_STEP`, a `const`-derived value, not an
independent magic number); a new `selfhost::repair` op performs ODD-0025 §2.8's update-in-place
repair (verbatim body + `source` written into the *same* node, `id`/`edges`/`status`/`number` all
preserved, via `Store::persist` overwrite — no `delete` anywhere); and the ODD-0020 schema-minor
bump s03 deferred is executed (`SchemaVersion::CURRENT` now `v1.1`, with a new test proving an
existing `v1.0` node stays valid). Entirely fixture-verified; the live corpus's byte-hash is
unchanged throughout. **What implementing it revealed:** (1) cloning the existing `Frontmatter`
and mutating only the touched fields is a stronger, more reusable "preserve everything else"
pattern than field-by-field reconstruction — worth defaulting to for any future node-update
operation (e.g. s06's supporting-doc work, if it ever needs to update rather than only create);
(2) `coverage.rs`'s heuristic arc-matching has a concrete, now-nameable gap — it still can't
resolve a **named** arc's assigned number (that depends on the full sorted list of named-arc
directories, not a single directory name), so a named arc's docs will keep reporting
"uncovered" even after `self_host` mints it a real node; disclosed in code and flagged as
explicit s06+ scope (wire the matcher against `source.paths` once that's the exact-match
mechanism ODD-0025 §5 anticipates), not silently left to be rediscovered as a `check` false
positive. MF-2/MF-3/MF-5 stay **planned** — the capability exists and is fixture-proven in full;
the live-corpus outcome each asserts is s05's job.

### v1.6 — 2026-07-27 — s04 restructured: cap killed; capability/live-run split; numbers = handles

Three operator decisions (in-session, superseding a past-CDC framing):

1. **The `MAX_MVP_ARC` / `arc_in_scope` cap is removed outright.** It was a hardcoded scope-lock
   that bought nothing, was the root cause of the 6-missing-arcs hole (design-notes: "scoped to the
   MVP arcs and never widened"), and directly contradicts the "no file left behind" property — a cap
   and that property cannot both be true. `self_host` now imports every arc/slice dir. (`arc_in_scope`
   is shared with `coverage.rs:512`, so removal also makes the detector stop treating those arcs as
   out-of-scope — more honest.)
2. **`number` is a non-structural human handle**, not identity (the ULID) and not order (the
   dependency DAG; 0013 §2.3). So named arcs (`arc-store-home`, `arc-release-hardening`,
   `arc-llm-command-surface`, `arc-migration-fidelity`) simply get an assigned handle by a
   deterministic rule — **no deferred "canonical numbering" decision is required** (an earlier CDC
   over-escalated this). *(Bonus, noted not scoped: s03's `source` field is a more principled
   idempotence key than `(type, number)` — a future refinement.)*
3. **s04 splits into capability-then-live-run.** s04 builds + fixture-proves the cap-removal +
   update-in-place repair + schema bump with **no** live mutation; a new **s05 — live repair run**
   fires it on the real corpus. Downstream renumbered: old s05/s06/s07 → **s06/s07/s08**. This also
   corrects the old s04-row's stale "delete bodyless nodes then re-migrate / a real `delete`
   capability" — ODD-0025 §2.8's update-in-place (via `persist` overwrite, preserving id/edges/status)
   needs no delete. Decisions logged in `design-notes.md` §3 (F11/F12).

### v1.5 — 2026-07-27 — s03 CDC-verified PASS; two findings ratified upward

**CDC verification: PASS** (`slice03-fidelity-core/cdc-verification.md`; 12/12 rows). Reproduced by
CDC on the live machine: the stub synthesis is gone, the live store is untouched (0 nodes carry
`source:`, still 60 nodes), the three fields are typed, `fidelity.rs` uses `trim+lf` with no stored
hash, and `de_opt_version` (the `1.0`→`"1"` fix) is sound and tested; cargo rows attested→CI. **The
F-3 correction was CDC's error, correctly caught** — the s03 ledger F-3 bundled `source` with
`author`/`version` as document-only, contradicting F-7; ODD-0025 §2.2 (`source` on *every migrated
node*) is right, and CC corrected toward it. Ratified CC's v1.4 bubble-up and carried up two findings:
**(a) ODD version-field hygiene** — bumped 0013→2.4 / 0020→v1.2 frontmatter + entry (corrected on disk
2026-07-27); **(b) branch deviation** — resolved by the fast-forward to `release/1.0.x`. Hardened the
schema-minor bump onto s04's row (now s04-capability, executed there).

### v1.4 — 2026-07-27 — s03 (fidelity core) closed; the faithful-import capability is fixture-proven

**Slice 03 is closed** (`slice03-fidelity-core/closing-report.md`; 12/12 ledger rows done, 1 with a
corrected criterion, 0 deferred, 0 no-op). Delivered ODD-0025's model as code: `odm-core` gained
typed `source`/`author`/`version` fields (round-trip + per-type validity); both importers now import
bodies **verbatim** — the stub-synthesis root cause is gone — behind a shared hard body-hash gate
(`odm-migrate::fidelity`), and populate the `source` record. ODD-0013 (v2.4) and ODD-0020 (v1.2)
carry the ODD-0025 §4 amendments. Fixture-verified; the live corpus untouched. Revealed: (1) the
F-3/F-7 internal inconsistency (see v1.5); (2) the `version: 1.0`→`"1"` YAML-float hazard, fixed via
`legacy::de_opt_version`; (3) the schema-minor bump is bigger than a rider — now owned by s04.
MF-2/MF-3 stay planned (live-corpus outcomes).

### v1.3 — 2026-07-27 — s02 (model) closed; ODD-0025 Accepted; provenance→source, author/version typed, F10 mint-all

**Slice 02 is closed** (14/14 rows; authored CDC-seat, operator-gated). Deliverable **ODD-0025**
(Accepted). Authoring revealed: (1) `provenance` was the wrong name (0013 reserves it for derived
lineage) → renamed **`source`** (§2.0 origin/source/provenance split); (2) `author`/`version` become
typed fields (operator corrected a proposed drop); (3) F10 resolved **mint-all**; (4) the
`note`-vs-`artifact` sub-fork → a new `artifact` type. MF-2/3/4/6/7 point at ODD-0025 as baseline.

### v1.2 — 2026-07-27 — s01 CDC-verified; report self-coverage escalated to F10

**CDC verification: PASS** (`slice01-coverage-discovery/cdc-verification.md`; 9 rows). F-5 (44 stubs),
F-6 (60 provenance-absent), the 60-node composition, and zero-`unsafe` independently re-derived by
CDC. Escalated **F10**: the arc's own generated reports would fail the enforced doc-coverage gate
forever unless dispositioned (resolved mint-all in v1.3).

### v1.1 — 2026-07-27 — s01 (coverage-discovery) closed; the arc's work-list is now exact

**Slice 01 is closed** (9/9 rows). Shipped the read-only `coverage` module + `odm migrate --coverage`;
live inventory **326 source docs, 266 uncovered, 6/12 arc dirs + 39/44 slice dirs represented, 44
stub bodies, 60/60 provenance-absent** — committed as the arc's authoritative work-list. Revealed the
chunk-artifact classification question, the three meanings of "research", and the self-referential
report finding — all fed s02.

### v1.0 — 2026-07-27 — arc shaped

Shaped from `reconciliation-audit-2026-07-27.md` + the migration/provenance design thread. Seven
slices, load-bearing order. F-forks resolved except F4 (a s02 detail). Unblocked by ODD-0024.
Release-blocking; subsumes L-8b.
