# Slice 16 (Migration Fidelity) — orient P-12 readiness: exclude retired nodes

> Refs: `../arc-plan.md` (P-12 gate) · **s15** (the collapse that retires `#1001`) ·
> `command-surface-uat-checklist.md` **F-22** (the retired-node leak, LLM-arc-scoped — s16 fixes only the
> P-12-critical orient slice of it) · `crates/odm-cli/src/orient.rs`. `depends_on:` s15.
>
> **Capability/fixture slice — no live `odm`-branch mutation.** The last step before the P-12 demonstration.

## Goal

Make `odm orient` demonstrate P-12 on the frozen corpus. **Done when** `orient` **excludes retired nodes**
from the surfaces it presents — the project-selection list and the READY set — so a fresh session on the
collapsed store auto-orients to the single active project (`#1000`) and renders its `# Vision`, instead of
being asked to choose between `#1000` and the **retired** `#1001`.

## Why

The arc-close freeze fired: the project-vision pair collapsed, the corpus is byte-faithful (0 drift), and
`check` is green (0 errors). But `odm orient` printed **"2 projects, none selected — choose one"**, listing
`#1000` (active) **and `#1001` (retired)**. `orient.rs:63` filters `node_type() == Project` with **no
`retired()` guard**, so the tombstone counts. `orient` already auto-orients when exactly one project exists
(`projects.len() == 1`), so a single predicate closes it: exclude retired → one active project → the vision
renders. This is the P-12-critical slice of punch item **F-22** (retired nodes leaking into `orient`/`next`);
the rest of F-22 (`next`, and any other surface) stays routed to the LLM-command-surface arc.

## Scope

**In (`release/1.0.x` code + fixtures):**

- **orient project selection** (F-1): `orient.rs`'s project enumeration (line ~63) filters
  `node_type() == Project` **and `retired().is_none()`**, so a retired project is never counted, listed, or
  offered for selection. A single remaining active project auto-orients (existing `len() == 1` path).
- **orient READY set** (F-2): the READY/next-actions list `orient` renders also excludes retired nodes (the
  same leak that surfaced the retired `#1605` UAT tombstone in READY — F-22). One consistent "orient never
  surfaces a retired node" rule.
- Fixtures for each; the model/index and every other command are untouched.

**Out:**

- **`next`** and any non-`orient` surface — the broader **F-22**, LLM-command-surface arc.
- Any change to retirement semantics or the collapse — `#1001` stays a retired tombstone (supersede-don't-
  delete); s16 only stops `orient` from *surfacing* it.
- The numeric-handle display retirement (punch B2-2) — LLM arc.

## Verification

Fixture, class-(a). After the change: a fixture store with one active + one retired project asserts `orient`
auto-orients to the active one and renders its vision (no "N projects, none selected"); a fixture with a
retired node in the ready frontier asserts it's absent from READY. Runtime attested→CI; CDC reproduces by
direct read + a re-run of `odm orient` on the frozen store (operator) showing the single project + its
vision.

## Exit

`ledger.md` closed; CDC-verified. `odm orient` on the frozen store demonstrates P-12: a fresh session reaches
situational awareness from `orient` alone. On close, bubble up to `../arc-plan.md`: s16 done; **the arc-close
runs** — MF-9 composition (fidelity achieved, reproduced) + the P-12 demonstration (now clean) + the
bubble-up → Migration Fidelity closes.
