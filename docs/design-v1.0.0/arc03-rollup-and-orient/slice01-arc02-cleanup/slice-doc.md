# Slice 01 (Arc 03) — Arc 02 cleanup (plan-of-record)

> Refs: ODD-0013 §4.3 (tears), §2.3 (frontmatter schema), §4.5 (decomposition),
> §7 (command surface). `depends_on:` Arc 02 (on `main`, CI-green). Origin: three
> follow-ups flagged in Arc 02 CDC (slice06 + slice07 `cdc-verification.md` /
> `closing-report.md`), none blocking, all real.
>
> **Why this slice exists:** Arc 02 closed self-host-usable, but three CDC flags were
> left as nice-to-haves and two 0013 passages drifted from the realized code. This
> slice closes them *first* in Arc 03 so the rollup (slice02) can render tear
> rationale and `check` severities are calibrated before `orient` surfaces them.
> Orthogonal to the rollup/orient path — touches `odm-core`/`odm-store`/`odm-cli`/
> `oxur-odm`, nothing in the rollup/vision path.

## Goal

Close the three Arc 02 CDC follow-ups and reconcile the two 0013 passages they
touch. **Done when** a tear's rationale persists and round-trips and is shown by
`check`; the real `odm` process is exercised end-to-end by `assert_cmd` (covering
`run()`/exit codes); recomposition-finding severities match the staleness/
soft-satisfaction philosophy; and 0013 §2.3/§4.3/§4.5 describe the code as built.

## Scope

**In:**

- **Tear rationale persistence** (slice07 CDC flag #1; 0013 §4.3). Today `odm tear
  --because` *validates* the rationale via `Tear::new` but persists to a bare
  `tears: Vec<Dependency>`, so the text is dropped — defeating §4.3's audit purpose.
  Fix: `tears` entries carry `{ edge, because }` (a new typed `odm-core` frontmatter
  entry — distinct from `odm-graph`'s pure `Tear<N>` in `cycle.rs`, which the graph
  algorithms keep using; graph-build maps frontmatter tears → `Tear<N>` as today).
  `odm tear` persists the rationale on the **source** node; `parse ∘ emit = identity`
  holds with `tears` populated; `check`'s active-tears listing surfaces each
  rationale. Reverse edges stay derived.
- **Binary-level `assert_cmd` suite** in `oxur-odm/tests/` (slice06 + slice07 CDC).
  In-process `dispatch` tests cannot reach `run()` or the real process's `ExitCode`;
  this suite drives the built `odm` binary end-to-end: a clean graph → `EXIT_OK`; a
  violating graph → `EXIT_VIOLATIONS`; `--json` shape on the real binary. Lifts the
  `run()` coverage gap slice07 flagged (TOTAL line 90.99%, `run()` uncovered).
- **`check` severity recalibration** (slice06 CDC rec #2). Recomposition findings are
  currently all `Error`. Recalibrate to match how staleness/soft-satisfaction are
  treated: **orphan** and **decomposition-drift** stay `Error` (a structural break /
  a now-false assertion); **undeveloped-stub** and **advanced-without-decomposition**
  become `Warning` (advisory; fail only under `--strict`). So everyday `check` no
  longer exits 1 merely because an arc was advanced before its decomposition was
  affirmed.
- **0013 doc reconciliation** (keep-the-doc-honest class). (a) §2.3/§4.3: the `tears:
  []` bare list and the §4.3 prose ("required rationale") now agree — `tears` carries
  `{ edge, because }`. (b) §2.3/§4.5: `decomposed` is the typed `Decomposition { on,
  children }` realized in arc02 slice05, not the scalar `decomposed: complete` the doc
  still shows.

**Out:** rollup/orient (slice02/03); `reconcile`/drift (A5); any *new* `check`
categories; evidence-regression semantics (A7). No change to satisfaction /
terminal-gate behavior.

## Verification

`cargo test --workspace` green incl. the new tear-rationale round-trip and the
`oxur-odm` `assert_cmd` suite; `cargo clippy --all-targets -- -D warnings` exit 0; no
`unsafe`; coverage ≥ 90% (line) per affected crate (target 95%); the 0013 diffs
applied. Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI / local 1.85+). The three Arc 02
follow-ups are closed and 0013 is honest. Unblocks slice02 (tear rationale now
renderable in the rollup).
