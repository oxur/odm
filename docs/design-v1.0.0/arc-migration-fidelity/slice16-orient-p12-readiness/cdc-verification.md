# Slice 16 (Migration Fidelity) — CDC verification: orient P-12 readiness

> **Arc:** Migration Fidelity · **Slice:** 16 · **Verifier:** CDC (independent) · **Date:** 2026-08-02 ·
> **Method:** LEDGER-DISCIPLINE v2.0 §A. **Fixture slice.** Code + fixtures reproduced by direct read on
> `release/1.0.x`; runtime attested-by-CC → CI (+ CC's real-corpus binary run on the orphan branch).
> `odm` store unaffected (this is `orient` read-path code; the store is committed at `41ace1f`).

## Verdict

**PASS — CDC-verified.** `odm orient` now excludes retired nodes from every surface it presents — project
selection, READY, and BLOCKED — so on the collapsed corpus it auto-orients to the single active project
(`#1000`) and renders its vision, resolving the P-12 symptom. The fix is correct **and** dodged a real trap
I planted (§2). Diff stayed entirely in `orient.rs` + its tests, per scope.

## 1. The fix (F-1/F-2/F-3) — reproduced

`orient.rs`:
- **`is_retired(store, id)`** (line 376): `store.load(id).is_ok_and(|d| d.frontmatter().retired().is_some())`
  — a **targeted per-node store read**, checking the *loaded* document's retirement.
- **Project selection** (F-1, line 71): `.filter(|f| f.node_type() == Project && !is_retired(store, f.id()))`
  — a retired project is never counted, so the existing `len() == 1` path auto-orients to the sole active
  one.
- **READY** (F-2, line 109) + **BLOCKED** (F-3, line 110): `model.ready.retain(|r| !is_retired(...))` /
  `model.blocked.retain(...)`. **CC's F-3 audit found BLOCKED had the identical leak** — a real
  addition beyond the literal ask, correctly folded into the one "orient never surfaces a retired node"
  rule.

## 2. The trap CC dodged — and it was mine

The cc-prompt (and the slice-doc) specified the predicate as **`f.retired().is_none()` on the frontmatter
`orient` already has**. That is **wrong** — a silent no-op: those frontmatters are reconstructed from the
`.odm/` index projection, which does **not** carry retirement, so the guard would compile, pass clippy, and
filter nothing. CC caught it immediately via the fixture (a real store with a retired node — see §3), traced
it to the same gap `commands.rs`'s `check` had already hit and documented, and used the correct
`store.load()`-based `is_retired`. **This is a CDC-prompt defect caught by CC's fixture discipline, not a CC
defect** — worth naming plainly: my literal predicate would have shipped a green-but-inert filter; the
real-store fixture is exactly what exposed it. Precedent-matching (the `commands.rs` per-node load) is the
right resolution.

## 3. Tests (15/15) — reproduced, non-vacuous

The retired-exclusion tests build **real stores with retired nodes** (not index frontmatters), which is what
makes them catch the dead predicate:
- `orient_excludes_retired_project_from_selection` — active `#1` + retired `#2` on disk, no `use project` →
  asserts orient auto-orients (`VISION  #1 Active`), **never** "none selected", the tombstone **not** listed,
  and `--json` `project.number == 1`. This is the P-12 scenario, reproduced.
- `orient_still_lists_multiple_active_projects` — guards against over-filtering: two *active* projects still
  prompt.
- `orient_excludes_retired_node_from_ready` (F-2) + `orient_excludes_retired_node_from_blocked` (F-3).
Plus CC's real-corpus run (the `.worktrees/odm` orphan branch, read-only): `orient` auto-orients to `#1000`'s
vision, `#1001` absent from the human view and `--json`. Attested; the fixture above reproduces the same
shape structurally.

## 4. Ledger — CDC disposition

F-1/F-2/F-3 reproduced in code + real-store fixtures; F-4 (no scope bleed) reproduced — the diff is
`orient.rs` + `tests/orient.rs` only, `next`/other surfaces untouched, `#1001` still a retired tombstone;
F-5 attested→CI (clippy/`unsafe`/coverage). **5 rows, no silent drops.** The broader F-22 (`next`, and any
other command) stays routed to the LLM-command-surface arc, as scoped.

## 5. Bubble-up (PM Part IV)

- **Did s16 deliver?** Yes — `orient` demonstrates P-12: a fresh session auto-orients to the single active
  project and its vision; retired nodes are absent from selection, READY, and BLOCKED.
- **Arc-plan change:** flip s16 → **CDC-verified PASS**. **The arc-close is now the sole remaining step** —
  a distinct CDC/gate activity: the MF-9 composition check (fidelity already reproduced — 0 drift), the P-12
  demonstration (now clean; `41ace1f` store committed), and the bubble-up (mark the arc CLOSED in
  `arc-plan.md` + `project-plan.md`; `odm node decomposed 58837400` to clear the arc node's
  undecomposed-parent warning).

## Closure

s16 **CDC-verified PASS** on 2026-08-02. `orient` excludes retired nodes at all three surfaces via a correct
`store.load`-based check (dodging the inert frontmatter predicate the prompt specified — a disclosed CDC
spec defect); real-store fixtures + CC's binary run demonstrate the clean P-12 orient. **Migration Fidelity
is one step from closed: the arc-close.**

_Verified by: CDC (independent), 2026-08-02 — `release/1.0.x` working tree; code/fixtures by direct read,
execution attested-by-CC → CI._
