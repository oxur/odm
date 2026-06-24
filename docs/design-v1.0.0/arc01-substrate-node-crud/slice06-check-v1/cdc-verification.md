# CDC Verification — Slice 06: `check` v1 + link-integrity

> Independent verification of CC's closed ledger (impl `5b39c54`; closed `7f81b22`),
> per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced** = CDC re-ran;
> **attested** = CC's evidence, not reproduced here. **Last slice of Arc 01.**

## Environment constraint (disclosed)

Cowork sandbox has no 1.85+ toolchain; cargo-gated rows route to CI / local.

## Row dispositions

**Row count:** 9 opened, 9 addressed. No silent drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
`odm-core::check` is a **pure validator** — `check(&[Frontmatter]) -> Vec<Finding>`,
no `std::fs`/`io`/walkdir (architecture per §8); `Violation` is `#[non_exhaustive]`
(v2 extends without rewrite); the three finding families are present (completeness,
link-integrity over all 8 edge kinds, supersession-chain); exit codes wired
(`dispatch` → code, `run` → `ExitCode`, `main` → `ExitCode`); **no `unsafe`** in
check or cli. → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
L-1…L-9 (30 odm-cli + 9 odm-core check tests; clippy; line 95.84%, check.rs 99%;
`odm check` clean → exit 0). → **PENDING CI.**

## Rulings on CC's four flagged items

1. **v1 completeness = non-empty `name` only.** **Accepted.** The other mandatory
   fields are *parse*-enforced — a missing one fails to load and never reaches the
   validator — so `name` (present-but-empty) is the only validator-reachable
   completeness case at v1. The per-type completeness table is the v2 extension
   point (`Violation` is `#[non_exhaustive]`). Sound.
2. **Exit model 0/1/2 (slice05 command errors now exit 2, was 1).** **Accepted.**
   `2 = usage/error` is more principled than folding errors into `1 = violations`;
   no test pinned the old behavior, so not a regression — a clean refinement.
3. **In-process tests (not `assert_cmd`).** **Accepted** — same operator-approved
   pattern as slice05; the `assert_cmd` hint is superseded.
4. **Fix affordances.** **Accepted.** A real command where one exists (empty-name →
   `odm rename`) and precise file-edit instructions for dangling/cycle cases (since
   `link`/`unlink` arrive in Arc 02). Errors-as-affordances honored within what v1
   can offer.

## Verdict

Slice 06 **CDC-verified on structure; all four items accepted; cargo rows pending
CI.** On CI green, slice 06 closes — and with it **Arc 01 is complete**: identity
(02) → schema (03) → store (04) → CRUD (05) → structural check (06). "Files are the
source" is proven end-to-end. Arc 02 builds the graph engine and `check` v2 by
*extending* `odm_core::check` (the `#[non_exhaustive]` `Violation` is the seam).

CDC: planning thread, 2026-06-22. Iterations used: 1.
