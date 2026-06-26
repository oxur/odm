# CDC Verification — Arc 04 / Slice 01: Index record + snapshot persistence

> Independent verification of CC's closed ledger (impl `69952ce`; closed `c7dd6ba`),
> per LEDGER-DISCIPLINE **v2.0** (slice scale, §A). First slice under v2.0 — evidence
> carries a strength; a `done` row reaches ≥ `reproduced` at its own scale. CDC
> reproduces structural rows here; cargo rows route to CI / local 1.85+.

## Environment constraint (disclosed)

No 1.85+ toolchain in the sandbox; cargo executions route to CI / CC's local 1.95.0 run
(held **attested pending CI**). CDC reproduces structural rows by reading the branch
`arc04-slice01-record-persistence` at `69952ce`.

## Row dispositions

**Row count:** 8 opened, 8 addressed. No silent drops. ✔

**Reproduced by CDC (structural, at `69952ce`):**

- **F-1** — `crates/odm-index/` exists; `"crates/odm-index"` in `[workspace] members`;
  `postcard = { version = "1.0", default-features = false, features = ["use-std"] }` in
  `[workspace.dependencies]`. ✔
- **F-2** — `IndexRecord` (`record.rs`) carries id, rel_path, the stat fields
  (mtime_secs/nsec, size, inode, mode), both fingerprints (`content_hash`/`meta_hash`:
  `Digest`), and metadata (node_type, **`gates: Vec<String>`**, tags, edges, title,
  updated). ✔ (the `state → gates` adaptation — ruling 1)
- **F-3** — header is `MAGIC ("ODMINDEX") | version (u16) | algo (u8)` at fixed offsets,
  postcard body `{ index_timestamp, record_count, records }`, trailing 32-byte SHA-256
  checksum over the prefix. ✔
- **F-5** — persistence reuses `odm_store::atomic::write` (`snapshot.rs:278`); no second
  writer. ✔
- **F-6** — `enum RebuildReason` + `Load::RebuildNeeded(..)`; corrupt / version-mismatch
  / missing → typed rebuild outcome, never a silent bad parse. ✔
- **F-7** — `sha2.workspace = true`; no `xxhash`/`twox`/`blake3`. ✔
- **F-8 (no `unsafe`)** — grep over `crates/odm-index/src` → no matches. ✔

**Attested by CC (local rustc 1.95.0), pending CI:** `cargo test -p odm-index` → 7
passed; full workspace → 234 passed; clippy `-D warnings` → exit 0; `fmt --check` clean;
line coverage **98.78%**. → **PENDING CI** (`attested → reproduced`).

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its assigned piece?** ✔ — A4's record + snapshot format + persistence +
  self-heal, exactly the slice01 scope. Every "In" item shipped; every "Out" item held
  out (no walk, no change-detection, no consumers).
- **Silent-drop diff honest?** ✔ — 8/8 rows dispositioned; the one uncovered line
  (the `CountMismatch` guard, unreachable through the public API) is named, not buried.
- **Findings dispositioned + arc-plan updated?** ✔ — CC added arc-plan **v1.2** routing
  all three findings (gates-not-state → slice04; xxh3 format-versioned hook → slice06;
  index-owned `EdgeKind` → slice02), none forcing a re-break. This is the slice→arc
  bubble-up working as designed on its first run.

## Rulings on CC's flagged items

1. **`state: State` → `gates: Vec<String>`.** **Accepted** — faithful to odm's
   multi-gate model (there is no scalar state); disclosed; routed to slice04. *CDC
   action (below): propagate the finding into slice04's body line, which still read
   "state."*
2. **Index owns its `EdgeKind`** (mirrors, not reuses, the domain enum). **Accepted** —
   the correct coupling for a versioned cache: the wire format is governed by the
   snapshot format-version, not hostage to the domain enum. Routed to slice02.
3. **`created` omitted (ULID-derived); SHA-256 over xxh3.** **Accepted** —
   `created`-from-ULID matches ODD-0014 §3.1; SHA-256 satisfies F-7 (reuse `sha2`), and
   the `HashAlgo` enum + 1-byte algo id make the xxh3 swap a format-versioned,
   non-breaking change if slice06's benchmark warrants it. A clean hook.
4. **One uncovered line (the `CountMismatch` guard).** **Accepted** — a
   defense-in-depth internal-consistency check, unreachable through the public encode
   path (count always matches); named explicitly; coverage 98.78% clears the floor.

## CDC actions applied (plan-keeping)

- **Propagated finding #1 to the arc-plan body:** slice04's line read "type/tag/
  **state**/edge maps"; corrected to "**gate**" so the body matches the v1.2 record (the
  plan-change discipline is *change the body + log it*, not log-only).
- **Normalized the new arc-ledger status convention:** A-1's Status read `attested` —
  but that is an *evidence strength*, not a Status (open/done/deferred/no-op). Since
  slice01's cargo rows are attested-pending-CI, A-1 is **`open`** with `attested`
  evidence; it flips to **`done`** when slice01 reproduces (CI green). Set the convention
  on A-1 so A-2…A-6 (and arc05/06's ledgers) inherit it. Recorded as arc-plan **v1.3**.

## Verdict

**Arc 04 / Slice 01 CDC-verified on structure; all four flags accepted; cargo rows
pending CI.** The `odm-index` snapshot foundation is sound — typed self-healing, reused
atomic write, format-versioned for the deferred xxh3/sharding/mmap options. The v2.0
slice→arc bubble-up ran cleanly on first use (one convention normalized for the rows to
come). On CI green, A-1 flips `open → done`; slice02 (cold-path build) can fill the
snapshot.

CDC: planning thread, 2026-06-26. Iterations used: 1.
