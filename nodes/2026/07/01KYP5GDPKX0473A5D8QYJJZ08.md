---
id: 01KYP5GDPKX0473A5D8QYJJZ08
number: 550625700
type: artifact
schema: artifact/v1.1
name: 'Closing Report — Slice 05 (Arc 02): Decomposition/recomposition integrity'
created: 2026-06-24
updated: 2026-06-24
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc02-graph-gates-derived-order/slice05-recomposition-integrity/closing-report.md
  class: closing-report
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKW1C0RPYRW5GV37GK
---
# Closing Report — Slice 05 (Arc 02): Decomposition/recomposition integrity

> Per LEDGER_DISCIPLINE: a per-row walk, not a prose summary. CC's `done` is
> **proposed done**; CDC re-runs every Verify (cargo rows via CI or a local
> 1.85+ toolchain) before slice 06 opens.

- **Implementation commit:** `f3bd4ba`.
- **Branch:** `arc02-slice05-recomposition-integrity` (based on
  `arc02-slice04-derived-order-and-satisfaction`; not pushed; not merged to
  `main`).
- **Scope delivered:** decomposition/recomposition integrity, all in odm-core —
  `recompose::Recomposition` (reverse-`part_of` forest: children, parent,
  roots), `recompose::integrity` (orphan / undeveloped-stub / decomposition-drift
  / advanced-without-decomposition findings), and the typed
  `frontmatter::Decomposition` + `decomposed` field (the guarded assertion).
  slice06 (`check` v2) aggregates these predicates.
- **Result:** 10 rows, all `done`. 0 deferred, 0 no-op. 0 amendments raised.
- **Aggregate gates:** `cargo test -p odm-core` → all pass (8 new recompose
  tests; arc01/02 tests unregressed, full workspace green); clippy `-D warnings`
  → exit 0; no `unsafe`; coverage TOTAL line 97.85% / region 97.25% (recompose.rs
  line 98.88%, region 99.52%).

## Per-row walk

| ID | Status | Evidence (re-runnable at `f3bd4ba`) |
|----|--------|-------------------------------------|
| H-1 | done | `cargo test -p odm-core recompose_children` → 1 passed; `Recomposition::children` returns the complete child set, sorted. |
| H-2 | done | `cargo test -p odm-core single_parent_total` → 1 passed; roots ⊎ resolved-children partitions the corpus; single-parent is structural (`part_of: Option`). |
| H-3 | done | `cargo test -p odm-core detect_orphan` → 1 passed; `Issue::Orphan` for an arc/slice with no resolvable parent; project + standalone note exempt. |
| H-4 | done | `cargo test -p odm-core detect_undeveloped_stub` → 1 passed; parent-capable + past-planning + zero children → `UndevelopedStub{gate}`. |
| H-5 | done | `cargo test -p odm-core decomposed_assertion` → 1 passed; `affirm_decomposed` records a sorted/deduped `Decomposition{on, children}`. |
| H-6 | done | `cargo test -p odm-core decomposed_drift_guard` → 1 passed; `DecompositionDrift{added, removed}` = current-vs-affirmed set difference. |
| H-7 | done | `cargo test -p odm-core advance_without_decomposition` → 1 passed; reached-terminal-without-assertion flagged; affirming clears it. |
| H-8 | done | `cargo test -p odm-core no_semantic_scope_guessing` → 1 passed; sound-but-sparse parent yields zero findings; no missing-scope variant exists. |
| H-9 | done | clippy `-D warnings` → exit 0; `! grep '\bunsafe\b' crates/odm-core/src` → no match; `cargo fmt --check` clean. |
| H-10 | done | `cargo llvm-cov -p odm-core … --ignore-filename-regex '(odm-graph\|odm-store\|odm-cli)/'` → TOTAL line 97.85% (recompose.rs 98.88%), from a clean `target/llvm-cov-target`. |

## Decisions worth a CDC look (flagged, not silently changed)

1. **`decomposed` is a structured assertion, not the bare `complete` scalar.**
   §2.3 illustrates `decomposed: complete`. A bare scalar records *that* a
   decomposition was affirmed but not *what* against — so it cannot detect a
   later **removal** of a child (H-6). I modeled `Decomposition { on, children }`
   (on-disk `decomposed: { on, children: [...] }`), recording the affirmed child
   set, and de-dup/sort it on write for a stable round-trip. Flagging the
   deviation from the literal scalar; the spec text is illustrative, the
   drift-guard is normative, and the latter forces the richer shape.

2. **"Advanced toward done" (H-7) = reached the terminal gate.** §4.5 says "a
   parent advanced toward done without the assertion." I read "done" as the
   type's terminal gate. An alternative reading — *any* progress past planning —
   would make H-7 fire as early as H-4 (stub) and largely overlap it. Keeping
   H-7 at the terminal gate makes the two findings distinct: H-4 = "you advanced
   an *empty* container," H-7 = "you *finished* a container you never affirmed
   the decomposition of." Flagging the threshold choice.

3. **Orphan (H-3) overlaps `check` v1 `DanglingPartOf` for an unresolved
   parent.** An arc/slice whose `part_of` names an absent id is reported both as
   an orphan here (recomposition is not total) and as a dangling edge by `check`
   v1 (link-integrity). These are deliberately different lenses; I did not
   suppress either. slice06's `check` v2 aggregation can dedupe/group them.
   Flagging the intentional overlap.

4. **arc01 unknown-key proptest skip-list extended.** The I-5 proptest
   (`unknown_keys_preserved_proptest`) skips generated keys that collide with
   modeled fields, but its list had not been updated when slice04 wired `status`
   (nor for the pre-existing `retired`). Adding the typed `decomposed` field made
   the gap matter: a generated scalar on any of `status`/`retired`/`decomposed`
   would now fail to deserialize into the typed shape rather than land in
   `extra`. I extended the skip-list to include all three. In practice the
   generator (`[a-z_]{3,12}`) almost never emits those exact names, so the test
   was passing by luck; this closes the latent flake. Flagging the test edit to a
   closed arc01 slice.

## Uncertainties named

- **Drift cannot see a child whose `part_of` was repointed to a *missing*
  parent.** If a child is reparented to an id absent from the corpus, it leaves
  the old parent's child set (correctly flagged as `removed` drift) and is itself
  an orphan — but the "where did it go" is only inferable by cross-referencing
  the two findings, not stated directly. Acceptable for a structural pass.
- **No reserved/retired exemption yet.** A `reserved` placeholder parent or a
  `retired` node is still subject to stub/advance checks. Whether retirement or
  reservation should exempt a node from recomposition integrity is a policy
  question I left to the `check` v2 aggregation (slice06), where exemptions are
  better centralized.
- **`recompose.rs` residue (~1%).** The single uncovered line is the
  unresolved-`part_of` match arm's empty body region; TOTAL line 97.85% clears
  the bar comfortably.
- **Sandbox/CI parity.** All cargo evidence was produced on a local toolchain;
  CDC should reproduce on CI / a local 1.85+ run, from a clean
  `target/llvm-cov-target` (stale artifacts give bogus coverage).
