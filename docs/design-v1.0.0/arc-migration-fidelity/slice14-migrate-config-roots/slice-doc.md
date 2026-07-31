# Slice 14 (Migration Fidelity) — Config-driven migrate roots + additional-paths (plan-of-record)

> Refs: `../arc-plan.md` (s14 inserted before the arc-close freeze) · **s13** (the `migrate --all`
> composition this corrects) · **ODD-0022 §4.2** (the config split: locator `odm.toml` vs the store's
> operational `config.toml`) · **ODD-0025** §2.5/§2.6 (artifact/note derivations) · `crates/odm-cli/src/{lib.rs,migrate.rs,commands.rs}`,
> `crates/odm-migrate/src/{legacy.rs,notes.rs,artifacts.rs}`. `depends_on:` s13.
>
> **Capability slice — fixture-proven; no live `odm`-branch mutation.** The corrected `migrate` is then
> fired live by the **arc-close final reconcile-and-freeze** (the arc's capability/live rhythm: s12→s13,
> here s14→the freeze). **s14 blocks that freeze** — the freeze must run the *corrected* command, not the
> current one.

## Goal

Make `odm migrate` **config-driven** rather than positional-path-driven, restore the legacy `docs_directory`
+ `"design"` semantic the current `--all` dropped, and give the operator a first-class way to sweep
arbitrary extra legacy directories that persists across runs. **Done when** `migrate` takes no
`<LEGACY_PATH>`; an optional comma-separated `[ADDITIONAL_PATHS]` positional feeds arbitrary dirs; every
root (the design/research corpus, the dev corpus, the additionals) is resolved from the `[legacy]` config
block the code actually reads; the design/research corpus is `docs_directory` **+ `/design`** (computed,
not the raw `docs_directory`); additional paths are unioned into `[legacy].additional_paths` (sorted +
uniqued, written back); overlapping paths are set-subtracted so nothing is processed twice; and the whole
thing is idempotent and `--dry-run`-safe — all fixture-proven, `.worktrees/odm` untouched.

## Why

Three defects surfaced by the operator against the shipped `migrate --all`:

1. **Wrong positional contract.** `migrate` requires `<LEGACY_PATH>` = "the corpus," but `--all` *also*
   reads `docs_directory`/`dev_directory` from config — two roots, ambiguously related. The tool should be
   run from where `odm.toml`/the store `config.toml` lives and resolve every root from config; the
   positional should carry only *additional* dirs, not the corpus.
2. **The `docs_directory` "design" append was dropped.** Legacy odm (v0.3.5−) computed the primary indexed
   corpus as `docs_directory` **+ hard-coded `"design"`**. The current `all()` uses
   `configured_docs_directory` **as-is** (`crates/odm-cli/src/migrate.rs::all` → `commands::configured_docs_directory`,
   no join). It "works" only because the store `config.toml` currently sets `docs_directory = "./docs/design"`
   directly. The operator is moving config to the canonical `[legacy] docs_directory = "./docs"` (the
   parent) — at which point the as-is code reconciles design/research over **all of `./docs`** (plan tree +
   dev + design), applying the frontmatter design/research rules to files that must not get them. The
   append must be restored so the design rules apply to **`docs_directory/design` only**.
3. **No persistent additional-paths.** Un-migrated legacy projects have dirs beyond `docs_directory`/`dev_directory`
   (research, brainstorm-sessions, chats). There is no way to sweep them, and — since they'll be deleted
   only *after* migration is proven — no way to remember them for the re-runs/checks in between. They need
   to be captured in config so a forgotten re-pass still covers them.

**Config-location subtlety (must resolve, F-6).** `configured_directory` reads
`StoreHome::resolve(root).operational_text()` — **the store's `config.toml`**, whose `docs_directory` is
still the top-level `"./docs/design"`. The `[legacy]` block the operator added to the *code-branch*
`odm.toml` is **not the file the code reads**. So the `[legacy]` home for `docs_directory`/`dev_directory`/
`additional_paths` must be the operational config the code resolves (the store `config.toml`), consistent
with the ODD-0022 §4.2 split (operational half lives in the store). The slice must land the keys where they
are read, and any v1.0+ config creation/migration must write `[legacy]` there — else re-runs lose
idempotency.

## Scope

**In (capability; `release/1.0.x` code + fixtures; `.worktrees/odm` untouched):**

- **CLI surface** (`lib.rs`, `migrate.rs`). Drop the required `legacy_path: String`. Add an optional
  comma-separated positional `additional_paths: Option<String>` (parsed → `Vec<PathBuf>`, trimmed, empties
  dropped). Every mode (`--all`, `--coverage`, `--artifacts`, `--notes`, `--vision`, the default plan /
  design-research dispatch) resolves its roots from config, not the removed positional. Update the clap
  `about`/doc-comments and the `--help` usage. Ideal shape: `odm migrate --all research,brainstorm,chats`.
- **Design-corpus append** (F-2). The design/research reconcile root is `docs_directory` **joined with
  `design`** — a single, documented computation, replacing the raw `configured_docs_directory`. The
  frontmatter design/research rules (NN-state dirs, §2.4 field mapping) apply to *that* dir only.
