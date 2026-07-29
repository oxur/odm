---
id: 01KYNM102YEZS6SV6D8HRDQT76
number: 58837408
type: slice
schema: slice/v1.1
name: Slice 08 (Migration Fidelity) — Source-path portability (plan-of-record)
created: 2026-07-29
updated: 2026-07-29
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-migration-fidelity/slice08-source-path-portability/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SVSRZMRH0E07YJMEP
---
# Slice 08 (Migration Fidelity) — Source-path portability (plan-of-record)

> Refs: `../arc-plan.md` v2.8 (CDC Finding 1) / v2.9 (this slice + the tightened spec); ODD-0025
> §2.0/§2.2 (`source` as the stored identity axis); s05 (`source.paths` as the `by_source` key); s07
> `slice07-live-run/cdc-verification.md` (the finding, with the demonstration that absolute paths don't
> resolve off the authoring machine). `depends_on:` s07 (the live corpus this rewrites).
>
> **Capability + live corrective re-migration, unbundled from feature work** (the s06/s07 shape). It
> makes odm's committed self-hosted store **portable** — the precondition for coverage-in-`check`
> working anywhere but the authoring machine (s09).

## Goal

Make `source.paths` **repo-content-root-relative and canonical** so the s05 identity key is stable
across machines, checkouts, and worktrees — then **rewrite the 61 committed nodes** to that form. **Done
when** every node's `source.paths` is `docs/…`-relative and canonical, two checkouts at different
absolute roots produce **identical** paths / `by_source` matches / coverage, the live store is rewritten
absolute→relative with **no body/id/schema change and no re-mint**, `check` is green, and a re-run **from
a different checkout path** is idempotent — all as one revertible commit.

## Why (the property, stated precisely)

Two collaborators ingesting the same files must get identical results — that determinism is what makes
odm's migration *repeatable*, and absolute paths (`/Users/oubiwann/…`, baked into all 61 nodes at s07)
cannot satisfy it: they key identity to one machine's filesystem, so a re-run from a different checkout
would **re-mint duplicates** (the `by_source` miss → treated as new — the exact s05 fragility, via the
path instead of the number), and they don't even resolve inside a different mount. Relative-to-content-root
is not a preference here; it is forced by the repeatability guarantee.

## Scope

**In:**

- **Store `source.paths` relative to the repo *content* root**, yielding `docs/design-v1.0.0/…` — the
  path a plain checkout of the branch shows. **Not** the checkout/worktree/superproject root: anchoring
  there would bake in `.worktrees/1.0.x/`, its own portability bug. The anchor is the **git toplevel of
  the plan tree being migrated** (the docs worktree), so the same doc yields `docs/…` whether it's
  reached via `.worktrees/1.0.x/` or a normal `release/1.0.x` clone. Touch the seam where paths become
  storable: `migrate::resolve()` (currently `root.join(arg)` → absolute, `migrate.rs:330`),
  `selfhost::body_source_path` / `discover` (the `plan_root.join(...)` that seeds `node.source`), and
  `fidelity::build_source` (which stores `paths` verbatim).
- **Canonicalize**: forward slashes, no `./`/`..`, no trailing slash; and **decide the case rule** so a
  path stored on case-insensitive macOS still matches on case-sensitive Linux CI (store as-on-disk and
  compare exactly, or case-fold — pick one, justify it, test it). The stored form must be **identical
  regardless of how the plan-root arg was spelled** (trailing slash, `./` prefix, absolute vs relative
  arg).
- **One shared anchor for write *and* resolve.** The same anchor relativizes a path for storage/keys
  and resolves a stored relative path back to absolute for the filesystem read in `reconcile_source`
  (`fs::read_to_string`). Write and read use one function, so they cannot drift.
- **Transition-safe `by_source` matching (the re-mint guard).** The 61 existing nodes hold **absolute**
  paths today. The `by_source` key must be the **canonical-relative form derived from whatever is
  stored** — canonicalizing an absolute stored path to relative for the key — *and* from the freshly
  discovered path, so during the absolute→relative rewrite an absolute-stored node and a
  relative-discovered lookup collapse to the **same key** → matched, **updated in place, never
  re-minted**. This is the s05 `to_populate` transition pattern applied to the path *form*.
- **Live corrective re-migration** (behind the s07 protocol): snapshot the `odm` store + capture the
  known-good SHA → `--dry-run` (expect **61 path rewrites, 0 creates, 0 body/id/schema changes**;
  adjudicate before firing; store byte-identical after the dry-run) → fire as **one revertible commit**.

**Out:**

- `artifact`-node minting + wiring doc-coverage into `check` + the `coverage.rs` report fixes (Findings
  2–3) + the 14 design/research nodes' `source` — all **s09** (coverage enforcement), which this slice
  unblocks by making paths portable.
- The project-node synthesis re-cast (s10); the arc-close reconcile demo (s11).
- Any change to what `source.paths` *means* (still the stored identity axis) or to the body-hash gate
  (unchanged) — only the *form* of the stored path changes. If ODD-0025 needs a line on path form, amend
  it; don't work around.

## Verification

The rewritten store is the evidence (live, class-(b)). After the committed run:

- Every node's `source.paths` entries are **`docs/…`-relative and canonical** (no leading `/`, no
  `.worktrees/`, forward slashes).
- **Cross-checkout determinism (the point):** fixture-prove two stores built from the *same* docs at
  *different* absolute roots have byte-identical `source.paths` and identical `by_source` outcomes; and
  a live **re-run from a different checkout path mints 0 / reconciles 0** (idempotent across roots — the
  property absolute broke).
- **No collateral change:** 0 stubs still, 61 source-bearing still, project + retired still excluded,
  bodies/ids/schema **unchanged** (only `source.paths` string form differs), `check` green,
  `orient`/`rollup` byte-stable.
- Coverage set-difference = 0 uncovered arcs/slices, computed identically from either checkout root.

CDC reproduces by direct read (path form, cross-root determinism by remapping, body/id invariance);
`check`/`orient`/`rollup` attested → CI.

## Rollback & findings discipline

Same spine as s07. Any gate failure — a re-mint in the dry-run (the transition guard failed), a body/id
change, `check` red, a non-idempotent cross-root re-run — **reverts to the captured SHA and becomes a
finding**, not forced. The re-mint case especially: if the dry-run shows any create, **stop** — the
transition-safe matching is wrong and must be fixed before any live write.

## Exit

`ledger.md` closed; CDC-verified against the committed store. `source.paths` is portable, the identity
key is stable across machines, and coverage can now be enforced in CI (s09). On close, bubble up to
`../arc-plan.md`: Finding 1 resolved; s09 (coverage enforcement) is unblocked and next.
