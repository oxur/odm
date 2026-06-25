# CC Prompt — Slice 01 (Arc 03): Arc 02 cleanup

Close the three Arc 02 CDC follow-ups and reconcile the two 0013 passages they
touch. Small, contained, and orthogonal to the rollup/orient path — sequenced first
in Arc 03 so the rollup can render tear rationale.

> **Start condition:** Arc 02 is on `main`, CI-green (it is). No upstream slice
> dependency. If `main` isn't green, hold.

## Read first
1. `slice01-arc02-cleanup/ledger.md` (11 rows).
2. `slice-doc.md` (same dir).
3. `docs/design/01-draft/0013-odm-architecture-design.md` §2.3 (schema), §4.3 (tears),
   §4.5 (decomposition), §7 (commands).
4. The Arc 02 flags being closed: `arc02-.../slice07-cli-graph-mutators/closing-report.md`
   (Flagged for CDC #1, tear rationale) and `slice06-check-v2/cdc-verification.md`
   (rulings #1 assert_cmd, #2 severity recalibration).

## Load skills
- **rust-guidelines**: `11-anti-patterns.md`, `05-type-design.md`, `02-api-design.md`,
  `14-cli-tools/03-error-handling.md`.
- **collaboration-framework → LEDGER_DISCIPLINE**.

## Task

1. **Persist the tear rationale.** Change `tears` (frontmatter — currently
   `Vec<Dependency>` in `crates/odm-core/src/frontmatter.rs`) to a typed entry that
   carries both the torn `edge` (a `Dependency`) **and** the `because` rationale.
   **Naming:** `odm-graph` already defines a pure `Tear<N>` (`cycle.rs`) used by the
   graph algorithms — name the new frontmatter/persistence type distinctly (e.g.
   `TornEdge`) to avoid the collision; graph-build keeps mapping frontmatter tears →
   `Tear<N>` as today. `odm tear X depends_on Y --because <r>` persists the rationale
   on the **source** node (today it validates via `Tear::new` then drops it).
   `parse ∘ emit = identity` with `tears` populated; omit `tears` when empty (arc01/02
   nodes round-trip byte-identically). Surface each rationale in `check`'s
   active-tears listing.
2. **Binary-level `assert_cmd` suite** in `oxur-odm/tests/`. Drive the built `odm`
   binary end-to-end (the in-process `dispatch` tests cannot reach `run()` or the
   real `ExitCode`): clean graph → `EXIT_OK`; violating graph → `EXIT_VIOLATIONS`;
   `--json` shape on the real binary. This is the suite slice06/07 CDC asked for.
3. **Recalibrate `check` recomposition severities.** orphan + decomposition-drift =
   `Error`; undeveloped-stub + advanced-without-decomposition = `Warning` (fail only
   under `--strict`). Match the existing staleness / soft-satisfaction treatment.
4. **Reconcile 0013.** §2.3/§4.3: `tears` carries `{ edge, because }` (not a bare
   list). §2.3/§4.5: `decomposed` is the typed `Decomposition { on, children }`
   realized in arc02 slice05, not the scalar `decomposed: complete`.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** If a criterion is wrong or impossible, raise an
  amendment — don't quietly skip it.
- Reverse edges stay derived (never written). Mutations persist atomically (odm-store).
- **Back-compat:** a pre-change node must still parse; emitting an empty `tears` must
  not invent the key. If any on-disk node carries a bare-form tear, note the migration
  (no-op expected — pre-release, test fixtures only).
- Errors-as-affordances on every failure path. No `unsafe`; typed errors
  (`thiserror` in libs); coverage ≥ 90% (line), target 95%.

## Deliverables
Green `cargo test --workspace` (incl. the `oxur-odm` `assert_cmd` suite) + clippy +
coverage; `ledger.md` evidence per row; `closing-report.md` (per-row walk for all 11,
What Worked, uncertainties named); the 0013 diff. Feature branch
(`arc03-slice01-arc02-cleanup`); not `main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration
cap; your `done` is *proposed done* → CDC verifies (cargo rows via CI / local 1.85+).
