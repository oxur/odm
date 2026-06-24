# CDC Verification — Slice 05: Node CRUD commands

> Independent verification of CC's closed ledger (impl `f33c983`; refactor `af190ab`;
> docs `2b3b2d8`), per LEDGER_DISCIPLINE. Evidence levels (0001-D3): **reproduced**
> = CDC re-ran; **attested** = CC's evidence, not reproduced here. The Slice 04 CI
> fix (`d00f436`) rides on this branch and is assessed below too.

## Environment constraint (disclosed)

Cowork sandbox has no 1.85+ toolchain; cargo-gated rows route to CI / local.

## Row dispositions

**Row count:** 11 opened, 11 addressed. No silent drops. ✔

**Reproduced by CDC (structural, re-run in-session):**
odm-cli is **library-only** (no `[[bin]]`, no `src/main.rs`); `dispatch(...)` is the
in-process entry point; `assert_cmd`/`predicates` dev-deps removed; **no `unsafe`,
no `unwrap`/`expect`** in `odm-cli/src`; the published binary is still `odm`
(oxur-odm umbrella); odm-core's retire additions present (`frontmatter_mut`,
`retired: Option<Retirement>`, `retire()`, `edges_mut`). → **PASS.**

**Attested by CC, pending CI / 1.85+ reproduction:**
K-1…K-11 (the 22 in-process dispatch tests; clippy; line 96.17% / region 90.31%
coverage — both clear ≥90). → **PENDING CI.** *Coverage caveat (CC's, valid):* after
deleting the binary, a stale `target/llvm-cov-target` produced a phantom `main.rs`
and a bogus ~50% — CDC must run coverage from a **clean** state.

## Slice 04 CI fix (`d00f436`) — accepted

The slice04 CI failure was `gix` `commit_as` setting the commit object's identity
from the passed signature, while the **HEAD reflog** write draws its identity from
`repo.committer()` (git config), which CI lacks. Fix seeds an **in-memory**
`user.name`/`user.email` after init/open — verified by `config_snapshot_mut` (never
touches `.git/config`). Correct root-cause and fix; CC reproduced the CI condition
locally (empty HOME + `GIT_CONFIG_NOSYSTEM=1` + `GIT_CONFIG_GLOBAL=/dev/null`, 22
pass) — a faithful CI repro. Slice 04 closes on the CI green that carries this fix.

## Rulings on CC's five flagged items

1. **`retired` field extends §2.3 (undocumented).** **Valid — fixed.** This is a
   C5-class doc drift (a new schema field not in the doc). Documented in **0013
   v1.7** (`retired: { reason, on }`, optional, set by `odm retire`, file kept).
   Added **Q-10**: Arc 02 may fold retirement into the gate model — decide when
   gates land.
2. **`supersede X --with Y` records the edge on Y→X (source = newer, per §3).**
   **Accepted — correct per §3.** The CLI names the old node first for ergonomics;
   the stored edge direction is right. No change.
3. **`--yes` is an accepted affirmation (no interactive prompt yet).** **Accepted.**
   Forward-compatible; `--dry-run` is the substantive half today.
4. **Diagnostics routed to stderr by hand, not via oxur-cli helpers (which print to
   stdout).** **Accepted — correct call.** CC prioritized the master rule
   (data→stdout, diagnostics→stderr; CLI-27) over "use the helper." *Logged: that
   oxur-cli's output helpers send diagnostics to stdout is a latent upstream issue
   worth a ticket later — not now.*
5. **New CLI test binary → refactored to in-process dispatch (binary deleted).**
   **Accepted — better design.** Operator-approved (option 1). Verified structurally
   (no `[[bin]]`, `dispatch` present, dev-deps dropped, binary still `odm`). The
   in-process tests avoid subprocess + global-cwd races. Good outcome from the
   pushback.

## Watch-item for CI

`retired` was added to odm-core's `Frontmatter` *after* slice03's round-trip
proptest was written. Confirm the round-trip proptest exercises `retired: Some(...)`
(not just the `None` default) so the new field is covered, not merely defaulted.

## Verdict

Slice 05 **CDC-verified on structure; all five items dispositioned (one → a doc
fix); the Slice 04 CI fix accepted; cargo rows pending CI.** On CI green (clean
coverage state), slices 04 and 05 close together and slice 06 (`check` v1) opens.

CDC: planning thread, 2026-06-22. Iterations used: 1.
