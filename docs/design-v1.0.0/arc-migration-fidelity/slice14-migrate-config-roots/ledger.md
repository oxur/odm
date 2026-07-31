# Slice 14 (Migration Fidelity): Config-driven migrate roots + additional-paths

> Per LEDGER-DISCIPLINE v2.0 (slice scale, §A). Evidence carries a **strength**
> (`asserted < attested < reproduced < reconciled`); a `done` row reaches ≥ `reproduced`. **Capability
> slice — fixture-only; `.worktrees/odm` untouched.** Code/fixtures are class-(a): CDC reproduces by
> direct read; runtime execution (`cargo`/`clippy`/`llvm-cov`) attested→CI. The corrected `migrate` is
> fired live by the **arc-close freeze**, not here. Five-iteration cap.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | **Positional redesign**: `<LEGACY_PATH>` dropped; optional comma-separated `[ADDITIONAL_PATHS]` added; every mode resolves roots from config; `--help` updated | `migrate` with no positional runs; `migrate --all a,b,c` parses 3 additionals (trimmed, empties dropped); `--help` shows no `<LEGACY_PATH>`, optional `[ADDITIONAL_PATHS]` | serious (the contract) | operator | open | | clap `about`/doc-comments + dispatch in `lib.rs`/`migrate.rs` updated together. |
| F-2 | **`docs_directory` + `"design"` append restored**: design/research reconcile runs over `docs_directory/design` (computed), not raw `docs_directory` | fixture: `docs_directory=<t>/docs`, `<t>/docs/design/NN-state/*.md` → reconcile touches `<t>/docs/design` only; a file directly under `<t>/docs` gets **no** design/research derivation | serious (the core bug) | v0.3.5 semantic / operator | open | | Replaces `all()`'s as-is `configured_docs_directory` use. The NN-state + §2.4 rules apply to the computed dir only. |
| F-3 | **Root map from config**: design=`docs_directory/design`; dev=`dev_directory` (as-is); plan-set self-host + `--artifacts` sweep root resolved from config (D-1) — no positional anywhere | read each mode's root resolution; confirm none reads a corpus positional | serious | slice-doc | open | | **D-1 (flag):** recommended umbrella = `docs_directory` (parent); alt = a dedicated key. Adopt unless it breaks; flag. |
| F-4 | **Additional-path processing**: each effective additional dir migrated as arbitrary docs | fixture: an additional dir of loose `.md` → covered as `artifact` nodes (D-2) | serious | operator | open | | **D-2 (flag):** recommended = `--artifacts` generic derivation + a corpus-autodetect escape hatch; flag if fixtures argue otherwise. |
| F-5 | **`additional_paths` persistence**: new positional paths unioned into `[legacy].additional_paths`, **sorted + uniqued**, written back; re-read every run | fixture: pass `x,y` → config `["x","y"]`; run 2 (no positional) still covers x,y; pass `y,z` → `["x","y","z"]` (no dup y) | serious (forgot-to-re-pass guard) | operator | open | | `--dry-run` writes no config (F-8). Write-back is deterministic (sort+dedup → a re-pass of the same set rewrites nothing). |
| F-6 | **`[legacy]` config home + idempotency**: `docs_directory`/`dev_directory`/`additional_paths` resolve from the operational config the code reads (store `config.toml`), under `[legacy]` (top-level still honored); v1.0+ config carries `[legacy]` for re-run idempotency | direct read: the keys resolve from `StoreHome::resolve(root).operational_text()`; the operator's `[legacy]` block lands where `configured_directory` reads it | serious (idempotency) | ODD-0022 §4.2 | open | | The code-branch `odm.toml` `[legacy]` the operator added is **not** the file the code reads — resolve so the intended config takes effect. |
| F-7 | **Set-subtraction dedup**: effective additionals = `(positional ∪ config)` − {equals/contains/contained-by `design_root` or `dev_root` (and plan roots if D-1)}; nothing processed twice | fixture: an additional == or nested-under `docs_directory/design` (or `dev_directory`) is excluded from the generic pass, processed once by its proper derivation; overlap both directions handled | serious (no custom-rule-less re-processing) | operator | open | | The failure this guards: a design/dev dir swept a second time by the generic (artifact) pass, dropping the frontmatter/notes rules. |
| F-8 | **Idempotent + dry-run-safe**: a second identical run = 0-change no-op on every step incl. the config write-back; `--dry-run` writes neither nodes nor config | run twice: second run 0 created / 0 reconciled / 0 config bytes changed; `--dry-run` leaves store + config byte-identical | serious | s13 protocol | open | | |
| F-9 | **No model drift / amend-not-work-around**: the design/dev/artifact/note derivations unchanged except their *root resolution*; any normative line amended in the cited ODD, not worked around | cross-read: derivations intact; ODD amended if `additional_paths`/append needs a model line | correctness | ODD-0022/0025 | open | | The append is a *restored legacy semantic*, not a new rule; `additional_paths` is a config affordance. |
| F-10 | **Clippy clean; no `unsafe`; coverage ≥ 90% on touched code** | clippy `-D warnings` exit 0; `! grep unsafe`; `llvm-cov` ≥ 90% on changed (CLI wiring + root resolution + set ops + config write-back) | polish/correctness | CLAUDE.md | open | | Set-ops + config write-back are the substantive new logic to cover. |

## What Worked

_(At slice close.)_

## Closure

_(At slice close.)_ Fixture-only — no store commit. Verified by: `<CC then CDC>`. Rows: 10. Done: `<n>`.
Deferred: `<n>`. On close, bubble up to `../arc-plan.md`: s14 done (migrate config-driven + append restored
+ additional-paths persistent + dedup); **the arc-close resumes** — the final reconcile-and-freeze fires the
corrected `migrate --all` live, then the P-12 demo, then Migration Fidelity closes.
