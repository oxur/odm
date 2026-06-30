# Slice 01 (Arc 05): `desired_facts` schema + `Probe` trait + shell probe

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`
> at slice scale. CC fills evidence at `attested` per commit; CDC reproduces (CI / local
> 1.85+ — the sandbox has no 1.85 toolchain). Five-iteration cap. **Arc opener** — the
> model + trait + first probe the rest of A5 composes on. No runner / command / rollup
> wiring this slice (slices 02–04).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | A node declares `desired_facts`: an optional list of `{ id, describe, probe }`; absent/empty is the default; it round-trips through the frontmatter serde layer (parse → emit → parse is stable) on **any node type, incl. the project node** | `cargo test -p odm-core desired_facts_round_trip` (covers a project node + a slice node + the empty default) → ok | serious | 0013 §5.2 / arc-plan slice01 | open | | Program-level facts = facts on the project node; no separate layer (arc-plan v1.3). Serde-evolution caution (A4 slice02): always-serialized fields, no `skip_serializing_if` surprises. |
| F-2 | A malformed `desired_facts` entry (unknown probe `kind`, missing required field, empty `id`) is a **build/parse error carrying `Position`**, not a silent drop or panic | `cargo test -p odm-core desired_facts_malformed_errors_with_position` → ok | serious | project error convention (CLAUDE.md) | open | | Mirrors the parse/build-error-with-`Position` rule used across the workspace. |
| F-3 | The `Probe` trait + `ProbeOutcome` result model exist in `odm-reconcile`: outcome is the **three-way** `Holds` \| `Drifted { expected, observed }` \| `Error { reason }` (not a `bool` — "couldn't check" ≠ "checked, drifted") | `cargo test -p odm-reconcile probe_outcome_has_three_variants` + `grep -nE "enum ProbeOutcome|trait Probe" crates/odm-reconcile/src` | serious | 0013 §5.2 | open | | The contract slices 02–04 build on. Keep minimal + additive-friendly. |
| F-4 | The **shell** probe maps a declared command to the outcome: expectation met → `Holds`; exit/stdout diverges → `Drifted{expected, observed}`; command cannot run → `Error` | `cargo test -p odm-reconcile shell_probe_{holds_on_match,drifts_on_divergence,errors_on_unrunnable}` → ok (real local commands: `true`/exit 0, a diverging exit, a non-existent binary) | serious | arc-plan slice01 | open | | Drift vs. Error distinction is load-bearing — test both, not just holds. |
| F-5 | The shell probe's **trust model is documented**: author-declared commands run locally with the user's privileges; **no sandbox** in the MVP; the boundary is stated in the probe's doc comment | `grep -nE "trust|author-declared|no sandbox|privilege" crates/odm-reconcile/src` shows the doc comment; cross-ref slice-doc | serious | arc-plan v1.3 (resolved Q) | open | | A security-adjacent boundary — documented explicitly, not assumed. Guardrails (`--no-exec`/allowlist) deferred (out of MVP scope), recorded as a no-op-with-rationale if no code is needed. |
| F-6 | `odm-reconcile` is a workspace member: manifest references `[workspace.dependencies]` (no version literals) and inherits `[workspace.lints]`; it builds | `cargo build -p odm-reconcile` → ok AND `grep -nE "workspace = true|\.workspace = true" crates/odm-reconcile/Cargo.toml` AND `! grep -nE '= "[0-9]' crates/odm-reconcile/Cargo.toml` (no version literals) | serious | CLAUDE.md (workspace conventions) | open | | Mirrors A4 slice01 creating `odm-index`. New crate added to `[workspace] members` + publish order note. |
| F-7 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for `odm-reconcile` + the `odm-core` additions | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-reconcile/src` AND `cargo llvm-cov --summary-only` (the two crates) → **line** ≥ 90% | serious | CLAUDE.md | open | | |

## What Worked

_(At slice close. Patterns that made the slice close cleanly.)_

## Closure

_(At slice close.)_ Closed at commit `<SHA>` on `<date>`. Verified by: `<CDC/session>`.
Rows: 7. Done: `<n>`. Deferred: `<n>`. No-op: `<n>`.
