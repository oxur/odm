# Closing report — Arc 04 / Slice 02: Cold-path build

> CC implementation closing report. Status: **proposed-done** (`attested`) → CDC
> reproduces the cargo rows via CI / a local 1.85+ toolchain (`attested →
> reproduced`). Impl commit `d03d5d0`; docs commit (this report + ledger +
> arc-plan bubble-up) follows.

## What this slice built

The first (cold) run: a full corpus walk that fills the snapshot slice01 defined.
`build_records` produces one fully-populated `IndexRecord` per node file — stat
from `lstat`, `content_hash` over the raw bytes, metadata from the parsed
`Document`, edges mapped to the index's `EdgeRef` (with qualifiers), and a
deterministic `meta_hash` — and `build` assembles them with an `index_timestamp`
into a `Snapshot` persisted via slice01's `persist`. The O(corpus) pass paid
once; the incremental warm path is slice03.

## Per-row ledger walk (9 rows)

- **B-1 — done (attested).** `Store::node_paths()` factored out of `load_all`;
  the cold build reuses it (no re-derived `nodes/YYYY/MM` traversal). One record
  per file; missing/empty `nodes/` → empty set. `cold_build_one_record_per_file`
  + `cold_build_empty_corpus` green.
- **B-2 — done (attested).** Stat fields from `symlink_metadata` (lstat): size,
  mtime (portable `modified()`), inode/mode (Unix). `cold_build_stat_fields`
  compares against an independent stat.
- **B-3 — done (attested).** `content_hash` = SHA-256 of the raw bytes, verified
  against an independent hash of the on-disk file.
- **B-4 — done (attested).** Metadata from the document: node_type, `gates`
  (reached-gate set, gate-name sorted), tags, title, updated.
- **B-5 — done (attested).** All edge kinds mapped, with qualifier fidelity (see
  decision below). Two tests cover all-kinds + the `Updates`/qualified-tear arms.
- **B-6 — done (attested).** `meta_hash` over a canonical, sorted encoding of the
  semantic metadata; deterministic across runs.
- **B-7 — done (attested).** `build` stamps `index_timestamp = now` and persists
  via slice01's `Snapshot::persist` (no reimplemented writer).
- **B-8 — done (attested).** Built → persist → load round-trips to identical
  records as `Load::Loaded`.
- **B-9 — done (attested).** clippy `-D warnings` → exit 0; no `unsafe`; line
  coverage **98.68%**; `fmt --check` clean; full workspace `cargo test` → 246
  passed; odm-store suite green after the `node_paths` refactor.

## Decisions / deviations flagged (not buried)

1. **B-5 — `EdgeRef` enriched to full qualifier fidelity (deliberate, recorded).**
   The slice-doc posed: preserve `depends_on.satisfied_at` / supersede-kind / tear
   `because`, or just `(target, kind)`? I **enriched** `EdgeRef` with
   `qualifier: Option<EdgeQualifier>`. Rationale: the arc's capability is "feed
   `list`/`orient`/graph-build so they stop re-walking." Index-backed *satisfaction*
   needs `satisfied_at`; `orient`'s active-tears listing needs `because`. Without
   the qualifier, slice04 would have to re-read frontmatter for those — a
   half-measure that defeats the arc. This **extends slice01's `EdgeRef`** (a
   CDC-verified type); I kept **`FORMAT_VERSION = 1`** because no on-disk index can
   exist yet (the crate is wired into no command — the first persist is slice04 or a
   future build command), so there is no v1 file to break. **If any command persists
   an index before slice04 lands, this must become a version bump.** Flagged for CDC.
2. **B-6 — `meta_hash` field set excludes `updated` and the stat fields.** The
   meta-hash is the *meaning* fingerprint slice05's early cutoff compares; it covers
   `node_type, gates, tags, edges, title` (tags+edges sorted for canonicality, gates
   already sorted). Including `updated` would couple "meaning changed" to a
   bookkeeping date and weaken early cutoff (a body-only edit must leave `meta_hash`
   unchanged). A deliberate, documented choice — slice05 consumes it.
