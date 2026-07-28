# CC Prompt — Slice 08 (Migration Fidelity): Source-path portability

Make `source.paths` **repo-content-root-relative and canonical** (a portable identity key), then
**rewrite the 61 committed nodes** to that form via a live corrective re-migration. This resolves CDC
v2.8 Finding 1 — absolute `/Users/oubiwann/…` paths, baked into the live store at s07, key identity to
one machine and would re-mint duplicates on any other checkout. Fixture-prove the capability first;
then the live rewrite behind the s07 snapshot → dry-run → fire protocol.

> **Start condition:** on `release/1.0.x` (green). **This slice mutates the live `odm` store** — but
> only as one revertible commit, after a clean dry-run. **Snapshot/revert + dry-run-first are HARD
> gates.** The dry-run must show **61 path rewrites, 0 creates** — *any* create means the transition
> guard (Task 4) is wrong; **stop and flag CDC, do not fire.** **Why:** two collaborators ingesting the
> same files must get identical results — that repeatability is odm's whole claim, and absolute paths
> can't satisfy it. Relative-to-content-root is forced, not preferred.

## Read first
1. `slice08-source-path-portability/ledger.md` (10 rows) — the spec of "done."
2. `slice-doc.md` (esp. **Why** + **Transition-safe matching**); `slice07-live-run/cdc-verification.md`
   **§ Finding 1** (the demonstration that absolute paths don't resolve off-machine).
3. **ODD-0025** §2.0/§2.2 (`source` as the stored identity axis); s05's `by_source` key.
4. **The code you change:**
   - `crates/odm-cli/src/migrate.rs::resolve()` (line ~330) — `root.join(arg)` → absolute today.
   - `crates/odm-migrate/src/selfhost.rs` — `body_source_path` / `discover` (the `plan_root.join(...)`
     that seeds `node.source`), the `by_source` index build + lookup, `reconcile_source`'s
     `fs::read_to_string` (the *resolve* side).
   - `crates/odm-migrate/src/fidelity.rs::build_source` — stores `paths` verbatim (relativize before, or
     here).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `05-type-design.md`, `03-error-handling.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE.

## Task
1. **Relativize to the repo *content* root** (F-1). Store `source.paths` as `docs/design-v1.0.0/…` —
   the path a plain checkout shows. The anchor is the **git toplevel of the plan tree being migrated**
   (the docs worktree), **not** the superproject/checkout root — anchoring there bakes in
   `.worktrees/1.0.x/`, its own portability bug. Introduce one anchor-derivation + one relativize
   helper; use them at every seam where a path becomes storable.
2. **Canonicalize + decide the case rule** (F-2). Forward slashes, no `./`/`..`, no trailing slash; the
   stored form must be **identical regardless of how the plan-root arg was spelled** (trailing slash,
   `./`, absolute vs relative arg). Decide macOS↔Linux-CI case handling (store-as-disk-compare-exact
   **or** case-fold) — **flag your choice and test it**; don't leave it implicit.
3. **One shared anchor, write *and* resolve** (F-3). The same anchor relativizes for storage/keys and
   resolves a stored relative path back to absolute for `reconcile_source`'s filesystem read. One
   function both directions — they must not drift.
4. **Transition-safe `by_source` matching — the re-mint guard** (F-4). The 61 live nodes hold
   **absolute** paths now. Make the `by_source` key the **canonical-relative form derived from whatever
   is stored** (canonicalizing an absolute stored path to relative) *and* from the discovered path — so
   an absolute-stored node and a relative-discovered lookup collapse to the **same key**, get **matched
   and rewritten in place, and are never re-minted**. This is the s05 `to_populate` transition applied
   to the path *form*. **Get this right before any live run** — it's the difference between rewriting 61
   nodes and minting 61 duplicates.
5. **Tests (fixtures/`TempDir`)** (F-2…F-6): storage form; canonical-invariance to arg spelling; the
   body-hash gate still passes via anchor+relative read; **the transition** (seed absolute → re-migrate →
   matched + rewritten, 0 created); **cross-checkout determinism** (two `TempDir` roots → identical
   `source.paths`, no duplicate on cross-root re-run); coverage set-difference stable across roots.
6. **Live corrective re-migration** (F-7/F-8), behind the s07 protocol: confirm `1.0.x` green + store
   clean; **capture the known-good SHA + before-manifest**; **dry-run** → adjudicate (**61 rewrites, 0
   creates, 0 body/id/schema changes**; store byte-identical after) → **fire as one revertible commit**.
   Then verify on the committed store: paths relative+canonical; 0 stubs; 61 source-bearing;
   project+retired excluded; **bodies/ids/schema unchanged** (only the path string moved); `check` green;
   `orient`/`rollup` byte-stable; **a re-run from a different checkout path is idempotent** (0/0).

## Constraints (flag, don't silently change)
- **Only `source.paths` string *form* changes** — not what `source` means (still the identity axis),
  not bodies/ids/schema, not the body-hash gate. If ODD-0025 needs a line on path form, **amend it**;
  don't work around.
- **Anchor = plan-tree git toplevel** (→ `docs/…`), never the superproject root (→ `.worktrees/1.0.x/…`).
- **Dry-run-first + snapshot SHA are non-negotiable.** Any create in the dry-run → the transition guard
  is wrong → stop, don't fire.
- Don't pull **s09** (artifact mint, doc-coverage into `check`, the `coverage.rs` Findings 2–3 fixes, the
  14 design/research nodes' `source`), **s10** (synthesis), or **s11** (reconcile) forward.
- No `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables
The committed rewritten store (`odm` branch) + the code change (`release/1.0.x`); `ledger.md` evidence
per row (`attested`/`reproduced` — cite actual path strings, counts, command exits); `closing-report.md`
— per-row walk, the before/after (absolute→relative path examples, 61 rewritten / 0 created), the case-rule
decision, any findings, **plus the v2.0 Bubble-up** (did s08 make paths portable; what the transition
revealed; the silent-drop diff vs In/Out). Branch: `release/1.0.x` (impl) + `odm` (store).

## Working agreement
Amend don't work around; flag every deviation; five-iteration cap (+ split-escape if the capability and
the live rewrite won't both land in one context — land the **relativization + transition-safe matching +
fixtures** first, split the live rewrite, flag CDC). Your `done` is proposed-done — CDC reproduces the
store-state rows by direct read (path form, cross-root determinism, body/id invariance) + CI. On close,
bubble up to `../arc-plan.md` (Finding 1 resolved; s09 next).
