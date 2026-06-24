# CDC Verification — Arc 02 / Slice 03: Gates, status & evidence

> Independent verification of CC's closed ledger (impl `a1dce22`; closed `91c8aba`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran;
> **attested** = CC's evidence, not reproduced here.

## Environment constraint (disclosed)

Cowork sandbox has no 1.85+ toolchain; cargo-gated rows route to CI / local.

## Row dispositions

**Row count:** 9 opened, 9 addressed. No silent drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
`Evidence` total order `Asserted < Attested < Reproduced < Reconciled` (derived
`Ord`) with `#[default] Asserted` (least-confident); `GateSet::from_toml_str` +
`terminal()`; `Status` is a `BTreeMap`-backed vector (not a scalar) with `set_gate`
validating against the type's `GateSet` (`UnknownGate` otherwise); no `unsafe`;
`toml` dep added to odm-core. → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
H-1…H-9 (9 tests incl. a YAML round-trip proving the §2.3 `status:` shape); clippy;
line 98.31% (gates.rs 100%). → **PENDING CI.**

## Rulings on CC's flagged items

1. **Typed `Status` is standalone; `Frontmatter` not modified.** **Accepted as a
   disclosed deferral — and now tracked (see below).** CC built and *proved*
   (wire-compatible YAML round-trip) the typed model, but did not replace
   `Frontmatter`'s preserved-unknown-key passthrough, because doing so would change
   arc01's closed `unknown_keys_preserved` test (2→1) and lacks a persistence path
   yet. Correct call. **My slice-doc overstated arc01** ("status serialization
   already in the arc01 schema") — arc01 only *preserved status as an unknown key*;
   the typed model is new here. CC is right; owning that.
2. **`set_gate` is an odm-core op, not a CLI command.** **Accepted.** The `odm
   set-gate` CLI command is tracked as an open item needing a home (below).
3. **`toml` dep added to odm-core (gate-set parsing lives where tested).**
   **Accepted.** `odm.toml` is now parsed by two crates — odm-store (paths, via
   confyg) and odm-core (`[gates.*]`) — reading **disjoint** sections. Fine.
4. **`BTreeMap` serialization order ≠ configured sequence order.** **Accepted** —
   deterministic, so round-trip-safe. *Note for arc03:* the rollup/`orient` should
   render a node's status in **gate-sequence order** (built→tested→…), not the
   serialized alphabetical order — a rendering concern, not storage.

## Uncertainty (CC-named) — accepted, routed to slice 04

`set_gate` records any in-set gate; it does **not** enforce sequence order or
monotonic evidence. Correct scoping. Slice 04 decides: (a) whether reaching gate N
requires gates < N, and (b) whether evidence may regress.

## Tracked deferrals (so they are not silent drops — 0001 E5)

1. **Wire typed `Status` into `Frontmatter`** (replace the preserved-unknown-key
   passthrough with the typed field). **Home: slice 04** — satisfaction must *read*
   a node's status, so slice 04 needs the typed field. This will flip arc01's
   `unknown_keys_preserved` 2→1 (status moves unknown→known) — an **expected,
   disclosed** change to a closed test, not a regression. *Folded into slice 04's
   slice-doc + ledger this turn.*
2. **`odm set-gate` CLI command** (persist a gate change to the node file).
   Needs a home — slice 04 stretch or an arc03/CLI slice. Tracked as an open item;
   not yet placed.

## Verdict

Arc 02 / Slice 03 **CDC-verified on structure; all items accepted; two deferrals
tracked (one folded into slice 04); cargo rows pending CI.** On CI green it closes
and slice 04 (derived order & evidence-leveled satisfaction) opens — the *consuming*
half, which also wires Status into Frontmatter.

CDC: planning thread, 2026-06-22. Iterations used: 1.
