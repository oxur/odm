# Slice 01 (Arc 04) — Index record + snapshot persistence (plan-of-record)

> Refs: ODD-0014 §3.1 (index record shape + header), §3.3 (persistence format + atomic
> write), §0 (the stat-cache convergent finding); arc04 `arc-plan.md` slice01 + the Arc
> Ledger. `depends_on:` A1 (the store's `atomic::write` + frontmatter types this record
> mirrors). First slice of A4 — it creates the `odm-index` crate.
>
> **Why this slice exists:** A4's index is a persisted, sorted, derived **stat-cache**.
> Before anything can *build* it (slice02) or *read it incrementally* (slice03), the
> on-disk artifact needs a shape: a record type, a versioned + checksummed snapshot
> format, and a crash-safe way to write and re-read it. This slice lands exactly that —
> the foundation the rest of the arc stands on — and nothing more.

## Goal

Create the `odm-index` crate and define the index **snapshot**: the `IndexRecord` type,
a versioned + checksummed file header, serialization that round-trips, atomic
persistence (reusing the store's `atomic::write`), and self-healing on a corrupt or
version-mismatched file. **Done when** a set of records can be written to a snapshot and
read back identically, the write is crash-safe, and a corrupt/old-format file is
detected on load and reported as "rebuild needed" — never silently mis-parsed.

## Scope

**In:**

- **The `odm-index` crate**, added to the workspace members, building clean.
- **`IndexRecord`** (ODD-0014 §3.1): identity (`id`, `rel_path`); the stat-cache fields
  (`mtime_secs`, `mtime_nsec`, `size`, `inode`, `mode`); the two fingerprints
  (`content_hash`, `meta_hash`); and the extracted metadata for in-memory filter/sort
  (`node_type`, gate/state, `tags`, `edges`, `title`, `updated`). The *type* — populating
  it from real files is slice02.
- **Snapshot header** (§3.1): magic bytes, **format-version**, hash-algorithm id, the
  **index-timestamp** (for slice03's racy `>=` test), record count, and a **trailing
  checksum** over the body.
- **Serialization round-trip:** `encode ∘ decode = identity` over a set of records, via
  **postcard** (compact, stable wire format, pure Rust — ODD-0014 §3.3); hashing reuses
  the workspace's existing **`sha2`** (a fast non-crypto hash like xxh3 is a deferred
  perf option, §3.1).
- **Atomic persistence:** write the snapshot via `odm_store::atomic::write` (temp +
  fsync + rename + dir-fsync — already implemented; **reuse, don't reimplement**).
- **Self-healing signal:** on load, a bad checksum or a magic/format-version mismatch is
  detected and surfaced as a typed "rebuild needed" outcome — never a silent bad parse.

**Out:** the cold-path build (walkdir + `lstat` + parse → records) — slice02; warm-path
change detection (the racy `>=` test, deletion detection) — slice03; the in-memory
filter/sort maps + wiring consumers — slice04; early-cutoff — slice05; benchmarks —
slice06; sharding, rkyv/mmap, and any watcher — deferred (arc-plan open questions).

## Verification

`cargo test -p odm-index` green (record shape, header+checksum, round-trip proptest,
atomic persist, corrupt/version self-heal); `cargo build` adds `odm-index` to the
workspace; clippy `-D warnings`; no `unsafe`; coverage ≥ 90% (line) for `odm-index`.
Rows in `ledger.md`.

## Exit

`ledger.md` closed; CDC verified (cargo rows via CI / local 1.85+ — `attested` →
`reproduced` per LEDGER-DISCIPLINE v2.0). The snapshot format exists and is crash-safe;
slice02 can fill it from a corpus walk.
