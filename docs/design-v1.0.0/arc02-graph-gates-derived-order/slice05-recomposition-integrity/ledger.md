# Slice 05 (Arc 02): Decomposition/recomposition integrity

> Per LEDGER_DISCIPLINE. Cargo rows reproduced in CI / local 1.85+. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| H-1 | Reverse-`part_of` enumerates a parent's complete child set | `cargo test -p odm-core recompose_children` → ok | serious | 0013 §4.5 | done | `f3bd4ba`: `recompose_children` → 1 passed. `Recomposition::children(parent)` returns the full child set, sorted by id; a leaf has none. | |
| H-2 | Every non-root node resolves to exactly one parent (total, unambiguous) | `cargo test -p odm-core single_parent_total` → ok | serious | 0013 §4.5 | done | `f3bd4ba`: `single_parent_total` → 1 passed. roots ⊎ resolved-children partitions the corpus; single-parent is structural (`part_of` is `Option`). | |
| H-3 | Orphan detection: a non-root node with no resolvable parent is flagged | `cargo test -p odm-core detect_orphan` → ok | serious | 0013 §4.5 | done | `f3bd4ba`: `detect_orphan` → 1 passed. `Issue::Orphan` for a work node (arc/slice) with no part_of or an unresolved one; project (root) + standalone note exempt. | Unresolved part_of overlaps `check` v1 `DanglingPartOf` (different lens). |
| H-4 | No-stub: a `project`/`arc` advanced into a working/complete gate with zero children is flagged | `cargo test -p odm-core detect_undeveloped_stub` → ok | correctness | 0013 §4.5 | done | `f3bd4ba`: `detect_undeveloped_stub` → 1 passed. `UndevelopedStub{gate}` when parent-capable + reached a gate past index-0 (planning) + zero children; planned-only or has-children → clean. | |
| H-5 | `decomposed: complete` assertion recorded on a parent | `cargo test -p odm-core decomposed_assertion` → ok | correctness | 0013 §4.5 | done | `f3bd4ba`: `decomposed_assertion` → 1 passed. `affirm_decomposed(children, on)` stores a sorted/deduped `Decomposition{on, children}`; `decomposed()` reads it. | Modeled `{on, children}`, not bare scalar (drift needs the affirmed set) — flagged. |
| H-6 | Guard: children added/removed after `decomposed: complete` flags for re-affirmation | `cargo test -p odm-core decomposed_drift_guard` → ok | serious | 0013 §4.5 | done | `f3bd4ba`: `decomposed_drift_guard` → 1 passed. `DecompositionDrift{added, removed}` = set-difference of current children vs the affirmed set (both add and remove cases). | |
| H-7 | Guard: a parent advanced toward done WITHOUT `decomposed: complete` is flagged | `cargo test -p odm-core advance_without_decomposition` → ok | correctness | 0013 §4.5 | done | `f3bd4ba`: `advance_without_decomposition` → 1 passed. `AdvancedWithoutDecomposition` when parent-capable + reached terminal gate + no assertion; affirming clears it. | "Toward done" read as reached-terminal — flagged threshold choice. |
| H-8 | Semantic missing-scope detection is NOT attempted (documented non-goal; no false "missing scope" claims) | `cargo test -p odm-core no_semantic_scope_guessing` (asserts the API only reports structural facts) → ok | correctness | 0013 §4.5 | done | `f3bd4ba`: `no_semantic_scope_guessing` → 1 passed. A done arc with a single affirmed slice child yields zero findings; `Issue` has no missing/excess-scope variant by design (module doc states the non-goal). | |
| H-9 | Clippy clean (`-D warnings`); no `unsafe` | `cargo clippy -p odm-core --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-core/src` | serious | CLAUDE.md | done | `f3bd4ba`: clippy → exit 0 (after `.filter().next_back()`→`.rfind`, needless-borrow fixes); unsafe grep → no match; fmt --check clean. | |
| H-10 | Coverage ≥ 90% (target 95%) | `cargo llvm-cov -p odm-core --summary-only --ignore-filename-regex '(odm-graph|odm-store|odm-cli)/'` → **line** ≥ 90% (target 95%) | correctness | CLAUDE.md | done | `f3bd4ba`: TOTAL **line 97.85%** / region 97.25% (recompose.rs line 98.88%, region 99.52%); from a clean `target/llvm-cov-target`. | |

## What Worked

- **Pure function over the corpus, mirroring `check` v1.** `recompose` takes
  `&[Frontmatter]` (+ `GateSets`) and returns deterministic findings — no I/O,
  no CLI knowledge. `check` v2 (slice06) aggregates these predicates alongside
  the v1 structural checks without rewriting either.
- **Forest vs. findings split.** `Recomposition` answers the *structural*
  questions (children, parent, roots — H-1/H-2); `integrity` layers the
  *judgements* (H-3/4/6/7) on top. Recomposition stays reusable for `show`.
- **Single-parent is free.** Because `part_of` is `Option<Id>`, "exactly one
  parent" is unrepresentable-otherwise; H-2 verifies totality (the partition)
  rather than re-checking a multiplicity the type already forbids.
- **The non-goal is enforced by the type, not just prose.** `Issue` has no
  missing/excess-scope variant, so the engine *cannot* emit a guessed-scope
  claim; H-8 pins that a structurally-sound-but-sparse parent stays clean.
- **Gate index drives "advanced"/"done".** A small helper reads the type's gate
  sequence: past index-0 = "working" (stub trigger), reached terminal = "done"
  (advance-without-decomposition trigger). Types without a gate-set are exempt
  from those two checks (advancement unjudgeable) but still orphan/drift-checked.

## Closure

Closed at `f3bd4ba` on `2026-06-24`. CDC: pending (cargo rows reproduced by CDC
in CI or a local 1.85+ toolchain). All `done` states are *proposed done* pending
that independent verification. Total rows: 10. Done: 10. Deferred: 0. No-op: 0.

**Flagged for CDC.** (1) `decomposed` modeled as `{on, children}`, not the bare
`decomposed: complete` scalar from §2.3 — the scalar carries no record of the
affirmed set, so it cannot detect *removal*; the structured form is required for
H-6. (2) "Advanced toward done" (H-7) read as **reached the terminal gate**; an
alternative reading is "any progress past planning" (which would overlap H-4) —
threshold choice documented. (3) Orphan (H-3) for an *unresolved* `part_of`
overlaps `check` v1's `DanglingPartOf` (a different lens — recomposition totality
vs. link-integrity); slice06 can dedupe when aggregating. (4) The arc01
unknown-key proptest skip-list was extended to include the now-typed
`status`/`retired`/`decomposed` (a latent gap since slice04 wired `status`); a
scalar on those keys would now fail to deserialize rather than land in `extra`.
