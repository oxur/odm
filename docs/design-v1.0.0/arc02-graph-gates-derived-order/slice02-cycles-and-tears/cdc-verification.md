# CDC Verification — Arc 02 / Slice 02: Cycle detection + tears

> Independent verification of CC's closed ledger (impl `b7514cf`; closed `be77af5`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran;
> **attested** = CC's evidence, not reproduced here.

## Environment constraint (disclosed)

Cowork sandbox has no 1.85+ toolchain; cargo-gated rows route to CI / local.

## Row dispositions

**Row count:** 8 opened, 8 addressed. No silent drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
engine stays domain-agnostic (`odm-graph` has no `odm-core` dependency, even after
cycle code); no `unsafe` in `odm-graph/src` (H-7 half); `Cycle<N>` is a typed
`std::error::Error` (H-1/H-5 share one type); `Tear::new` rejects an empty/whitespace
rationale via `MissingRationale` — an unjustified tear is unrepresentable (H-4);
`detect_cycle(ordering_kinds, tears)` and `active_tears(...)` present, taking the
ordering kinds + tears **per call** (engine stays pure/generic). → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
H-1 (detect + name members), H-2 (acyclic ⇒ none), H-3 (tear breaks cycle), H-4
(tear requires rationale), H-5 (cycle-without-tear is a hard error), H-6 (list
active tears), H-7 (clippy), H-8 (line 93.58%). 16 odm-graph tests. → **PENDING CI.**

## Rulings on CC's flagged items

1. **Kahn detects, DFS names.** **Accepted — correct technique.** Kahn's residual
   (nodes with non-zero in-degree after peeling) over-includes nodes merely
   *downstream* of the cycle; a DFS back-edge walk isolates the true members.
   Pinned by a dedicated test. Sound.
2. **`Option<Cycle>` where `Cycle: Error`.** **Accepted — elegant.** One type
   satisfies both "names the members" (H-1) and "hard error" (H-5); slice 06's
   `check` consumes it directly as a failure (verified `impl Error for Cycle`).
3. **Tears + ordering-kinds passed per call.** **Accepted — good design.** Keeps
   the engine pure and domain-agnostic (verified: no `odm-core` dep), and the same
   primitive will power `next`/`blocked` in slice 04 (the caller — `odm-core` —
   supplies which edge kinds are "ordering").

## Uncertainty (CC-named) — accepted with the gap recorded

~9% of `cycle.rs` is defensive guards (an `extract_cycle` fallback CC judges
unreachable, a duplicate-queue guard) left un-fault-injected; line coverage 93.58%
clears the ≥90 floor. Accepted as a **named gap**, not silent — consistent with how
slice 04's gix defensive branches were dispositioned. *Minor option for later:* an
`unreachable!()` with an invariant comment is more honest than an untested
defensive branch where the state is genuinely impossible — CC's call either way.
"One cycle reported even if several exist" is fine for v1 (deterministic; one is
enough to fail `check`); enumerating all can come later if a use-case wants it.

## Verdict

Arc 02 / Slice 02 **CDC-verified on structure; all items accepted; cargo rows
pending CI.** On CI green, it closes and slice 03 (gates/status/evidence) opens —
where the `Evidence` total order + per-type gate-sets land (the recording half of
evidence-leveled satisfaction).

CDC: planning thread, 2026-06-22. Iterations used: 1.
