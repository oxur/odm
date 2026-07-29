---
id: 01KYNDTQ6SYP3MYNPHCTNEA9PM
number: 7773601
type: slice
schema: slice/v1.1
name: Slice 01 — Store resolution + two-config split (slice-doc / plan-of-record)
created: 2026-07-28
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-store-home/slice01-store-resolution/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KYNDTQ6SHYYMCPX4JN86ZJ6F
status:
  built:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
  planned:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
  tested:
    reached: 2026-07-28
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-28
---
# Slice 01 — Store resolution + two-config split (slice-doc / plan-of-record)

> **Arc:** Store Home & `init` (`arc-store-home`) · **Ledger:** SH-1 · **Realizes:** ODD-0022 §4.2.
> **Foundational** — every other slice (02–04) and RH C-5's cutover are load-bearing on this.

## Goal

Make the odm store **locatable and configurable** via the two-config model, so the store root can
resolve to a worktree instead of the repo root — **with no git-worktree creation** (that is slice
02). After this slice: given a `[store]` locator plus an existing store dir containing `config.toml`,
all commands read/write `nodes/` under the configured store; with **no** `[store]`, behaviour is
unchanged (`<repo-root>/nodes/`).

## Scope

**In:**
- **`[store]` section in `odm.toml`** — `worktree_base`, `worktree_name`, `branch_name`, all
  optional, documented defaults `.worktrees` / `odm` / `odm`.
- **Store-root resolution** — `[store]` present ⇒ root = `<repo>/<worktree_base>/<worktree_name>`;
  absent ⇒ repo root (current behaviour, back-compat).
- **Two-stage config load** — (1) *locator*: read `[store]` from `odm.toml` via the existing
  cwd → repo-root → user search; (2) *operational*: load **`config.toml`** from the resolved store
  root (gate-sets, display, `docs_directory`, author). If `config.toml` is absent, fall back to
  reading the operational config from `odm.toml` (so existing repos keep working).
- **All node reads/writes** (`odm-store` layout) use the **resolved** store root.

**Out (later slices / not this slice):**
- Creating the worktree, orphan branch, or gitignore — **slice 02** (`init` bootstrap).
- `init` / attach / ff-sync / `rename` — slices 02–04.
- Migrating odm's own corpus onto the orphan branch — **RH C-5's cutover** (SH-6).
- **No `git` subprocess / shell-out** anywhere in this slice (that arrives in slice 02).

## Verification approach

- **Unit:** resolution (`[store]` present → the worktree path; absent → repo root); config
  precedence (`config.toml` over `odm.toml` operational) and the fallback when `config.toml` is
  absent.
- **Integration:** hand-place `.worktrees/odm/` with a `config.toml` + `nodes/`, set `[store]` in
  `odm.toml`; `odm list` / `odm check` operate on that store. Remove `[store]` → they operate on
  `<repo-root>/nodes/`, unchanged.
- `odm check` green in both modes; **zero behaviour change** for repos without `[store]`.

## Exit criteria

- `[store]` resolves the store root; the no-`[store]` fallback is intact.
- Two-stage load works; `config.toml` operational settings are honoured; the `odm.toml` fallback
  works for repos that have no `config.toml` yet.
- No git-worktree ops / no `git` subprocess introduced (grep-clean).
- `cargo build` / `test` / `clippy --all-targets -- -D warnings` / `fmt` green; no `unsafe`.
- All SH-1 `ledger.md` rows reach a final status.