3. **`Store::node_paths()` added to odm-store** (a refactor of a prior-arc crate).
   It is a pure factoring of `load_all`'s existing walk; `load_all` now parses what
   it returns and keeps its exact contract (sorted-by-id, missing-dir → empty,
   first-error propagates). odm-store's suite stayed green. Sanctioned by the
   cc-prompt ("factor a path-yielding helper out of `load_all`").
4. **`mtime` via portable `modified()`, inode/mode Unix-only.** Stat fields use
   `symlink_metadata`; mtime is split from the portable `SystemTime`, while
   inode/mode use `MetadataExt` under `#[cfg(unix)]` and fall back to `0` elsewhere
   (the index treats them as opportunistic — never a lone correctness signal,
   ODD-0014 §4). The crate compiles on non-Unix; it is exercised on Unix.

## Uncertainties / things CDC should look at

- **`rel_path` separators.** `rel_path` is the `strip_prefix`'d path via
  `to_string_lossy` — on Unix it is `nodes/YYYY/MM/<ULID>.md` with `/`. On Windows
  it would carry `\`. Since the project targets darwin/linux this is fine, but if
  cross-platform index portability ever matters, normalize to `/`. Flagged.
- **`meta_hash` field-set ratification.** The exclude-`updated` choice (decision 2)
  is mine to make per the ledger ("CC's choice"); slice05 depends on it. Worth a CDC
  nod before slice05 builds early-cutoff on top of it.
- **Two genuinely hard-to-reach error arms remain uncovered** (`BuildError::Read`,
  `::Stat`) — they need a file that `node_paths` lists but that then fails to
  read/stat (a permissions/race case). The `Parse` and `Utf8` arms *are* covered by
  the two malformed-file tests. Crate line coverage is 98.68%.
- **The slice01 `CountMismatch` guard** (`snapshot.rs`) remains the one uncovered
  line there (unchanged from slice01; unreachable via the public API).

## Bubble-up to the arc (LEDGER-DISCIPLINE v2.0 §A / PM Part IV)

- **Did slice02 deliver its assigned piece of the A4 capability?** Yes. The arc
  needs a "first run [that] pays a full scan + hash + parse and persists." This
  slice is exactly that cold path: walk → stat+hash+parse → records → persisted
  snapshot, reusing slice01's format. It deliberately does **not** do warm-path
  change detection (slice03), filter/sort maps (slice04), or early-cutoff
  consumption (slice05) — its assigned scope, no more.
- **What did it reveal the arc-plan didn't anticipate?**
  - The **`EdgeRef` qualifier-fidelity** question (slice-doc flagged it as open).
    Resolved by enriching `EdgeRef` — **arc-plan input for slice04**: graph-build
    can read satisfaction (`satisfied_at`) and tear rationale (`because`) straight
    from the index; it does *not* need to re-read frontmatter for ordering/satisfaction.
    This strengthens A-10 (index-backed graph-build matches the baseline).
  - The **`meta_hash` field set** (excludes `updated`/stat) is now fixed —
    **arc-plan input for slice05**: early cutoff compares this exact set; a body-only
    edit (content_hash differs, meta_hash same) recomputes nothing.
  - A **postcard discipline**: `skip_serializing_if` desyncs a non-self-describing
    format. The index's record/format must keep every field always-serialized; any
    future "optional" field needs a format-version bump, not a skip. Worth a line in
    the arc's format-evolution notes.
- **Slice-scale silent-drop diff (scope-as-specified vs. scope-as-delivered):**
  none. Every "In" item in `slice-doc.md` shipped (walk, all stat fields,
  content_hash, all metadata fields, edge mapping across all kinds, deterministic
  meta_hash, assemble + persist); every "Out" item (warm path, filter/sort maps,
  early-cutoff consumption, benchmark) was held out. No row softened; 9 opened, 9
  dispositioned.

## Iterations

One. In-slice corrections before close: the `skip_serializing_if` postcard desync
(removed; regression seed committed), a clippy `sort_by_key` lint, and two added
tests (the `Updates`/qualified-tear arms + the malformed/UTF-8 error paths) that
lifted coverage to 98.68%.
