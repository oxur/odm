# CC Prompt — Slice 04 (Arc 04): Enrich record + wire consumers

Make the consumers read the index instead of full-scanning the corpus. First enrich the
record so it carries what the graph needs (per-gate **evidence**) — `FORMAT_VERSION → 2`
— then wire `list`, the graph readers, and the composed views off the index.

> **Start condition:** slices 01–03 CDC-verified / CI-green (`Snapshot`/`Load` + the
> version-mismatch self-heal; `build_one`, `meta_hash`, `EdgeRef` qualifiers;
> `reconcile`). If they aren't in, hold.
>
> **This is a large slice.** If it won't hold one context with iteration headroom, split
> at the named seam — (a) enrich+maps+`list` · (b) index→graph adapter + readers · (c)
> composed views — and route the continuation via the bubble-up. **Do not** invent a
> `04b` bisection name; flag it and we renumber properly.

## Read first
1. `slice04-enrich-and-wire/ledger.md` (10 rows).
2. `slice-doc.md` (same dir) and `../arc-plan.md` (slice04 v1.6 + Arc Ledger A-4/A-10).
3. **ODD-0014 §3.5** (in-memory filter/sort; bodies stay out; no FTS), **§2.4** (index
   feeds the graph build).
4. Reuse points: `odm-index` (`build_one`, `Snapshot`/`Load`, `reconcile`, `EdgeRef`);
   `odm-core` (`status().reached()` + `Evidence`; the `NodeGraph`/`Satisfaction`
   builders the adapter feeds); `odm-cli` (`list`, `next`/`blocked`/`path`/`check`,
   `rollup`/`orient` — currently `load_all`-based).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `02-api-design.md`, `05-type-design.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (evidence at `attested`; write the
  **Bubble-up to the arc** section at close).

## Task
1. **Enrich + bump:** `IndexRecord.gates` → per-gate evidence (gate + `Evidence`);
   `FORMAT_VERSION 1 → 2` (an old index → `RebuildNeeded(VersionMismatch)` → cold
   rebuild via slice01's self-heal — no migration code). Populate in `build_one`; extend
   `meta_hash` to cover gate evidence (still exclude `updated`/stat).
2. **In-memory maps** on load: `type→ids`, `tag→ids`, `gate→ids`, edge adjacency. No
   disk after load; no FTS.
3. **Index→graph adapter:** build `NodeGraph` + `Satisfaction` inputs from index records
   (edges+qualifiers+evidence) — no frontmatter parse; feed the *existing* odm-core
   graph/satisfaction (don't re-derive them).
4. **Wire consumers, each `reconcile`-then-read:** `list` (maps); `next`/`blocked`/
   `path`/`check` (index-backed graph); `rollup`/`orient` (index-backed model) — `orient`
   loads only the current project's `Document` for the vision body (one targeted load).
5. **Identical-to-baseline:** each wired consumer's output equals its `load_all` output
   (assert by test).

## Constraints (flag, don't silently change)
- **Amend, don't work around.** Raise an amendment if a row is wrong/impossible.
- **Reuse, not reimplement:** `build_one`, `reconcile`, the odm-core graph/satisfaction,
  the `Rollup`/orient models. The adapter + maps are the only substantial new code.
- **Bodies stay out of the index** (0014 §3.5): only `orient`'s vision triggers a single
  targeted `store.load(project)` — never carry bodies in the record.
- No `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables
Green `cargo test -p odm-index -p odm-cli` + clippy + coverage; `ledger.md` evidence per
row (at `attested`); `closing-report.md` — per-row walk **plus the v2.0 Bubble-up to the
arc** (did slice04 deliver its piece; what it revealed; the silent-drop diff; note A-10
progress). Feature branch (`arc04-slice04-enrich-and-wire`); not `main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration cap
(if you hit it on size, that's the split-seam signal — flag it); your `done` is
*proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local 1.85+). On
close, bubble up to `arc-plan.md` (A-4) per LEDGER-DISCIPLINE v2.0 §A.
