---
id: 01KYP5GT17Z6D9A6N9WGBP352W
number: 503208100
type: artifact
schema: artifact/v1.1
name: 'CDC Verification — Arc 05 / Slice 01: `desired_facts` schema + `Probe` trait + shell probe'
created: 2026-06-30
updated: 2026-06-30
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc05-reconciliation/slice01-desired-facts-probe/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KWXMBBTKM4PA275KRNSWQTWB
---
# CDC Verification — Arc 05 / Slice 01: `desired_facts` schema + `Probe` trait + shell probe

> Independent verification of CC's closed ledger (impl + close on
> `arc05-slice01-desired-facts-probe`, commits `21cfbf1` + `f28f3b0`), per
> LEDGER-DISCIPLINE v2.0 (slice scale, §A). CDC reproduces structural rows here; cargo
> rows route to CI / a local 1.85+ run.

## Environment constraint (disclosed)

CDC's sandbox has no 1.85+ toolchain (apt cargo 1.75; the workspace is edition 2024 / MSRV
1.85). CC built + ran on a local rustc 1.95.0 — so CC's cargo rows are **attested** (run,
not blind-asserted), and CDC reproduces them via CI. The ledger header's "no 1.85 sandbox"
note is CDC's perspective; CC's note that a 1.95 toolchain was present is about CC's box —
it does not change the evidence strength (attested → reproduced on CI). CDC reproduces the
**structural** rows on the branch below.

## Row dispositions

**Row count:** 7 opened, 7 addressed (`done`). No silent drops. ✔

**Reproduced by CDC (structural):**

- **F-1** — `odm_core::desired` carries `DesiredFact` (`desired.rs:31`), `ProbeSpec`
  (internally-tagged `#[serde(tag="kind")]` enum, `:55`), `ShellExpect` (`:69`, `exit:i32`
  required + optional `stdout_contains`); `Frontmatter.desired_facts: Vec<DesiredFact>`
  (`frontmatter.rs:207`, defaulted `:241`). Named test `desired_facts_round_trip`
  (`tests/frontmatter.rs:575`) covers project + slice + empty-default. ✔
- **F-2** — empty-`id` rejected by a `deserialize_with="deserialize_non_empty_id"`
  validator (`desired.rs:35,85–91`) → serde `custom` error (which the YAML backend
  positions); test `desired_facts_malformed_errors_with_position` (`tests/frontmatter.rs:643`)
  covers unknown `kind` / missing `run` / empty `id`. ✔
- **F-3** — `odm-reconcile`: `enum ProbeOutcome` = `Holds | Drifted{expected,observed} |
  Error{reason}` (`lib.rs:36`), no `bool`; object-safe `trait Probe` (`:62`). Test
  `probe_outcome_has_three_variants` (`tests/probe.rs:25`). ✔
- **F-4** — `impl Probe for ShellProbe` (`lib.rs:136`) **execs directly** via
  `Command::new(program).args(&args)` over `split_whitespace()` (`:138,144`), **not** `sh -c`;
  spawn failure → `Error` (`:144–`). 9 tests in `tests/probe.rs` incl.
  `shell_probe_{holds_on_match, drifts_on_divergence, errors_on_unrunnable}` + stdout-match,
  empty-command, signal-termination. ✔ (see ruling 1)
- **F-5** — `ShellProbe` `# Trust model` doc (`lib.rs:70–90`): author-declared, local,
  user-privilege, no MVP sandbox; guardrails recorded as a no-op-with-rationale. `grep`
  matches. ✔
- **F-6** — `crates/odm-reconcile` in `[workspace] members` (`Cargo.toml:10`); no version
  literals in its manifest (`grep '= "[0-9]'` → none); CLAUDE.md crate table + publish
  order updated (also backfilled a missing `odm-index` row — A4 leftover, benign). ✔
- **F-7 (no `unsafe`)** — `grep` empty in `odm-reconcile/src` + `odm-core/src/desired.rs`;
  also denied workspace-wide (`[workspace.lints] rust.unsafe_code = "deny"`). ✔

