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
| F-1 | A node declares `desired_facts`: an optional list of `{ id, describe, probe }`; absent/empty is the default; it round-trips through the frontmatter serde layer (parse → emit → parse is stable) on **any node type, incl. the project node** | `cargo test -p odm-core desired_facts_round_trip` (covers a project node + a slice node + the empty default) → ok | serious | 0013 §5.2 / arc-plan slice01 | done | `cargo test -p odm-core desired_facts_round_trip` → ok (1 passed); covers project + slice + empty-default nodes, accessor, empty-skipped-on-emit, and the proposed YAML shape. **attested** | Program-level facts = facts on the project node; no separate layer (arc-plan v1.3). Serde-evolution caution (A4 slice02): always-serialized fields, no `skip_serializing_if` surprises. |
| F-2 | A malformed `desired_facts` entry (unknown probe `kind`, missing required field, empty `id`) is a **build/parse error carrying `Position`**, not a silent drop or panic | `cargo test -p odm-core desired_facts_malformed_errors_with_position` → ok | serious | project error convention (CLAUDE.md) | done | `cargo test -p odm-core desired_facts_malformed_errors_with_position` → ok (1 passed); unknown `kind`, missing `run`, and empty `id` each return `FrontmatterError::Yaml` whose message carries `line`/`column`; empty-id also names the field. **attested** | Mirrors the parse/build-error-with-`Position` rule used across the workspace. Realized via the YAML backend's positioned error (the existing `Yaml(String)` channel); empty-`id` is a `deserialize_with` semantic check that serde still positions. |
| F-3 | The `Probe` trait + `ProbeOutcome` result model exist in `odm-reconcile`: outcome is the **three-way** `Holds` \| `Drifted { expected, observed }` \| `Error { reason }` (not a `bool` — "couldn't check" ≠ "checked, drifted") | `cargo test -p odm-reconcile probe_outcome_has_three_variants` + `grep -nE "enum ProbeOutcome|trait Probe" crates/odm-reconcile/src` | serious | 0013 §5.2 | done | `cargo test -p odm-reconcile probe_outcome_has_three_variants` → ok; `grep -nE "enum ProbeOutcome\|trait Probe" crates/odm-reconcile/src/lib.rs` → `enum ProbeOutcome` (l.36), `trait Probe` (l.62). Three variants `Holds`/`Drifted{expected,observed}`/`Error{reason}`; no `bool`. **attested** | The contract slices 02–04 build on. Keep minimal + additive-friendly. |
| F-4 | The **shell** probe maps a declared command to the outcome: expectation met → `Holds`; exit/stdout diverges → `Drifted{expected, observed}`; command cannot run → `Error` | `cargo test -p odm-reconcile shell_probe_{holds_on_match,drifts_on_divergence,errors_on_unrunnable}` → ok (real local commands: `true`/exit 0, a diverging exit, a non-existent binary) | serious | arc-plan slice01 | done | `cargo test -p odm-reconcile` → 9 passed, incl. `shell_probe_holds_on_match` (`true`→Holds), `shell_probe_drifts_on_divergence` (`false`→Drifted{exit 0, exit 1}), `shell_probe_errors_on_unrunnable` (missing binary→Error); plus stdout-match holds/drift, empty-command Error, and signal-termination Error. **attested** | Drift vs. Error distinction is load-bearing — test both, not just holds. **Design note (flagged):** the probe execs the command directly (whitespace-tokenized argv), *not* via `sh -c` — that is precisely what makes "cannot run → Error" distinct from "ran & diverged → Drift" (the non-negotiable distinction). No-shell-metacharacters is a documented MVP boundary. |
| F-5 | The shell probe's **trust model is documented**: author-declared commands run locally with the user's privileges; **no sandbox** in the MVP; the boundary is stated in the probe's doc comment | `grep -nE "trust|author-declared|no sandbox|privilege" crates/odm-reconcile/src` shows the doc comment; cross-ref slice-doc | serious | arc-plan v1.3 (resolved Q) | done | `grep -nE "trust\|author-declared\|no sandbox\|privilege" crates/odm-reconcile/src/lib.rs` → matches in the `ShellProbe` `# Trust model` doc (ll. 70–90): author-declared, local, user-privilege, no MVP sandbox. **attested** | A security-adjacent boundary — documented explicitly, not assumed. Guardrails (`--no-exec`/allowlist) deferred (out of MVP scope) and recorded **as a no-op-with-rationale in the doc comment** — no code needed this slice. |
| F-6 | `odm-reconcile` is a workspace member: manifest references `[workspace.dependencies]` (no version literals) and inherits `[workspace.lints]`; it builds | `cargo build -p odm-reconcile` → ok AND `grep -nE "workspace = true|\.workspace = true" crates/odm-reconcile/Cargo.toml` AND `! grep -nE '= "[0-9]' crates/odm-reconcile/Cargo.toml` (no version literals) | serious | CLAUDE.md (workspace conventions) | done | `cargo build -p odm-reconcile` → ok; `grep -nE "workspace = true\|\.workspace = true" crates/odm-reconcile/Cargo.toml` → 8 matches (package inheritance + `[lints]` + `tempfile`); `grep -nE '= "[0-9]' …Cargo.toml` → no matches (no version literals). Added to `[workspace] members`; CLAUDE.md crate table + publish-order note updated. **attested** | Mirrors A4 slice01 creating `odm-index`. New crate added to `[workspace] members` + publish order note. (Also added the missing `odm-index` row to the CLAUDE.md table — an A4 leftover, fixed in passing.) |
| F-7 | Clippy clean (`-D warnings`); no `unsafe`; coverage ≥ 90% (line) for `odm-reconcile` + the `odm-core` additions | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0 AND `! grep -RnE '\bunsafe\b' crates/odm-reconcile/src` AND `cargo llvm-cov --summary-only` (the two crates) → **line** ≥ 90% | serious | CLAUDE.md | done | `cargo clippy --all-targets --all-features -- -D warnings` → exit 0; `grep -RnE '\bunsafe\b' crates/odm-reconcile/src crates/odm-core/src/desired.rs` → none (also denied workspace-wide by `[workspace.lints] rust.unsafe_code = "deny"`); `cargo llvm-cov --summary-only -p odm-reconcile -p odm-core` → `odm-reconcile/src/lib.rs` **96.00% line**, `odm-core/src/desired.rs` **100.00% line** (both ≥ 90). **attested** | |

