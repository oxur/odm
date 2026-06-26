# CC Prompt — Slice 01 (Arc 04): Index record + snapshot persistence

Create the `odm-index` crate and land the index **snapshot**: the `IndexRecord` type, a
versioned + checksummed file format, postcard round-trip, atomic persistence (reusing the
store), and self-healing on a corrupt/old file. This is A4's foundation — the on-disk
shape the build (slice02) and incremental detection (slice03) will stand on.

> **Start condition:** A1–A3 on `main`, CI-green. First slice of A4; `odm-index` does
> not exist yet — you create it. If `main` isn't green, hold.

## Read first
1. `slice01-record-persistence/ledger.md` (8 rows).
2. `slice-doc.md` (same dir) and `../arc-plan.md` (the Arc Ledger + the A4 capability).
3. **ODD-0014 §3.1** (record shape + header) and **§3.3** (persistence format + the
   atomic-write sequence) — the direct design source. §4 for the guardrails (stat-only
   is a correctness bug; nanosecond mtime is not a correctness signal — both matter in
   slice03, but read them now).
4. `crates/odm-store/src/atomic.rs` (`atomic::write` — temp + fsync + rename + dir-fsync;
   **reuse it**) and the frontmatter types the record's metadata mirrors.

## Load skills (via `/<name>`)
- `/rust-guidelines` — anti-patterns first, then `05-type-design.md`, `02-api-design.md`.
- `/collaboration-framework` → LEDGER-DISCIPLINE (now v2.0 — note the evidence-strength
  column: fill evidence at `attested` per commit; CDC reproduces).

## Task
1. **Create `odm-index`** and add it to `[workspace] members`; it builds clean.
2. **`IndexRecord`** (0014 §3.1): identity (`id`, `rel_path`); stat fields
   (`mtime_secs`, `mtime_nsec`, `size`, `inode`, `mode`); fingerprints (`content_hash`,
   `meta_hash`); extracted metadata (`node_type`, gate/state, `tags`, `edges`, `title`,
   `updated`). Define the type; **do not** populate from files (that's slice02).
3. **Snapshot header:** magic, format-version, hash-algo id, **index-timestamp**, record
   count, **trailing checksum** over the body.
4. **Serialize with postcard** (add the dep); `encode ∘ decode = identity` over a record
   set (proptest). **Hash with the workspace `sha2`** — add no new hash dep (xxh3 is a
   deferred perf option, note it).
5. **Persist atomically by reusing `odm_store::atomic::write`** — do not reimplement the
   temp/fsync/rename sequence.
6. **Self-heal:** on load, a bad checksum or magic/format-version mismatch returns a
   typed "rebuild needed" outcome — never a silent bad parse.

## Constraints (flag, don't silently change)
- **Amend, don't work around.** Raise an amendment if a row is wrong/impossible.
- **Reuse `odm_store::atomic::write`** for persistence; don't duplicate it.
- **No `unsafe`** (rkyv/mmap are deferred per the arc-plan, so none is needed); typed
  errors (`thiserror`); coverage ≥ 90% (line), target 95%.
- Scope is the *type + format + persistence only* — resist pulling slice02's walk or
  slice03's change-detection forward.

## Deliverables
Green `cargo test -p odm-index` + clippy + coverage; `ledger.md` evidence per row (at
`attested`); `closing-report.md` — the per-row walk **plus the v2.0 Bubble-up to the arc**
section (did slice01 deliver its assigned piece of the A4 capability; what did it reveal
the arc-plan didn't anticipate; the slice-scale silent-drop diff). Feature branch
(`arc04-slice01-record-persistence`); not `main`.

## Working agreement
Amend don't work around; flag every deviation rather than burying it; five-iteration cap;
your `done` is *proposed-done* (`attested`) → CDC reproduces (cargo rows via CI / local
1.85+). On close, bubble up to `arc-plan.md` per LEDGER-DISCIPLINE v2.0 §A / PM Part IV.
