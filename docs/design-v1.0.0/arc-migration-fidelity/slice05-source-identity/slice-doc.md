# Slice 05 (Migration Fidelity) — Source-based identity (plan-of-record)

> Refs: `../design-notes.md` §3 (F11/F12 — F12 now DECIDED-done-here); `../arc-plan.md` v1.9 (the
> s05 row + the decision); ODD-0025 §2.0/§2.2 (`source`), §2.8 (update-in-place), §5 (coverage →
> exact set-difference on `source`). `depends_on:` s04 (the repair op + `source` field + the cap
> removal this builds on).
>
> **This is a *capability* slice — fixture-verified, NO live mutation.** The live run (backfill +
> repair + import, for real) is **s06**.
>
> **Why now (v1.9):** `number` is a *derived* value that has been doing an *identity* job — the root
> of every "number problem" this arc has hit (the s04 position-based-handle re-run-duplicate hazard
> being the latest). This slice retires `number` as a **correctness key**: `source.paths` (the stable
> identity s03 gave every node) becomes the key. Once it is, `number` is a pure display label nothing
> keys on, and the fragility dissolves permanently rather than being patched.

## Goal

Make **`source.paths` the identity/idempotence key** across `self_host`, the repair path, and
coverage matching — retiring `(type, number)` as a correctness key — and give **every** node a
`source` record so the key is universally present. **Done when** (a) `self_host` idempotence keys on
`source` (with a one-time coordinate transition for pre-`source` legacy nodes), (b) every faithful
non-stub node is `source`-backfilled (body unchanged, hash-gate-confirmed), (c) coverage matches
source-docs → nodes by `source.paths` (exact), (d) the named-arc handle is name-derived + stable,
all fixture-proven, with the live corpus untouched.

## Scope

**In:**

- **Source-keyed idempotence in `self_host`.** Replace the `(type, number)` idempotence key with
  **`source.paths`**. Transition rule (the corpus is pre-`source` today): build `by_source` from
  nodes that *have* `source`; for a node that lacks it, fall back to matching by structural
  coordinate for its **one-time** `source` population; thereafter it matches by `source`. Net: a
  re-run finds an already-imported node by its stable source path — never by a shifting number — so
  the s04 named-arc re-run-duplicate hazard cannot occur.
- **`source` backfill (reconcile-to-source) for faithful non-stub nodes.** Generalize s04's `repair`
  so it also covers non-stub nodes that lack `source`: match the node → its source doc by coordinate,
  run the existing body through the **hard body-hash gate** against the source body (a faithful body
  passes as a no-op; a *non*-faithful one surfaces as a gate failure — a real fidelity finding, not
  swallowed), and add the `source` record. **Exclude the project node** — its body is a *synthesis*
  of `project-plan.md` §1 (body ≠ source; ODD-0025 defers synthesis to s08), so it is not
  source-backfilled here; flag it, don't force it through the 1:1 gate.
- **Source-based coverage matching** (ODD-0025 §5). `coverage.rs` matches a source doc → its node by
  **`source.paths`** (exact set-difference) rather than the coordinate/number heuristic — which also
  resolves named arcs (CC's s04-disclosed gap) for free. *(Split-escape: if s05 runs heavy, this
  piece can move to s07 where coverage is rebuilt for enforcement — flag CDC. The correctness core is
  the idempotence key + backfill.)*
- **Name-derived stable named-arc handle.** Replace `named_arc_number(index)` (position-based) with a
  **name-derived** handle (deterministic from the slug, into the ≥ 1900 band, collision-handled), so
  the cosmetic `number` is stable under adding arcs and `odm show <n>` stays meaningful. `number`
  remains display-only — nothing keys on it.
- **Tests (fixtures/temp stores only):** source-keyed idempotence (a fixture with `source`-bearing
  nodes; adding a named arc + re-running mints **no duplicate**); the coordinate→source transition (a
  pre-`source` node populated once, then matched by source); backfill (a faithful node gets `source`,
  body unchanged, gate passes; a drifted body fails the gate); the project-node synthesis exclusion;
  source-based coverage (a named arc resolves by `source`); name-derived handle (stable under adding
  arcs, recomputable from the slug).

**Out:** the **live run** (s06 — any `.worktrees/odm` mutation); minting the `artifact` supporting
docs + wiring doc-coverage into `check` (s07); the **synthesis** treatment of the project node (s08);
re-pointing the live `context.json` (s06).

## Verification

`cargo test -p odm-migrate` (source-keyed idempotence, transition, backfill, coverage-by-source) and
`cargo test -p odm-core` green; `cargo clippy --workspace --all-targets -- -D warnings`; no `unsafe`;
coverage ≥ 90% (line) on changed modules; the idempotence test demonstrates **no duplicate** on a
re-run after a named-arc set change (the exact s04 hazard, now impossible). **Live store untouched.**
Cargo rows `attested`→`reproduced`-on-CI.

## Exit

`ledger.md` closed; CDC-verified. `source` is the identity key everywhere it was `(type, number)`;
every faithful node backfills; the handle is stable; `number` keys nothing. So **s06** can run the
live migration on a corpus where identity is stable and re-runnable. On close, bubble up to
`../arc-plan.md` (the v1.8 pre-mint requirement is now resolved at root; F12 done).
