# Slice 07 (Arc 04): Early-cutoff invalidation

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. Modest slice — early
> cutoff for a *batch CLI* is the persisted-artifact skip + the signal (0014 §2.4),
> not lazy/persistent graph caching.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| E-1 | `reconcile`'s `Delta` distinguishes **meta-changed** from **body-only**: a changed record whose new `meta_hash` == the prior record's `meta_hash` is body-only (not meta-changed); the `Delta` exposes the meta-changed id set | `cargo test -p odm-index delta_distinguishes_meta_changed_from_body_only` → ok | serious | 0014 §2.4/§2.5 | done | attested: test → ok. `Delta` gains `meta_changed: Vec<Id>` (⊆ `changed`); a new `note_change(delta, prior, updated)` helper at **both** CHANGED arms pushes to `changed` always and to `meta_changed` only when `meta_hash` differs. Test: a longer-body edit (same fm) → `changed`, not `meta_changed`; a renamed node (title is a meta field) → both. | `reconcile` already holds prior + new record at the CHANGED arm — just compare `meta_hash`. The signal A5/A7 will also read. |
| E-2 | `odm rollup` skips regenerating `ROLLUP.md` when the corpus is **semantically unchanged** since the last generation (no meta-change, no new, no deleted) — the file is left untouched (byte-identical, mtime preserved) | `cargo test -p odm-cli rollup_skips_on_body_only_change` → ok | serious | arc-plan slice07 / 0014 §2.4 | done | attested: test → ok. `Snapshot::meta_fingerprint()` = SHA-256 over each record's `(id, meta_hash)` in id order; `odm rollup` stamps it in the generated header (`fingerprint=<hex>`) and skips the rewrite if the existing file carries the same fingerprint. Test: a body-only edit → second `rollup` reports "unchanged … skipped" and `ROLLUP.md` is **byte-identical**. | Recommended: a combined `meta_hash`-fingerprint stamped in the generated header; skip if the current corpus fingerprint matches. |
| E-3 | `odm rollup` **regenerates** `ROLLUP.md` when any `meta_hash` changed, or a node was added/deleted (a meaning-change is never missed) | `cargo test -p odm-cli rollup_regenerates_on_meta_change` → ok | serious | arc-plan slice07 | done | attested: test → ok. Three triggers, each asserts "wrote" + changed bytes: (a) a gate change (`set-gate` → meta_hash moves), (b) a new node, (c) a deleted node (both move the record set ⇒ the fingerprint). Edge/origin/decomposed changes ride the **same** `meta_hash` mechanism (a `meta_hash` move always re-fingerprints). | Covers: gate/evidence change, edge change, origin/decomposed change, new/deleted node. The early cutoff must never skip a real change. |
| E-4 | A body-only edit still **refreshes the index record** (`content_hash` + stat updated via the warm path) — early-cutoff skips the *derived* recompute, not the record refresh | `cargo test -p odm-index body_only_edit_refreshes_record` → ok | serious | 0014 §2.4 | done | attested: test → ok. A body-only edit → the warm path rebuilds the record: `content_hash` **differs** from prior, `meta_hash` **unchanged**; the id is in `delta.changed` but **not** `delta.meta_changed`. The record refreshes; only the derived recompute is skipped (E-2). | The §2.4 distinction: record updates; downstream does not. |
| E-5 | The in-memory graph readers (`next`/`blocked`/`path`/`check`/`orient`) are **unchanged** — eager recompute per invocation (acceptable for a batch CLI, 0014 §2.4); no behavior change | `cargo test -p odm-cli` (the slice05/06 reader tests stay green) → ok | correctness | 0014 §2.4 | done | attested: no reader source touched this slice (`git diff` limited to `warm.rs`, `snapshot.rs`, `rollup.rs` + tests/docs); the slice05/06 reader suites (`index_backed.rs` derived-order, `orient.rs`, `cli.rs check`) stay green. No persistent in-memory derived caching added. | The design boundary: persistent in-memory derived caching is out (deferred unless slice08 shows need). |
| E-6 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for odm-index + odm-cli | `cargo clippy --all-targets -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-index/src crates/odm-cli/src` AND `cargo llvm-cov --summary-only --ignore-filename-regex` per crate → **line** ≥ 90% | serious | CLAUDE.md | done | attested: `cargo clippy --all-targets --all-features -- -D warnings` → exit 0; `grep` for `unsafe` → none; `cargo llvm-cov --summary-only` per crate (others ignored) → **odm-index line 95.18%**, **odm-cli line 93.68%**. Full `cargo test --workspace` green. | |

## What Worked

- **The two-fingerprint split paid off exactly as designed.** `meta_hash` has carried
  the "did the meaning change?" signal since slice02; this slice just *read* it. The
  delta distinction is a one-line `meta_hash` compare at the CHANGED arm (where `reconcile`
  already holds both records), and the rollup skip is a fingerprint comparison — no new
  traversal, no parse, no recompute.
- **One signal, two surfaces.** `Delta::meta_changed` (the in-memory subset) and
  `Snapshot::meta_fingerprint` (the persisted-artifact stamp) are both derived from the
  same `meta_hash`. The fingerprint over `(id, meta_hash)` is robust to history (it
  compares actual semantic state, not reconcile bookkeeping) — a new/deleted node moves
  the id set, a meaning-change moves a hash, a body-only edit moves neither.
- **Honoured the 0014 §2.4 ceiling.** Early cutoff for a batch CLI is the persisted-
  artifact skip + the signal — **not** Salsa-style lazy graph caching. The in-memory
  readers stayed eager and untouched; no caching was built.
- **Regenerate paths tested as hard as the skip.** Gate change, new node, and deleted
  node each assert a real regenerate; the skip is never allowed to swallow a meaning-change.

## Closure

All 6 rows `done` at **attested** (CC); cargo/coverage rows reproduce via CI / a local
1.85+ toolchain (the sandbox runs 1.95.0). Coverage: odm-index **95.18%** line (hits the
95% target), odm-cli **93.68%** line. No `unsafe`; clippy `-D warnings` clean; full
workspace `cargo test` green.

**On close:** a body-only edit refreshes the index record but leaves `ROLLUP.md`
untouched; any meaning-change / new / deleted node regenerates it. **Only slice08 (the
100k benchmark) remains** in Arc 04 — and it measures the finished system, which can then
settle whether deeper (in-memory) caching is ever warranted. Bubbled up to `arc-plan.md`
A-7 per LEDGER-DISCIPLINE v2.0 §A.
