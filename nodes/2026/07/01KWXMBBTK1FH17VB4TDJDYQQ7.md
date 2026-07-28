---
id: 01KWXMBBTK1FH17VB4TDJDYQQ7
number: 1407
type: slice
schema: slice/v1.1
name: Early-cutoff invalidation
created: 2026-06-29
updated: 2026-07-28
origin: planned
reserved: false
source:
  paths:
  - /Users/oubiwann/lab/oxur/odm/.worktrees/1.0.x/docs/design-v1.0.0/arc04-index-cache/slice07-early-cutoff/slice-doc.md
  class: slice-doc
  normalization: trim+lf
  migrated_by: odm-migrate/1.0.0
  migrated_on: 2026-07-28
edges:
  part_of: 01KWXMBBTKJQ1TKZP4DKFPACA7
status:
  built:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  planned:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
  tested:
    reached: 2026-07-07
    evidence: asserted
    evidence_dates:
      asserted: 2026-07-07
---
# Slice 07 (Arc 04) — Early-cutoff invalidation (plan-of-record)

> Refs: ODD-0014 §2.4 (early cutoff; **"for odm's batch CLI, eager recompute of the
> affected closure is acceptable"**), §2.5 (the two fingerprints — `content_hash` input,
> `meta_hash` derived); arc04 `arc-plan.md` slice07. `depends_on:` slice03 (the warm
> `reconcile` + `Delta`), slice06 (every consumer reads the index; `meta_hash` covers all
> semantic fields — evidence + origin + decomposed). *(Was slice05 → renumbered to 07,
> v1.10.)*
>
> **Why this slice exists:** the two-fingerprint split has been built into the record all
> along (`content_hash` = did the file change; `meta_hash` = did its *meaning* change) so
> that a **body-only edit** can update the record but **recompute nothing downstream**.
> This slice cashes that: the reconcile delta distinguishes meta-changed from body-only,
> and the one *persisted* derived artifact — `ROLLUP.md` — is **not** regenerated when the
> corpus is semantically unchanged.

## Goal

A body-only change is cheap end-to-end. **Done when** `reconcile`'s `Delta` marks which
changed records are **meta-changed** (vs. body-only: `content_hash` differs, `meta_hash`
same), and `odm rollup` **skips regenerating `ROLLUP.md`** on a semantically-unchanged
corpus (body-only edits / no new / no deleted) while still regenerating on any
`meta_hash` change, new, or deleted node — the body-only edit having refreshed the index
record but recomputed nothing downstream.

## Scope

**In:**

- **Meta-changed vs. body-only in the delta.** `reconcile` already holds both the prior
  record and the freshly-built one at the CHANGED classification — so it can compare
  `meta_hash`. A changed record whose **new `meta_hash` == prior `meta_hash`** is a
  *body-only* change. The `Delta` exposes the **meta-changed** subset (the ids whose
  *meaning* changed) distinct from body-only.
- **`odm rollup` early-cutoff.** Skip regenerating `ROLLUP.md` when the corpus is
  semantically unchanged since the last generation — i.e. no meta-change, no new, no
  deleted. *Recommended mechanism:* a combined **meta-fingerprint** (a hash over the
  sorted record `meta_hash`es) stamped in `ROLLUP.md`'s generated header; `odm rollup`
  recomputes the current fingerprint from the (reconciled) index and **skips the
  rewrite** if it matches — else regenerates and re-stamps. Robust to history (compares
  actual semantic state, not reconcile bookkeeping).
- **Record-refresh still happens.** A body-only edit still updates the index record
  (`content_hash` + stat) via the warm path — early-cutoff skips the *derived* recompute,
  **not** the record refresh (the §2.4 distinction).

**Out (per ODD-0014 §2.4):** persistent caching of the *in-memory* derived artifacts
(the assembled `NodeGraph`/`Rollup` model the readers build each invocation). 0014
explicitly accepts **eager recompute** for the batch CLI — each `odm` invocation rebuilds
the graph from records, which is acceptable; the in-memory readers (`next`/`blocked`/
`path`/`check`/`orient`) are **unchanged**. Revisit only if slice08's benchmark shows the
in-memory rebuild dominates (a post-arc / A4-follow concern, not this slice). Also out:
the benchmark itself — slice08.

## Design notes (settle here)

- **0014 §2.4 sets the ceiling.** Early-cutoff for a per-invocation CLI is the
  *persisted-derived-artifact* skip (`ROLLUP.md`) + the meta-changed signal — **not**
  Salsa-style lazy/persistent graph caching. The research already made that call; this
  slice honours it. The `meta_hash` foundation (slice02/04/06) is what makes the skip a
  cheap comparison, not a recompute.
- **The meta-changed signal is also forward value:** A5 reconcile and A7 telemetry will
  read "what *meaningfully* changed," not just "what touched disk."
- **Reuse:** the comparison rides slice03's `reconcile`/`Delta` + slice02's `meta_hash`;
  no new traversal or parse.

## Verification

`cargo test -p odm-index -p odm-cli` green (delta meta-changed vs body-only; `odm rollup`
skips on body-only / regenerates on meta-change/new/deleted; record still refreshes on a
body-only edit; in-memory readers unchanged); clippy `-D warnings`; no `unsafe`; coverage
≥ 90% (line) for odm-index + odm-cli. Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI / local 1.85+). A body-only edit
refreshes the record but leaves `ROLLUP.md` untouched; a meaning-change regenerates it.
Only **slice08 (the 100k benchmark)** remains — and it measures the finished system,
which can then settle whether deeper (in-memory) caching is ever warranted. Bubble up to
`arc-plan.md` (A-7) at close.