**Attested by CC (local rustc 1.95.0), pending CI:** clippy `-D warnings` → exit 0; line
coverage **odm-reconcile 96% / odm-core desired.rs 100%** (≥ 90); full workspace green.
→ **PENDING CI.**

## Rulings on CC's flagged decisions

1. **Exec directly (whitespace-tokenized argv), not `sh -c`. Accepted — and a good call,
   correctly *derived* rather than bolted on.** Routing through `sh -c` would turn a missing
   binary into exit 127 — a *divergence* — collapsing the non-negotiable `Error ≠ Drifted`
   distinction (the whole point of the three-way outcome). Letting the spawn itself fail is
   what keeps "couldn't check" distinct from "checked, drifted." **Bounded cost, recorded:**
   no shell metacharacters, and `split_whitespace` can't express a quoted arg containing
   spaces (e.g. a path with a space). For the arc's probe shapes (`pg_isready -h prod -t 2`,
   exit-code checks) this is fine. *Carried to slice02:* if a real need for quoted/structured
   args appears, the clean answer is an **explicit `argv: Vec<String>` form** of the shell
   probe (not re-introducing `sh -c`) — noted in the arc-plan, not built now.
2. **Frontmatter `desired_facts` uses `skip_serializing_if` (empty omitted on emit).
   Accepted — and CC correctly narrowed *my* ledger guidance.** My F-1 note carried the A4
   "no `skip_serializing_if`" caution forward, but that lesson is **postcard-specific** (a
   non-self-describing binary stream desyncs on skip). YAML frontmatter is self-describing,
   so skipping an empty `desired_facts` is correct — it matches the existing `tears`/`edges`
   no-baseline-churn pattern. The always-serialized obligation lands on the **index record**,
   not the frontmatter — which is exactly what CC flagged for slice02/03. CC got the
   distinction right; my note was over-broad. (Honest correction: the guidance, not the impl,
   was the imprecise thing.)
3. **`ProbeSpec` internally-tagged + not `#[non_exhaustive]`.** Accepted — adding the `file`
   kind in slice02 becomes a compile error at every match site, which is stronger than a
   tolerated wildcard. Good additive-evolution discipline.

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its piece?** ✔ — A-1: the model + trait + shell probe the rest of A5 composes
  on, exactly as scoped. No runner / command / rollup wiring (correctly out of scope).
- **Silent-drop diff honest?** ✔ — 7/7; the exec-directly boundary and the skip/index
  nuance are disclosed in the closing-report, not buried.
- **Findings + arc-plan?** **CDC plan-keeping applied.** CC wrote the bubble-up in its
  `closing-report.md` but did **not** propagate it into `arc-plan.md` (the A-1 row + version
  history were untouched — the slice-close arc-plan-update step, PM Part IV). CDC closed that
  gap: A-1 row → attested-on-close with the `cdc-verification` pointer; arc-plan **v1.4**
  records slice01's two carry-forwards — (a) the index-integration of `desired_facts` is a
  **nested structured** field, so the slice02/03 adapter + fidelity-test extension is more
  than a scalar add (sharpens the carried invariant); (b) the `argv` escape hatch for the
  shell probe's no-metacharacters boundary.

## Verdict

**Arc 05 / Slice 01 CDC-verified on structure; all three flags ruled; cargo rows pending
CI.** The arc opens on a clean foundation: a node declares `desired_facts`, a malformed fact
is a positioned error, and the `Probe` trait's three-way outcome (with the exec-directly
shell probe) preserves the load-bearing `Error ≠ Drifted` distinction from the first line of
code. The next slice (02) is concretely scoped: the `file` probe + the probe-runner — and it
is the likely site of the **first index reader of `desired_facts`**, which must honor the
adapter-fidelity invariant (now sharpened: nested-field reconstruction). A-1 is
attested-on-close; flips to `done` on CI green.

CDC: planning thread, 2026-06-30. Iterations used: 1.
