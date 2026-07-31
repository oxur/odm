# CC Prompt — Slice 14 (Migration Fidelity): Config-driven migrate roots + additional-paths

Make `odm migrate` **config-driven** instead of positional-path-driven, restore the legacy
`docs_directory` + `"design"` append the current `--all` dropped, and add a first-class, **persistent**
additional-paths sweep. **All in code, fixture-proven, `.worktrees/odm` untouched** — the live re-run is
the arc-close freeze, which will fire *this corrected command*, so **s14 blocks that freeze.**

> **Start condition:** on `release/1.0.x`, green, s13 merged. **Fixture only — no `odm`-branch commit,
> fire nothing on the live store.** This is a capability fix to the CLI + config resolution; the operator
> runs the corrected `migrate --all` live at the arc-close.

## The three defects you are fixing (operator-surfaced)

1. **Wrong positional.** `migrate` requires `<LEGACY_PATH>` = the corpus, but `--all` also reads config
   roots — two roots, ambiguous. Run-from-config instead; the positional carries only *additional* dirs.
2. **Dropped `"design"` append.** `crates/odm-cli/src/migrate.rs::all` uses `commands::configured_docs_directory`
   **as-is** (no join). Legacy odm computed the design corpus as `docs_directory` **+ `"design"`**. It only
   works today because the store config sets `docs_directory = "./docs/design"` directly; the operator is
   moving to the canonical `[legacy] docs_directory = "./docs"`, at which point as-is reconciles over **all
   of `./docs`**. Restore the append.
3. **No persistent additional-paths.** Legacy projects have extra dirs (research, brainstorm, chats) that
   must be swept and *remembered* between runs until they're deleted post-migration.

## Read first

1. `slice14-migrate-config-roots/ledger.md` (10 rows) — the spec of "done". `slice-doc.md` — esp. the root
   map, **D-1/D-2** (two flagged decisions — adopt the recommendation or flag), and the **F-6 config-home
   subtlety**.
2. **The code you change:**
   - `crates/odm-cli/src/lib.rs` — the `Migrate {}` clap struct (drop `legacy_path: String`; add
     `additional_paths: Option<String>`) + the `Command::Migrate` dispatch.
   - `crates/odm-cli/src/migrate.rs` — `migrate()`, `all()`, `discover_plan_roots()`,
     `reconcile_design_research()`; where `legacy_path`/`docs_root` flow today.
   - `crates/odm-cli/src/commands.rs` — `configured_directory` / `configured_docs_directory` /
     `configured_dev_directory` (add the `/design` join for the design root; add
     `configured_additional_paths` + the sorted-uniqued write-back).
3. **ODD-0022 §4.2** (config split: locator `odm.toml` vs the store's operational `config.toml` — the
   `[legacy]` keys belong in the operational half the code reads via `StoreHome::resolve(root).operational_text()`).
   **ODD-0025** §2.5/§2.6 (the artifact/note derivations the additionals reuse).

## Load skills (via `/<name>`)

- `/rust-guidelines` — anti-patterns first, then `03-error-handling`, `05-type-design` (the set-ops + the
  config write-back are the substantive new logic).
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task

1. **CLI surface** (F-1). Drop `legacy_path`; add optional `additional_paths: Option<String>`, parsed to a
   trimmed, empties-dropped `Vec<PathBuf>`. Rewire every mode to resolve roots from config. Update the
   `about`/doc-comments so `--help` shows no `<LEGACY_PATH>` and an optional `[ADDITIONAL_PATHS]`. Target
   shape: `odm migrate --all research,brainstorm,chats`.
2. **Design append** (F-2). Introduce one documented computation `design_root = docs_directory.join("design")`
   and use it for the design/research reconcile. The NN-state + §2.4 frontmatter rules apply to
   `design_root` **only**.
3. **Root map from config** (F-3, D-1). design=`docs_directory/design`; dev=`dev_directory` (as-is);
   plan-set self-host + `--artifacts` sweep root = **`docs_directory` (parent)** unless you find that
   doesn't hold — then flag. No mode reads a corpus positional.
4. **Additional paths** (F-4/F-5, D-2). Migrate each effective additional dir (recommended: the
   `--artifacts` generic derivation + a corpus-autodetect escape hatch). **Union** the passed positional
   paths into `[legacy].additional_paths`, **sort + dedup**, write back to the operational config;
   re-read every run. `--dry-run` writes no config.
5. **Set-subtraction dedup** (F-7). effective additionals = `(positional ∪ config.additional_paths)` minus
   any path that equals / contains / is contained by `design_root` or `dev_root` (and the plan roots if
   D-1 keeps the parent umbrella). A design/dev dir must never be re-processed by the generic pass. Handle
   overlap in both directions.
6. **Config `[legacy]` home** (F-6). Ensure `docs_directory`/`dev_directory`/`additional_paths` resolve
   from the config the code actually reads (store `config.toml`), under `[legacy]` (keep the existing
   top-level fallback). If you port the real project's keys under `[legacy]`, that's a `release/1.0.x` /
   store-config edit — **not** an `odm`-branch data mutation. Document that a v1.0+ config must carry
   `[legacy]` for re-run idempotency.
7. **Fixtures** (F-1…F-8): CLI parse (no positional; `a,b,c`); the append (design-only derivation);
   additionals + the sorted-uniqued union write-back across two runs; the dedup (design/dev overlap
   excluded); idempotence + dry-run-writes-nothing (nodes *and* config).

## Constraints (flag, don't silently change)

- **No live store mutation. No `odm`-branch commit.** The live re-run is the arc-close freeze.
- **The design/dev/artifact/note derivations themselves don't change** — only their *root resolution* and
  the new additional/dedup/persistence logic. Reconcile still only sets a body from its current source.
- **Amend, don't work around.** The append is a *restored legacy semantic*; `additional_paths` is a config
  affordance — if either needs a normative line, amend the cited ODD (0022 §4.2 / 0025).
- **Flag D-1 (umbrella root) and D-2 (additional-path derivation)** with your chosen resolution + why.
- No `unsafe`; typed errors; coverage ≥ 90% on touched code; clippy `-D warnings` clean.

## Deliverables

The code change on `release/1.0.x` (CLI redesign + append + additional-paths + dedup + config write-back +
the `[legacy]` home) and any ODD amendment; `ledger.md` evidence per row (`attested` → CI — cite test
names, exits, the `--help` output); `closing-report.md` — per-row walk, the D-1/D-2 decisions + rationale,
the config-home resolution, any ODD amendment, **plus the v2.0 Bubble-up** (did s14 make migrate
config-driven + restore the append + land persistent additionals; the silent-drop diff vs In/Out; confirm
the arc-close freeze fires *this* corrected command next). Branch: `release/1.0.x` only.

## Working agreement

Amend don't work around; flag every deviation; five-iteration cap. Your `done` is proposed-done — CDC
reproduces the code + fixtures + the config semantics + CI. On close, bubble up to `../arc-plan.md`: s14
done; **the arc-close resumes** (the final reconcile-and-freeze fires the corrected `migrate --all` live →
the P-12 demo → Migration Fidelity closes).
