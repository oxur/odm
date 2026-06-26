# Slice 02 (Arc 04) — Cold-path build (plan-of-record)

> Refs: ODD-0014 §2.1 (FIRST-full / SUBSEQUENT-incremental → the stat-cache),
> §3.1 (record fields), §2.5 (the two fingerprints); arc04 `arc-plan.md` slice02 + the
> Arc Ledger; slice01 (the `IndexRecord` + `Snapshot` + `persist` this fills) and its
> bubble-up finding #3 (the index owns its `EdgeKind`; slice02 maps domain edges → it).
> `depends_on:` slice01 (record + snapshot format), A1 (store walk + frontmatter parse).
>
> **Why this slice exists:** slice01 defined the snapshot's *shape* but left it empty.
> This slice does the **first (cold) run**: walk the corpus, stat + hash + parse every
> node file, populate the `IndexRecord`s, and persist a full snapshot. It is the
> O(corpus) pass paid once; the cheap incremental warm path is slice03.

## Goal

Build a full index snapshot from a corpus walk. **Done when** a cold build over
`nodes/` produces one fully-populated `IndexRecord` per node file — stat fields from
`lstat`, `content_hash` over the raw bytes, the extracted metadata (incl. `gates` from
the reached-gate set and `edges` mapped to the index's `EdgeRef`), and a deterministic
`meta_hash` — assembles them with an `index_timestamp`, and persists via slice01's
`Snapshot`; a built-then-loaded index round-trips identically.

## Scope

**In:**

- **The corpus walk.** Enumerate node files under `nodes/` — **reusing odm-store's walk**
  (the path it already uses in `load_all`); factor out a path-yielding helper rather
  than re-deriving the `nodes/YYYY/MM/<ULID>.md` traversal in `odm-index`. A missing or
  empty `nodes/` yields an empty record set (not an error).
- **Per-file record population** (`IndexRecord`, slice01):
  - **stat fields** (`mtime_secs`/`nsec`, `size`, `inode`, `mode`) from `lstat`;
  - **`content_hash`** = SHA-256 over the raw file bytes (the input fingerprint — slice03
    consumes it);
  - **extracted metadata** from the parsed `Document`: `node_type`, `gates` (the reached
    gate names from `status().reached()`), `tags`, `title` (= `name()`), `updated`;
  - **`edges`** — domain `Edges` mapped to the index's `EdgeRef`/`EdgeKind` (slice01
    finding #3), across all edge kinds present;
  - **`meta_hash`** = SHA-256 over a **canonical, deterministic** encoding of the
    extracted metadata (the derived fingerprint — slice05's early-cutoff consumes it);
    identical metadata ⇒ identical `meta_hash` across runs.
- **Assemble + persist.** Collect the records, set `index_timestamp` = now (just before
  write), and persist via slice01's `Snapshot` (**reuse `Snapshot::persist`; do not
  reimplement** encode/atomic-write).

**Out:** warm-path change detection (the `lstat`-compare, racy `>=` test, deletion
detection) — slice03; in-memory filter/sort maps + pointing `list`/`orient`/graph-build
at the index — slice04; early-cutoff *consumption* (meta_hash diffing to skip recompute)
— slice05; the benchmark harness — slice06.

## Design notes (settle here)

- **`EdgeRef` qualifier fidelity (open).** Does the cold build preserve edge *qualifiers*
  — `depends_on`'s `satisfied_at`, `supersedes`' kind, a tear's `because` — or only
  `(target, kind)`? It depends on what slice04's index-backed graph-build needs to
  reconstruct the ordering DAG + satisfaction. **Resolve against slice01's `EdgeRef`
  fields:** if it carries only `(target, kind)`, either enrich it now or flag for slice04
  that graph-build still reads full frontmatter for qualifiers. Pick the answer in the
  ledger; don't smuggle it.
- **Reuse over re-derivation** (the recurring discipline): the walk comes from odm-store,
  the snapshot/persist from slice01, the hashing from `sha2`, the metadata from
  odm-core's frontmatter accessors. `odm-index` adds the *assembly*, not new copies.

## Verification

`cargo test -p odm-index` green (cold build: one record/file, stat fields, content_hash,
metadata, edge mapping, deterministic meta_hash, persist, built-then-loaded round-trip,
empty-corpus); clippy `-D warnings`; no `unsafe`; coverage ≥ 90% (line) for `odm-index`.
Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI / local 1.85+ — `attested` →
`reproduced`). A cold build produces a full, persisted snapshot; slice03 can take the
warm path (load + `lstat`-compare the delta) against it. Bubble up to `arc-plan.md`
(Arc Ledger A-2) at close.
