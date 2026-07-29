---
id: 01KYP5G2FB384279R0DSVQJ7RR
number: 559100900
type: artifact
schema: artifact/v1.1
name: 'cc-prompt — RH C-7: Name normalization at the source (names embed no metadata)'
created: 2026-07-26
updated: 2026-07-26
origin: planned
reserved: false
source:
  paths:
  - docs/design-v1.0.0/arc-release-hardening/cc-prompt-c7-name-normalization.md
  class: cc-prompt
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-29
edges:
  part_of: 01KYNDTQ6SRYRWH942F71YMBEG
---
# cc-prompt — RH C-7: Name normalization at the source (names embed no metadata)

> **Arc:** Release Hardening (UAT) · **Chunk:** C-7 · **Covers:** `F-18` (and completes `F-6` at
> the source) · **Kind:** model (amend first) + surface · **Depends on:** C-2 (importer/type work)
> and C-3 (the §2.1 convention + display de-numbering already in place).
> **Chunk number:** **C-7** — **C-6 is reserved for G-2** (tear-rationale; `workflow-gap-coverage-review.md`).
> **Amend first:** fold in `C-7-amendment-ODD-0013.md` (§2.1 → v2.2) before the code.

## Goal

Make node names carry **no embedded metadata** — no number/positional prefixes, no document-role
suffixes like `(plan-of-record)` — **at rest in the data**, not just on display. Today 32 of 47
work-node `name` fields end in `(plan-of-record)` because `self-host` copied plan-doc H1 headings
(`# Arc 01 — Substrate & node CRUD (plan-of-record)`) verbatim.

## Changes

### Code — one shared name-normalizer in the importer

1. Add a single `normalize_name(raw: &str) -> String` used by **both** the `self-host` (work-node)
   and `migrate` (doc-node) name-derivation paths. It strips, from a derived heading/title:
   - a leading **`"<Type> NN —"` / `"<Type> NN (…):"`** number-prefix (e.g. `Arc 01 — `,
     `Slice 05 (Arc 06): `), and
   - a trailing **document-role suffix** from a known set: `(plan-of-record)`, `(build plan)`
     (extendable; keep the list explicit and documented).
   Leave everything else intact — descriptive parentheticals (`(v-major rebuild)`) are **not**
   metadata and must survive (see the amendment's scope note).
2. Unit-test the normalizer directly: prefix-only, suffix-only, both, neither, and a descriptive
   parenthetical that must be preserved.

### Data — regenerate the corpus with clean names

3. **Re-run `odm self-host` + `odm migrate`** so all node `name` fields are re-derived through
   `normalize_name`. Names are `freely editable` and `never affect identity or file location`
   (§2.1), so this is a safe in-place field update — **ids, numbers, gates, edges, bodies all
   unchanged**. Verify the diff touches only `name:` lines.

## Acceptance / ledger

- `cargo build`/`test`/`clippy --all-targets -- -D warnings`/`fmt` green; no `unsafe`.
- **`grep -rE "^name:.*\(plan-of-record\)" nodes` → empty**; no name carries a number-prefix or a
  role-suffix. Descriptive parentheticals (if any) preserved.
- `odm list` shows clean names **sourced from the data** (confirm by inspecting a `name:` field,
  not just the rendered row); the C-3 display-strip is now redundant (keep it as defense, or
  simplify — record the choice).
- `odm check` green; node count unchanged (a name is not identity).
- ODD-0013 §2.1 at **v2.2** landed before the code (model-first).

## Method / housekeeping

- One branch (e.g. `rh-c7-name-normalization`, off the C-3 tip); one mergeable diff; five-iteration
  cap. CC implements on local 1.85+; cargo rows attested-by-CC → reproduced-on-CI.
- On close: bubble up to `arc-release-hardening/arc-plan.md` — new RH ledger row (or fold into the
  compose rows); F-18 done; F-6 noted as now-source-enforced. Add C-7 to the Chunks table when
  scheduled (and C-6 for G-2), so the pipeline stays authoritative.
