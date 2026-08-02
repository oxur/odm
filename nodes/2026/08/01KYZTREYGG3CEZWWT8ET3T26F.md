---
id: 01KYZTREYGG3CEZWWT8ET3T26F
number: 579082500
type: artifact
schema: artifact/v1.1
name: 'Slice 14 (Migration Fidelity) — CDC verification: Config-driven migrate roots'
created: 2026-07-31
updated: 2026-07-31
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice14-migrate-config-roots/cdc-verification.md
  class: cdc-verification
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-08-01
edges:
  part_of: 01KYZTRDEQE5EMGN59BPH3ASYR
---
# Slice 14 (Migration Fidelity) — CDC verification: Config-driven migrate roots

> **Arc:** Migration Fidelity · **Slice:** 14 · **Verifier:** CDC (independent) · **Date:** 2026-07-31 ·
> **Method:** LEDGER-DISCIPLINE v2.0 §A. **Fixture slice — no live class-(b) row.** Code + fixtures + the
> ODD-0022 amendment reproduced by direct read on `release/1.0.x`; runtime execution
> (`cargo`/`clippy`/`llvm-cov`) attested-by-CC → CI.
> **Under review:** `ledger.md` (F-1…F-10), commits `45ca74c` (code + ODD + fixtures), `32f60e2` (close) —
> both `release/1.0.x`. `odm` store HEAD unchanged at `e06fffe` (reproduced — no live mutation).

## Verdict

**PASS — CDC-verified.** `odm migrate` is now config-driven with a clean, well-tested implementation, the
`docs_directory` + `"design"` append is restored, additional-paths are a first-class persistent
sweep-and-remember, and the set-subtraction dedup is correct. Both flagged decisions (D-1 umbrella root,
D-2 additional-path derivation) were resolved exactly as the slice-doc recommended. CC found and
honestly disclosed a real second bug (the `[legacy]` config-home resolution) and two arc-close findings —
one of which is a **hard, blocking precondition for the freeze that I am elevating** (§4). No live store
mutation (`odm@e06fffe` unchanged). A clean slice, and a genuinely thorough one — every ledger row carries
a dedicated, non-vacuous test.

## 1. The three defects — fixed, reproduced

| Defect | Fix reproduced in code | Test (read, non-vacuous) |
|--------|------------------------|--------------------------|
| **Wrong positional** (F-1) | `lib.rs`: `legacy_path: String` **removed**; `additional_paths: Option<String>` added (optional, comma-split → trimmed `Vec<String>`, empties dropped). Every mode resolves roots via `migrate.rs::docs_root(root)` from config; a missing `docs_directory` is a named error, not a silent fallback | `migrate_help_shows_no_legacy_path_and_an_optional_additional_paths`; `migrate_coverage_and_artifacts_resolve_docs_root_from_config_not_a_positional` |
| **Dropped `"design"` append** (F-2) | `commands::configured_design_directory` = `configured_docs_directory(root).join("design")`; `all()` uses it, **never** `docs_root` as-is. The NN-state + §2.4 rules scope to that subtree only | `migrate_all_applies_design_rules_only_under_docs_directory_design` — a design doc under `docs/design/…` gets the `Design` derivation; a loose `.md` directly under `docs/` does **not** (asserted both directions) |
| **No persistent additional-paths** (F-4/F-5) | `write_additional_paths` unions the passed strings with `[legacy].additional_paths`, **sorts + dedups**, writes back via `toml_edit` (format-preserving), builds an explicit `[legacy]` table; re-read every run via `configured_additional_paths` | `migrate_all_sweeps_and_persists_additional_paths`; `migrate_all_additional_paths_union_sorts_and_dedupes_across_runs` |

## 2. The rest of the ledger — reproduced

- **F-3 / D-1 (root map from config).** `docs_root(root)` = configured `docs_directory` (the umbrella
  parent); design = `docs_directory/design`; dev = `dev_directory` (as-is); plan-set self-host +
  `--artifacts` sweep over the umbrella. D-1 adopted as recommended (reuse `docs_directory` as the parent,
  no dedicated key), documented on the `docs_root` fn. No mode reads a corpus positional.
- **F-4 / D-2 (additional-path derivation).** `migrate_additional` autodetects: a Plan-shaped dir gets the
  self-host escape hatch, else the `--artifacts` generic derivation — exactly the recommended resolution.
  `mint_artifacts` mints nothing over an empty/already-covered dir, so it's idempotent-safe either way.
  Test: `migrate_all_plan_set_escape_hatch_self_hosts_an_additional_arc_directory`.
- **F-6 (config-home) — the second bug, well-handled.** `configured_additional_paths` /
  `write_additional_paths` read/write `StoreHome::resolve(root).operational_text()` / `.operational_path`
  — the store's `config.toml`, the file the code actually resolves. The `[legacy]` block the operator had
  added to the code-branch **locator** `odm.toml` is *not* that file once a store `config.toml` exists.
  ODD-0022 **v1.1** records the decision (the `[legacy]` sub-table lives in the operational config; the
  locator's is ignored; a v1.0+ config must carry `[legacy]` where the code reads it for re-run
  idempotency). Proven by `migrate_all_resolves_legacy_config_from_a_split_store_not_the_locator` — which
  plants a **decoy** `[legacy] docs_directory = "./wrong"` in the locator and asserts the store-config
  value wins. This is the amend-not-work-around discipline, done right.
