# CDC Verification — Arc 02 / Slice 07: CLI graph-mutators

> Independent verification of CC's closed ledger (impl `c557790`; closed `4f48c39`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran;
> **attested** = CC's evidence, not reproduced here. Closes Arc 02.

## Environment constraint (disclosed)

Cowork sandbox has no 1.85+ toolchain; cargo-gated rows route to CI / local.

## Row dispositions

**Row count:** 14 opened (incl. CDC-added M-8b `decomposed`), 14 addressed. No silent
drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
no `unsafe`; the mutator subcommands all present (`Link`, `Unlink`, `SetGate`,
`Tear`, `Decomposed`, `New --parent`). → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
the 55 odm-cli tests incl. **M-11 self-host smoke** (a graph built purely via the
CLI answers `next`/`blocked`); clippy; coverage line 90.99% (commands.rs 91.66%).
→ **PENDING CI.**

## Headline finding — tear rationale is validated but NOT persisted (a real gap)

CC flagged it; confirmed: `Frontmatter`'s `tears: Vec<Dependency>` stores only the
edge — there is **no rationale field**. So `odm tear X depends_on Y --because
"<reason>"` validates the reason (via `Tear::new`) and then **drops it on persist**.
That defeats the *purpose* of requiring a rationale (ODD-0013 §4.3 / DSM "tearing":
the rationale is what keeps the assumed dependency *auditable and visible*). A torn
edge with no recorded "why" is exactly the silent-assumption the design forbids.

**Root cause is partly mine:** slice07's **M-7** only required "creates the tear;
empty rationale rejected" — it did not require the rationale to *persist + round-trip*,
and the `tears` schema never modeled it. So M-7 passed as written while the intent
went unmet — a spec-softening I should have caught when writing the row.

**Recommendation — fix it as a small dedicated slice (call it `slice08`, *not*
`07.1` — we don't repeat the bisection pattern).** Scope: change `tears` to carry
its rationale (a `TornEdge { edge, because }` rather than a bare `Dependency`);
persist + round-trip it; surface it in `check`'s active-tears listing. Small, but it
restores a load-bearing audit property. Tracked.

## Rulings on CC's other flags

2. **`--yes` is a no-op runner** (no interactive prompts yet). **Accepted** —
   forward-compatible.
3. **In-process dispatch tests, not `assert_cmd`.** **Accepted** (the established
   library-only pattern).
4. **`run()` uncovered in-process; 95% would need binary-level tests.** **Accepted.**
   *Tracked (now flagged twice — slice06 + here):* a binary-level `assert_cmd` suite
   in `oxur-odm/tests/` would cover `run()` + the real process's exit codes
   end-to-end. Worth a small slice or arc03 inclusion.

## Process notes

- The untracked CDC artifacts (slices 04/05/05.1/06) + arc-plan edit + ODD-0018 are
  now committed (`b36a0ce`) — the earlier git-hygiene drift is resolved. (This
  slice07 doc will be the next to commit.)
- CDC-inserted M-8b (`decomposed`) flowed through cleanly — CC implemented + verified.
- **ODD-0018** entered the corpus via `b36a0ce`; CDC is not yet read-in on it (raised
  for awareness — likely Duncan/CC's, possibly the telemetry/PM-tool thread).

## Verdict

Arc 02 / Slice 07 **CDC-verified on structure; cargo rows pending CI.** **Arc 02 is
complete** — odm is self-host-usable (build + advance a graph via the CLI; `check`
gates it). One follow-up before the arc is *fully* sound: the **tear-rationale
persistence** fix (proposed slice08). MVP now needs only Arc 03 (rollup/orient).

CDC: planning thread, 2026-06-22. Iterations used: 1.
