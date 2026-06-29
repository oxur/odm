# CC Prompt — Slice 07 (Arc 04): Early-cutoff invalidation

Cash the two-fingerprint split: a **body-only edit** refreshes the index record but
recomputes nothing downstream. Mark meta-changed vs. body-only in the reconcile delta,
and make `odm rollup` skip regenerating `ROLLUP.md` when the corpus is semantically
unchanged. **Modest slice** — early cutoff for a *batch CLI* is the persisted-artifact
skip + the signal (0014 §2.4), **not** lazy/persistent graph caching.

> **Start condition:** slices 03 + 06 CDC-verified / CI-green — the warm `reconcile`/
> `Delta` and `meta_hash` (covering evidence + origin + decomposed) exist. If not in, hold.

## Read first
1. `slice07-early-cutoff/ledger.md` (6 rows).
2. `slice-doc.md` (same dir) and `../arc-plan.md` (slice07 + Arc Ledger A-7).
3. **ODD-0014 §2.4** (early cutoff — note it **accepts eager recompute for the batch
   CLI**; don't build Salsa-style lazy caching) and **§2.5** (the two fingerprints).
4. Reuse points: `odm-index` `reconcile`/`Delta` (slice03 — extend with the meta-changed
   subset), `meta_hash` (slice02/04/06), the `odm rollup` command + `ROLLUP.md` header
   (slice02).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `02-api-design.md`, `05-type-design.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence at `attested`; write the
  **Bubble-up to the arc** section at close).

## Task
1. **Delta meta-changed vs. body-only:** in `reconcile`, at the CHANGED arm (where you
   already hold the prior record + the freshly-built one), compare `meta_hash`. A changed
   record with **unchanged** `meta_hash` is *body-only*. Expose the **meta-changed** id
   set on the `Delta` (distinct from body-only changes).
2. **`odm rollup` early-cutoff:** skip regenerating `ROLLUP.md` when the corpus is
   semantically unchanged since the last generation (no meta-change, no new, no deleted)
   — leave the file untouched. Regenerate on any `meta_hash` change / new / deleted.
   *Recommended:* stamp a combined `meta_hash`-fingerprint in `ROLLUP.md`'s generated
   header; compute the current fingerprint from the reconciled index and skip if it
   matches.
3. **Record still refreshes:** confirm a body-only edit still updates the index record
   (`content_hash` + stat) via the warm path — only the *derived* recompute is skipped.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** Raise an amendment if a row is wrong/impossible.
- **Never skip a real change:** a `meta_hash` change, new, or deleted node must always
  regenerate `ROLLUP.md`. The cutoff is for body-only edits *only*. Test the regenerate
  paths (gate/edge/origin/decomposed change, new, deleted) as hard as the skip path.
- **In-memory readers stay eager** (0014 §2.4) — do **not** add persistent graph caching;
  `next`/`blocked`/`path`/`check`/`orient` are unchanged. (If you believe the in-memory
  rebuild is a real bottleneck, flag it for slice08's benchmark — don't build caching now.)
- Reuse slice03's `reconcile`/`Delta` + slice02's `meta_hash`; no new traversal/parse.
- No `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables
Green `cargo test -p odm-index -p odm-cli` + clippy + coverage; `ledger.md` evidence per
row (at `attested`); `closing-report.md` — per-row walk **plus the v2.0 Bubble-up to the
arc** (note only slice08 remains). Feature branch (`arc04-slice07-early-cutoff`); not
`main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration cap;
your `done` is *proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local
1.85+). On close, bubble up to `arc-plan.md` (A-7) per LEDGER-DISCIPLINE v2.0 §A.