- **F-7 (set-subtraction dedup).** `effective_additional_paths` = `(positional ∪ config)` resolved,
  sorted, deduped, then filtered by `overlaps(a,b) = a==b || a.starts_with(b) || b.starts_with(a)` against
  `design_root`, `dev_root`, and every `plan_root` — symmetric containment, so an additional that *is*,
  sits *under*, or *contains* an owned root is excluded from the generic pass. Crucially, the
  **persist-then-subtract order** is right: `write_additional_paths` runs *before* the subtraction, so an
  excluded-from-processing path is still *remembered* in config. Test:
  `migrate_all_excludes_an_additional_nested_under_design_or_dev_from_the_generic_pass`.
- **F-8 (idempotent + dry-run-safe).** `write_additional_paths` returns early on `dry_run`, and no-ops when
  the sorted-deduped union equals what's already stored. Tests: `migrate_all_is_idempotent`,
  `migrate_all_dry_run_writes_nothing`, `migrate_all_dry_run_writes_no_config`.
- **F-9 (no model drift).** The design/dev/artifact/note/vision derivations are unchanged — only their
  *root resolution* moved to config, plus the new additional/dedup/persist logic. The one normative touch
  is ODD-0022 v1.1 (the `[legacy]` sub-table) — cited, amended, not worked around.
- **F-10.** `unsafe`-free; clippy/coverage → CI. The substantive new logic (root resolution, the `overlaps`
  set-ops, the `toml_edit` write-back, the autodetect) each carries a dedicated fixture; coverage is not
  vacuous.

## 3. No live mutation — reproduced

`odm` branch HEAD unchanged at `e06fffe`. s14 is `release/1.0.x` code + fixtures only; the corrected
`migrate --all` is fired live by the arc-close freeze, not here. The capability/live rhythm (s12→s13) holds
once more.

## 4. Findings — CDC disposition

**CDC-F14-1 (elevated to a BLOCKING freeze precondition).** With the append now in code, the arc-close
freeze **must not run until `.worktrees/odm/config.toml`'s `docs_directory` is changed from `"./docs/design"`
to the parent `"./docs"`.** Reproduced the failure: today the store config has `docs_directory = "./docs/design"`
(top-level), so `configured_design_directory` → `./docs/design/design`, which does not exist — and worse,
`docs_root` (the umbrella) → `./docs/design`, so `discover_plan_roots` would look for the plan tree under
`./docs/design` and **miss `./docs/design-v1.0.0` entirely**, self-hosting nothing. CC disclosed this
clearly (`closing-report.md` §F-2/F-6, and the bubble-up). I am elevating it from a note to a **hard gate**:
it is the first step of the freeze, and the freeze's dry-run adjudication is now **doubly load-bearing** —
s14 changed root resolution *and* the config is being edited, so the dry-run is the authority on the
effective root set (plan tree self-hosted, design over `./docs/design`, dev notes, artifacts), not an
assumption of "just the two living-tail reconciles." Any create beyond the CDC-F1 living-tail nodes → stop
and adjudicate. *(dev_directory is already correct as-is at `"./docs/dev"`; `additional_paths` is unset,
fine — nothing to move yet.)*

**CDC-F14-2 (accepted; non-blocking).** The D-2 plan-set escape hatch self-hosts a Plan-shaped additional
dir, but cannot host a *second independent project* — a store is single-project by the store-wide
project-number constraint, not an s14 defect. Correctly disclosed, correctly *not* worked around (fixing it
would be an `odm-migrate` model change well outside this slice). It does not affect the freeze (which passes
no additional paths). Accepted as a documented boundary of the feature.

## 5. Observations

1. **`toml_edit` for the write-back is the right call** — a format-preserving edit that keeps the store
   config's comments and section style intact, and builds an explicit (non-inline) `[legacy]` table so a
   first write reads like a hand-authored one. Good instinct beyond the letter of the ask.
2. **CC checked its fix against my §7 runbook** — confirming the arc-close freeze commands parse only under
   the new syntax. That's the right reflex: the runbook is a downstream consumer of this CLI, and it now
   agrees with the shipped surface.
3. **Persist-then-subtract** (remember every named path, exclude only from *processing*) is the subtle-right
   ordering — a forgotten re-pass still covers a path that a given run happened to dedup away.

No findings beyond CDC-F14-1 (routed to the freeze) and CDC-F14-2 (accepted).

## 6. Bubble-up check (PM Part IV)

- **Did s14 deliver?** Yes — migrate is config-driven, the append restored, additional-paths persistent and
  deduped, the config-home bug fixed and modeled (ODD-0022 v1.1), all fixture-proven; `odm@e06fffe`
  untouched.
- **Silent-drop diff:** none. Both findings are named in the closing report; CDC-F14-1 is elevated here to
  a blocking gate so it can't be lost at freeze time.
- **Arc-plan change:** flip s14 → **CDC-verified PASS** (v2.28). **The arc-close resumes**, with the freeze
  runbook now carrying a **step 0**: update `.worktrees/odm/config.toml` `docs_directory` → `"./docs"`, then
  the dry-run adjudication (doubly load-bearing), then fire, then the P-12 demo, then Migration Fidelity
  closes.

## Closure

s14 **CDC-verified PASS** on 2026-07-31. Config-driven migrate reproduced with restored append, persistent
deduped additional-paths, and a properly-modeled config-home fix; every ledger row carries a non-vacuous
test; no live mutation (`odm@e06fffe`). One blocking precondition elevated for the freeze (CDC-F14-1:
store-config `docs_directory` → `"./docs"` first), one boundary accepted (CDC-F14-2). The arc-close resumes:
config fix → freeze (dry-run-adjudicated) → P-12 → Migration Fidelity closes.

_Verified by: CDC (independent), 2026-07-31 — against `release/1.0.x` (`45ca74c`…`32f60e2`); `odm@e06fffe`
unchanged. Code/fixtures/ODD reproduced by direct read; execution rows attested-by-CC → CI._
