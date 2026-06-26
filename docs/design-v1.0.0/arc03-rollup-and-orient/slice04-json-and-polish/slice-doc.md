# Slice 04 (Arc 03) — `--json` + polish (plan-of-record)

> Refs: ODD-0013 §7 (command surface — `--json` on every query, "stable, documented
> schemas"), §6 (rollup), §4.1 (orient); arc-plan slice04 (← MVP COMPLETE); slice01
> `cdc-verification.md` forward note + slice02 ruling 4 (the `check --json` v2 envelope
> has evolved twice — pin it); ODD-0017 (interop export consumers target these
> schemas). `depends_on:` slice02 (the `Rollup` model), slice03 (the orient view).
>
> **Why this slice exists:** slices 02–03 render human-readable Markdown only. The MVP
> promises `--json` "on every query" with stable schemas (0013 §7) — the machine
> contract that ODD-0017's export projection will target. This slice adds `--json` to
> `rollup` and `orient` over the **same model** (D-3, no second derivation), pins the
> canonical schemas (incl. the twice-evolved `check` v2 envelope), and does the
> errors-as-affordances polish pass. **It closes Arc 03 → MVP (A1–A3) complete.**

## Goal

Give `rollup` and `orient`/`brief` a stable, documented `--json`, serialized from the
slice02 model (not re-derived); lock every query's `--json` envelope as a canonical
schema (documented + shape-tested); and finish the errors-as-affordances pass. **Done
when** every query emits valid, documented, shape-locked JSON — including on the empty
corpus and no-project paths — and `check`/`rollup`/`orient` schemas are pinned against
silent drift.

## Scope

**In:**

- **`odm rollup --json`** — serialize the `Rollup` model (tree, status vectors,
  ready/blocked with soft-sat, active tears with rationale, provenance, drift +
  deferred slots) as `#[derive(Serialize)]` views, rendered with
  `serde_json::to_string_pretty` (the established pattern). Same model as the Markdown
  render (D-3).
- **`odm orient --json`** (and `brief --json`) — serialize the orient view (vision,
  current focus, ready/blocked, integrity, drift) over the same model.
- **Canonical schema pinning.** Document the stable `--json` schemas for `check`,
  `rollup`, and `orient` in **0013 §7** (a "JSON output schemas" note), and lock each
  envelope's shape with a test (keys + types) so an accidental change fails CI. The
  `check` v2 envelope (`ok`, `errors`, `warnings`, `findings[]`, `tears[]`) is pinned
  as-is — it has evolved twice without a contract; this fixes that.
- **`schema_version` marker** (additive) on the `rollup`/`orient`/`check` envelopes so
  consumers (0017) can detect future evolution. *Small new convention — flag for
  ratification (see design note).*
- **Errors-as-affordances polish.** A consistency sweep: every `orient`/`rollup`
  message and no-project/empty path names an exact fix command; `--json` stays valid on
  the empty corpus and no-project paths (never a bare error, never invalid JSON).

**Out:** the `.odm/` cache (A4); `reconcile`/drift computation (A5); deferred-node
surfacing (A5, Q-A3-1); the ODD-0017 export projection itself (this slice only *pins
the schemas* 0017 will target); changes to the already-stable `list`/`show`/`context`
schemas (left as-is, optionally version-marked for consistency).

## Design notes (settle here)

- **`schema_version` marker.** Recommended: an additive top-level field (e.g.
  `"schema": "rollup/v1"`) on each query envelope, so 0017 export consumers can pin a
  version. Additive ⇒ existing consumers unaffected. *Reversible call; flag if you'd
  rather pin by documentation + shape-tests alone and add versioning when 0017 lands.*
- **Serialize the model, don't reshape it.** The `--json` views mirror the model
  (D-3); resist inventing a parallel JSON-only structure that could drift from the
  Markdown render.

## Verification

`cargo test -p odm-cli` green (rollup/orient `--json` + the envelope shape-locks); JSON
valid on empty-corpus and no-project paths; 0013 §7 documents the schemas; clippy `-D
warnings`; no `unsafe`; coverage ≥ 90% (line) for odm-core + odm-cli. Rows in
`ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI / local 1.85+). Every query has a
stable, documented, shape-locked `--json`. **On close, Arc 03 is done and the MVP
(A1–A3) is complete** — the self-hosting trigger: the plan migrates *into* `odm` as
nodes (A6). (CDC runs the arc-level recomposition/silent-drop check at arc close.)
