# Closing report — Arc 04 / Slice 01: Index record + snapshot persistence

> CC implementation closing report. Status: **proposed-done** (`attested`) → CDC
> reproduces the cargo rows via CI / a local 1.85+ toolchain (`attested →
> reproduced`). Impl commit `69952ce`; docs commit (this report + ledger +
> arc-plan bubble-up) follows.

## What this slice built

A4's on-disk foundation: the new **`odm-index`** crate with the `IndexRecord`
type, a versioned + checksummed snapshot format, postcard round-trip, crash-safe
atomic persistence (reusing the store), and a self-healing load. Type + format +
persistence only — populating records from a corpus walk is slice02.

## Per-row ledger walk (8 rows)

- **F-1 — done (attested).** `odm-index` is a workspace member and builds clean;
  `postcard` added to `[workspace.dependencies]`.
- **F-2 — done (attested).** `IndexRecord` carries identity + stat-cache +
  fingerprints + extracted metadata; `index_record_shape_carries_all_fields`
  asserts every field group.
- **F-3 — done (attested).** The header carries magic, format-version, hash-algo
  id (fixed offsets), index-timestamp, record count, and a trailing checksum;
  `snapshot_header_fields_present` checks all six plus a round-trip.
- **F-4 — done (attested).** `encode ∘ decode = identity` over 0–20 arbitrary
  records (proptest), via postcard.
- **F-5 — done (attested).** `persist` reuses `odm_store::atomic::write` (grep
  confirms the single call site); `snapshot_atomic_persist_and_reload` writes and
  reloads identically.
- **F-6 — done (attested).** Every corruption/staleness mode is a typed
  `RebuildReason`; tested at the decode level (BadChecksum, TooShort, BadMagic,
  VersionMismatch, UnknownHashAlgo, Missing) and end-to-end through `load` on a
  corrupt on-disk file. Never a silent bad parse.
- **F-7 — done (attested).** Hashing reuses the workspace `sha2`; no new hash dep
  (xxh3 documented as a deferred perf option).
- **F-8 — done (attested).** clippy `-D warnings` → exit 0; no `unsafe`; line
  coverage **98.78%**; `fmt --check` clean; full workspace `cargo test` → 234
  passed.

## Deviations / interpretations flagged (not buried)

1. **`gate/state` → `gates: Vec<String>`.** ODD-0014 §3.1 names a single
   `state: State` field, written generically before odm's status model was fixed.
   odm has **no single lifecycle state** — status is a multi-gate vector — so I
   recorded the **reached-gate name set** (`gates`), which is strictly more
   information and supports "filter by gate/state" (slice04). Flagged for the
   CDC; if a single derived "current state" is wanted later, it is computable from
   `gates` + the gate sequence.
2. **The index owns its `EdgeKind`** (mirroring, not reusing, `odm_core::graph::
   EdgeKind`). `odm_core`'s enum is not `Serialize` and is an internal domain
   type; making it the cache wire format would couple the snapshot to domain
   internals. The index's copy is governed by the snapshot format-version — the
   correct seam for a versioned cache. Slice02 maps domain edges → `EdgeRef`.
3. **`created` is omitted from the record.** ODD-0014 lists `created: Ulid-derived`
   but it is *derivable from `id`* (the ULID embeds its timestamp), so storing it
   would duplicate state. F-2's field list does not include it. (`Id::created_at()`
   recovers it on demand.)
4. **Checksum/fingerprints use SHA-256, not xxh3.** Per the slice/ledger (F-7),
   reusing the workspace `sha2` rather than adding a fast-hash dep. ODD-0014 §3.1
   *recommended* xxh3 for the fingerprints; that is a deferred perf tweak, noted
   in `snapshot.rs`. Flagged because it is a (sanctioned) departure from the
   research's hash recommendation.

## Uncertainties / things CDC should look at

- **One uncovered line: `snapshot.rs:264` (`CountMismatch`).** It is a
  belt-and-suspenders guard — reachable only by a body whose stated `record_count`
  disagrees with its decoded records *while still passing the checksum*, i.e. a
  forged/hand-tampered file. The public API never produces it (encode writes
  `records.len()`), and `BodyOwned`/`BodyRef` are private, so it cannot be
  exercised through the crate's surface. Left uncovered deliberately; named here.
  (Crate line coverage is 98.78%.)
- **`record_count` is arguably redundant** with postcard's own length-prefix on
  the `records` vec. I kept it because ODD-0014 §3.1 lists "record count" as a
  header element and it gives a cheap independent consistency check. Flag if you'd
  rather drop it (would also retire the `CountMismatch` guard above).
- **`index_timestamp` is an `i64` Unix-seconds stamp, not stamped here.** This
  slice never *sets* it from the clock (that's slice03's "set just before write");
  it is a plain field that round-trips. No `Date.now()`-style call exists yet.

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

- **Did slice01 deliver its assigned piece of the A4 capability?** Yes. The arc's
  capability needs "a persisted, sorted, derived stat-cache … self-healing
  (corrupt/missing ⇒ rebuild)". This slice delivers the *artifact's shape and
  lifecycle*: the record, the versioned+checksummed format, crash-safe write, and
  the typed rebuild signal. It deliberately does **not** build or read the cache
  from files (slice02/03) — exactly its assigned scope. The arc-plan's slice01
  one-liner is satisfied as written.
- **What did it reveal the arc-plan didn't anticipate?**
  - The arc-plan (via ODD-0014) carried a generic `state: State` field; odm's
    multi-gate model has no such scalar, forcing the `gates: Vec<String>` decision
    (deviation 1). **Arc-plan input for slice04:** the filter/sort maps build on a
    *gate set* per node, not a single state — the "filter by state" affordance is
    really "filter by reached gate".
  - ODD-0014 recommended xxh3 for fingerprints but the slice mandated `sha2`
    (F-7). The `HashAlgo` enum + the 1-byte on-disk algo id make a later xxh3
    swap a format-versioned, non-breaking change — **the arc already has its
    migration hook** if slice06's benchmark shows hashing dominates.
  - `EdgeKind` had to be owned by the index (deviation 2) — a small but real
    decoupling the arc-plan's "extracted metadata (… edges)" one-liner glossed.
- **Slice-scale silent-drop diff (scope-as-specified vs. scope-as-delivered):**
  none. Every "In" item in `slice-doc.md` shipped (crate, record, header,
  round-trip, atomic persist, self-heal); every "Out" item (cold-path walk,
  change detection, filter/sort maps, early-cutoff, benchmarks, sharding/rkyv/
  mmap/watcher) was held out, not partially started. No criterion was softened;
  no row dropped (8 opened, 8 dispositioned).

## Iterations

One. Two minor in-slice corrections before close: a thiserror message that tried
to interpolate a module const (`{FORMAT_VERSION}` → reworded), and two added
tests to cover the `load`-level corruption path and the non-`NotFound` read error
(lifting coverage 96.34% → 98.78%).
