# CC Prompt — Slice 02 (Arc 04): Cold-path build

Do the first (cold) run: walk the corpus and fill the snapshot slice01 defined. One
fully-populated `IndexRecord` per node file — stat + content_hash + parsed metadata +
mapped edges + a deterministic meta_hash — assembled and persisted via slice01's
`Snapshot`. The O(corpus) pass paid once; the incremental warm path is slice03.

> **Start condition:** slice01 (record + snapshot persistence) CDC-verified / CI-green —
> `IndexRecord`, `Snapshot`, `persist`, and the index's `EdgeRef`/`EdgeKind` exist. If
> slice01 isn't in, hold.

## Read first
1. `slice02-cold-path-build/ledger.md` (9 rows).
2. `slice-doc.md` (same dir) and `../arc-plan.md` (Arc Ledger A-2; the A4 capability).
3. **ODD-0014 §2.1** (the cold/warm split), **§3.1** (record fields), **§2.5** (the two
   fingerprints: `content_hash` input vs `meta_hash` derived).
4. The pieces you reuse: `crates/odm-index/src/{record.rs,snapshot.rs}` (slice01 —
   `IndexRecord`, `EdgeRef`/`EdgeKind`, `Snapshot::persist`); `crates/odm-store` (the
   `load_all` walk + `Document` parse); odm-core frontmatter accessors (`node_type`,
   `name`, `updated`, `tags`, `edges`, `status().reached()`).

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `02-api-design.md`, `05-type-design.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE v2.0 (fill evidence at `attested`;
  write the **Bubble-up to the arc** section in the closing-report).

## Task
1. **Walk** `nodes/` — **reuse odm-store's walk** (factor a path-yielding helper out of
   `load_all` rather than re-deriving the `nodes/YYYY/MM/<ULID>.md` traversal in
   `odm-index`). Missing/empty `nodes/` → empty record set, no error.
2. **Populate each `IndexRecord`:** stat fields from `lstat`; `content_hash` = SHA-256 of
   the raw bytes; metadata from the parsed `Document` (`node_type`, `gates` = reached
   gate names from `status().reached()`, `tags`, `title` = `name()`, `updated`); `edges`
   mapped from domain `Edges` → the index's `EdgeRef`/`EdgeKind`; `meta_hash` = SHA-256
   over a **canonical, deterministic** encoding of the extracted metadata.
3. **Assemble + persist:** records + `index_timestamp` (= now at build) + count → slice01's
   `Snapshot`; persist via `Snapshot::persist` (**reuse**, don't reimplement).

## Constraints (flag, don't silently change)
- **Amend, don't work around.** Raise an amendment if a row is wrong/impossible.
- **Reuse, not reimplement:** the walk (odm-store), the snapshot/persist (slice01), the
  hashing (`sha2`), the metadata accessors (odm-core). `odm-index` adds assembly only.
- **Resolve the `EdgeRef` qualifier-fidelity question** (slice-doc design note): preserve
  `depends_on`'s `satisfied_at` / supersede-kind / tear-`because`, or just `(target,
  kind)` — decide against what slice04's graph-build needs, and record it on row B-5.
- `meta_hash` must be **order-stable** (deterministic across runs) — it gates slice05.
- No `unsafe`; typed errors; coverage ≥ 90% (line), target 95%.

## Deliverables
Green `cargo test -p odm-index` + clippy + coverage; `ledger.md` evidence per row (at
`attested`); `closing-report.md` — per-row walk **plus the v2.0 Bubble-up to the arc**
(did slice02 deliver its piece of the A4 capability; what did it reveal the arc-plan
didn't anticipate; the slice-scale silent-drop diff). Feature branch
(`arc04-slice02-cold-path-build`); not `main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration cap;
your `done` is *proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local
1.85+). On close, bubble up to `arc-plan.md` (A-2) per LEDGER-DISCIPLINE v2.0 §A.
