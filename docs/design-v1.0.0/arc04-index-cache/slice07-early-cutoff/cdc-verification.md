# CDC Verification — Arc 04 / Slice 07: Early-cutoff invalidation

> Independent verification of CC's closed ledger (impl + close on
> `arc04-slice07-early-cutoff`), per LEDGER-DISCIPLINE v2.0 (slice scale, §A). CDC
> reproduces structural rows here; cargo rows route to CI / local 1.85+.

## Environment constraint (disclosed)

No 1.85+ toolchain in the sandbox; cargo executions route to CI / CC's local 1.95.0 run
(held **attested pending CI**). CDC reproduces structural rows on the branch.

## Row dispositions

**Row count:** 6 opened, 6 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **E-1** — `Delta.meta_changed: Vec<Id>` (⊆ `changed`); `note_change(delta, prior,
  updated)` at both CHANGED arms pushes to `changed` always and `meta_changed` only on a
  `meta_hash` differ (`warm.rs:216`). Body-only edits land in `changed`, not
  `meta_changed`. ✔
- **E-2** — `Snapshot::meta_fingerprint()` = SHA-256 over each record's `(id, meta_hash)`
  in id order (`snapshot.rs:208`); `odm rollup` stamps it in the header
  (`fingerprint=<hex>`) and **skips the rewrite** when the existing file's stamp matches
  (`rollup.rs:124–133`). **Single reconcile** confirmed (`rollup.rs:96`): rollup
  reconciles once → fingerprints that snapshot → adapts the same records for the model;
  no double walk, no second `reconcile` via `index_frontmatters`. ✔
- **E-3** — regenerate forced on a `meta_hash` change / new / deleted (the fingerprint
  moves); tests cover a gate change, a new node, a deleted node. ✔
- **E-4** — a body-only edit refreshes the record (`content_hash` differs, `meta_hash`
  same; id in `changed` not `meta_changed`) — record refreshes, derived recompute
  skipped. ✔
- **E-5** — readers untouched: the diff is limited to `warm.rs`/`snapshot.rs`/`rollup.rs`
  (+ tests/docs); no persistent in-memory graph caching added (0014 §2.4 boundary held). ✔
- **E-6 (no `unsafe`)** — grep empty. ✔

**Attested by CC (local rustc 1.95.0), pending CI:** clippy `-D warnings` → exit 0;
`fmt` clean; line coverage **odm-index 95.18%** (a dip from 98% — still ≥ the 95% target)
**/ odm-cli 93.68%**; full workspace green. → **PENDING CI**.

## Rulings on CC's flagged items

1. **Fingerprint over `(id, meta_hash)`, not just `meta_hash`.** **Accepted, and a good
   call** — the node *set* is part of semantic state, so a delete+add that happens to
   preserve the `meta_hash` multiset still re-fingerprints (and regenerates). Correctly
   conservative.
2. **Skip compares stamped state, not reconcile events.** **Accepted** — robust by
   design: a hand-edited / pre-slice07 / stale `ROLLUP.md` carries no (or a different)
   stamp → safe regenerate. The cutoff can't be fooled by reconcile bookkeeping.
3. **`rollup` reconciles directly (one-line "duplication") vs. `index_frontmatters`.**
   **Accepted** — verified it is a *single* reconcile; rollup needs the `Snapshot`
   (records + `meta_hash`es for the fingerprint), which the frontmatter-only helper
   doesn't return, so inlining reconcile+adapter is the right seam. *Tiny optional
   refinement (not required):* the stamp-check could move ahead of `Rollup::assemble`/
   `render` to also skip the in-memory build on the skip path — but 0014 §2.4 accepts
   eager in-memory recompute, and the `--json` path needs the model anyway, so the
   current order (skip the *write*) matches the slice's stated scope.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — the meta-changed signal + the `ROLLUP.md` skip, exactly
  the modest 0014-§2.4 scope; the in-memory readers were deliberately left eager.
- **Silent-drop diff honest?** ✔ — 6/6; the eager-readers boundary + the coverage dip
  are disclosed, not buried.
- **Findings + arc-plan?** ✔ — A-7 attested (arc-plan v1.13). No CDC plan-keeping fix
  needed — no body-line inaccuracy, the status convention was applied correctly.

## Verdict

**Arc 04 / Slice 07 CDC-verified on structure; all flags ruled; cargo rows pending CI.**
The two-fingerprint split is cashed: a body-only edit refreshes the index record but
leaves `ROLLUP.md` byte-identical; any meaning-change / new / deleted node regenerates
it. The 0014 §2.4 ceiling (eager in-memory for a batch CLI) was honoured. **Only slice08
(the 100k benchmark) remains** — it measures the finished system, after which the
arc-close runs (recomposition/silent-drop across all 8 slices + the class-(b) compose
rows reproduced at arc scale) and **Arc 04 closes**.

CDC: planning thread, 2026-06-29. Iterations used: 1.