- **Root map, resolved from config** (the model, F-1/F-2/F-3):
  - **design/research corpus** = `docs_directory` + `/design` (computed) — the custom-rules corpus.
  - **dev corpus** = `dev_directory` (as-is, config-only) — `--notes`.
  - **plan-set self-host + artifacts sweep root** = **`docs_directory` (the parent)** — `discover_plan_roots`
    + `--artifacts` run over it, exactly as today but sourced from config instead of the positional.
    *(Decision D-1, flag: this reuses `docs_directory` as the umbrella. Alternative — a dedicated
    `plan_directory`/umbrella key. Recommended: parent = `docs_directory`, since the project layout is
    `docs/{design,design-v1.0.0,dev}` and `docs_directory=./docs` already names that umbrella. CC: adopt
    the recommendation unless it doesn't hold up, then flag.)*
  - **additional corpus** = the positional `[ADDITIONAL_PATHS]` ∪ `[legacy].additional_paths` (config),
    each resolved relative to `root`.
- **`additional_paths` persistence** (F-5). New positional paths are **unioned** into
  `[legacy].additional_paths`, the list **sorted + de-duplicated** before write-back, and re-read on
  every run so a forgotten re-pass still covers them. `--dry-run` writes no config.
- **Additional-path processing** (F-4). Each effective additional dir is migrated as arbitrary docs.
  *(Decision D-2, flag: recommended = the `--artifacts` generic derivation (supporting-doc `artifact`
  nodes, ODD-0025 §2.6), since these are un-typed legacy dirs; corpus auto-detect is the alternative if a
  dir is a plan set / state-dir corpus. CC: recommend artifacts-with-autodetect-escape-hatch; flag if the
  fixtures argue otherwise.)*
- **Set operations / dedup** (F-7). Compute effective additionals =
  `(positional ∪ config.additional_paths)` **minus any path that equals, contains, or is contained by**
  `design_root` (`docs_directory/design`) or `dev_root` (`dev_directory`) — and, if D-1 keeps the parent
  umbrella, minus the plan-set roots too. A dir already handled by the design or dev pass is **not**
  re-processed by the generic additional pass (which would apply the wrong, custom-rule-less derivation).
  Overlap in either direction (an additional that *is* `docs_directory`, or that *contains* it) is handled,
  not doubled.
- **Config `[legacy]` home + idempotency** (F-6). `docs_directory`/`dev_directory`/`additional_paths`
  resolve from the operational config the code reads (store `config.toml`), under `[legacy]` (top-level
  still honored via the existing fallback for back-compat). Document that a v1.0+ config must carry
  `[legacy]` for re-run idempotency; if the slice ports the real project's keys under `[legacy]`, that is a
  config edit on `release/1.0.x`/the store config, **not** an `odm`-branch data mutation.

**Out:**

- **The live re-run** — the arc-close **final reconcile-and-freeze** fires the corrected `migrate --all`
  on the live `.worktrees/odm` store (closes CDC-F1's living-plan tail *and* re-migrates over the corrected
  roots). Not here.
- **The P-12 self-host demo** and the rest of the arc-close.
- **L-8 / CDC-ARC-1** (the 14 RH-era design nodes' `version` back-fill + a standing frontmatter-fidelity
  check) — post-close; a corrected `migrate` is a prerequisite for it but not this slice's work.
- Deleting any legacy dir (that happens only after migration is proven; `additional_paths` exists precisely
  to bridge that interval).

## Verification

Fixture, class-(a) — code + `TempDir` fixtures; runtime execution attested→CI. After the change:

- **CLI:** `migrate` with no positional resolves all roots from config; `migrate --all a,b,c` parses three
  additionals; `--help` shows the new usage (no `<LEGACY_PATH>`, optional `[ADDITIONAL_PATHS]`).
- **Append:** a fixture with `docs_directory=<tmp>/docs` and `<tmp>/docs/design/NN-state/*.md` reconciles
  design/research over `<tmp>/docs/design` **only** — files directly under `<tmp>/docs` (e.g. a plan tree
  or a dev doc) do **not** receive the design/research derivation.
- **Additionals + persistence:** passing `x,y` migrates `x` and `y`, writes `[legacy].additional_paths =
  ["x","y"]` sorted+uniqued; a second run with no positional still covers `x,y` from config; passing `y,z`
  yields `["x","y","z"]` (union, sorted, uniqued — no dup `y`).
- **Dedup:** an additional equal to (or nested under) `docs_directory/design` or `dev_directory` is
  excluded from the generic pass and processed once, by its proper derivation.
- **Idempotent / dry-run:** a second identical run is a 0-change no-op on every step including the config
  write-back; `--dry-run` writes neither nodes nor config.

Runtime rows (`cargo`/`clippy`/`llvm-cov`, and any live `migrate` exec) attested-by-CC → CI; CDC reproduces
the code + fixtures + the config semantics by direct read.

## Rollback & findings discipline

Fixture-only — no snapshot/revert gate (no live mutation). Amend-don't-work-around: if the legacy-config
model or `additional_paths` needs a normative line, amend the cited ODD (0022 §4.2 for the config split;
0025 if the additional-corpus derivation needs a model note). Flag D-1/D-2 explicitly rather than silently
choosing. Five-iteration cap.

## Exit

`ledger.md` closed; CDC-verified against the code + fixtures. `migrate` is config-driven, the design append
restored, additional-paths first-class and persistent, dedup correct, idempotent. On close, bubble up to
`../arc-plan.md`: s14 done (migrate corrected); **the arc-close resumes** — the final reconcile-and-freeze
now fires the *corrected* `migrate --all` live, then the P-12 demo, then Migration Fidelity closes.