## What Worked

- **Empirically probing serde_norway before designing F-2.** A 5-minute throwaway
  experiment confirmed that unknown-variant, missing-field (in the nested sequence
  context `desired_facts` actually uses), and custom `deserialize_with` errors all
  carry `line`/`column`. That turned "carry `Position`" from an assumption into a
  verified property and settled the empty-`id` approach (a `deserialize_with`
  validator) before a line of schema was written.
- **Internally-tagged enum (`#[serde(tag = "kind")]`) for `ProbeSpec`.** It gives
  the additive wire-shape stability the slice demands for free, and — by *not*
  marking it `#[non_exhaustive]` — turns "add the `file` kind" into a compile error
  at every match site in slice02, which is stronger than a tolerated wildcard.
- **Exec-directly (not `sh -c`) fell out of the Error/Drift requirement.** Letting
  the spawn itself fail is what makes "cannot run" a distinct outcome; the design
  was driven by the non-negotiable three-way split rather than bolted on.
- **A local 1.95 toolchain was present** (despite the ledger's "no 1.85 sandbox"
  note), so every cargo row was built and run, not blind-attested — CDC reproduces,
  but nothing here is asserted-only.

## Closure

Closed at commit `a3cf014` on 2026-06-30. Verified by: CC (self-attested); CDC to
reproduce. Rows: 7. Done: 7. Deferred: 0. No-op: 0.

> Per LEDGER-DISCIPLINE v2.0 §A, all evidence is **attested** (CC built and ran
> every cargo/grep row on a local 1.95 toolchain). Independent **reproduction**
> (CI / local 1.85+) elevates these to `reproduced`; recorded in
> `cdc-verification.md`.
