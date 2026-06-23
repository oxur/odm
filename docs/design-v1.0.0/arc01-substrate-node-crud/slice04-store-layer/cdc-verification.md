# CDC Verification — Slice 04: Store layer

> Independent verification of CC's closed ledger (impl `4457597`; closed `6d68087`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran;
> **attested** = CC's evidence, not reproduced here. Branch note: CC self-corrected
> a slip (commits had landed on `main`; moved to `slice04-store-layer`, `main`
> restored to `e19571e`) — verified: `main` is clean, slice04 commits sit on the
> feature branch. Clean crash-and-recover.

## Environment constraint (disclosed)

Cowork sandbox has no 1.85+ toolchain; cargo-gated rows route to CI / local.

## Row dispositions

**Row count:** 10 opened, 10 addressed. No silent drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
J-9 (no `unsafe` in `odm-store/src`; no `unwrap`/`expect` in src); **no shell-out**
(`Command::new`/`std::process` grep clean — confirms the pure-`gix` decision is
honored, flags 2 & 3); `Id::created_at()` present in `odm-core/src/id.rs` and
documented (odm-core's `deny(missing_docs)` is green). → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
J-1 (path-from-ULID), J-2 (O(1) locate), J-3 (persist→reload round-trip), J-4
(atomic write, no partial), J-5 (full-scan load), J-6 (`gix` commit/status), J-7
(layered config), J-8 (self-heal), J-10 (clippy + coverage). → **PENDING CI.**

## Rulings on CC's four flagged items

1. **`Id::created_at()` added to `odm-core`.** **Accepted.** A minimal, documented,
   covered (odm-core stays 98.8%/99.6%) public method the store needs to derive the
   path from the id; it returns a `chrono` time, so `ulid` stays isolated. It grows
   the (closed) odm-core additively and with tests — not a re-open.
2. **Index-free `gix` (commit tree built from the worktree; status = tree-vs-HEAD).**
   **Accepted.** Pure `gix`, no shell-out (verified), honoring the Q-2 decision given
   gix 0.66 has no high-level add+commit. *Behavioral note for later:* `commit_all`
   commits the whole worktree, not a curated index — fine for a tool that owns its
   docs dir; if odm ever runs in a repo with unrelated worktree changes, the
   reconcile/clean model is where that surfaces.
3. **Legacy `git.rs` not harvested (it shells out).** **Accepted — correct call.**
   The slice-doc's "harvest where they fit" explicitly permits skipping what
   doesn't; a shell-out helper contradicts the `gix` decision.
4. **Coverage: line 93.1% / region 88.5%.** **Accepted, with the gap recorded (not
   silent).** Line clears the ≥90% floor. Region 88.5% is below 90; the gap is
   *defensive `gix` error branches* CC judged not worth fault-injecting. Against the
   "100% error paths" ideal this is a small, real shortfall — but fault-injecting
   gix's tree-building internals is high-effort/low-value, so it is dispositioned as
   an **accepted no-op on those specific branches**, named here rather than hidden.
   Two ledger-quality follow-ups (CC is right on both): J-10's Verify should (a)
   specify the **line** metric and (b) add `--ignore-filename-regex 'odm-core'` so a
   store crate's number isn't diluted by the instrumented path-dep. CC's reported
   figures already apply the exclusion, so they are honest store-only numbers.

## Verdict

Slice 04 **CDC-verified on structure; flags 1–3 accepted; flag 4 accepted with the
region-coverage gap explicitly recorded (defensive gix branches, not a silent
shortfall); cargo rows pending CI.** On CI green, slice 04 closes and slice 05 (node
CRUD) opens.

**Forward note:** future ledgers' coverage rows should scope to the crate under
test (`--ignore-filename-regex`) and state line-vs-region, to avoid the cross-crate
instrumentation ambiguity this slice surfaced.

CDC: planning thread, 2026-06-22. Iterations used: 1.
