# CDC Verification — Arc 02 / Slice 05: Decomposition/recomposition integrity

> Independent verification of CC's closed ledger (impl `f3bd4ba`; closed `8d6dc29`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran;
> **attested** = CC's evidence, not reproduced here.
>
> **Process note:** this slice was built and CC-closed before CDC ran (CDC attention
> was on slice 04, and 05.1 was inserted on top). Verifying retroactively — the slice
> must not be treated as advanced until this exists. Logged so the gap is visible,
> not silent.

## Environment constraint (disclosed)

Cowork sandbox has no 1.85+ toolchain; cargo-gated rows route to CI / local.

## Row dispositions

**Row count:** 10 opened, 10 addressed (per CC's closing report). No silent drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
the `recompose` module exists; `Frontmatter` carries `decomposed:
Option<Decomposition>` with `affirm_decomposed(children, on)`; `graph.children()`
provides reverse-`part_of` enumeration; no `unsafe` in odm-core. The module doc
frames the checks as what the structure *proves* (orphans, undeveloped stubs, drift
against the affirmed set) — consistent with the **structural-only** mandate (§4.5).
→ **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
H-1…H-10 — total recomposition, single-parent totality, orphan detection, no-stub,
`decomposed: complete` assertion + drift guard, advance-without-decomposition,
the *no-semantic-scope-guessing* non-goal, clippy, coverage. → **PENDING CI.**

## Notes

- §4.5's `decomposed: complete` is realized as a richer typed `Decomposition { on,
  children }` (the affirmed child set + date), not the scalar the §2.3 sketch
  implied — which is what makes the **drift guard** possible (children added/removed
  after affirmation is detectable). Good call; 0013 §2.3's `decomposed` mention
  should be reconciled to the typed shape (minor doc touch-up, like the earlier
  edge-order/`retired` fixes).
- The non-goal (no automatic missing-scope *guessing*) is honored — the engine
  reports only what `part_of` proves.

## Verdict

Arc 02 / Slice 05 **CDC-verified on structure; cargo rows pending CI.** Recomposition
integrity (the structural half of Q-7) is in. On CI green it closes.

CDC: planning thread, 2026-06-22. Iterations used: 1.
