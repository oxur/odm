---
number: 14
title: "Research — odm-index: incremental indexing & caching (no DB, no FTS)"
author: "topological sort"
component: All
tags: [research, index, cache, incremental, change-detection, performance]
created: 2026-06-20
updated: 2026-06-20
state: Final
supersedes: null
superseded-by: null
version: 1.0
---

# Research — odm-index: incremental indexing & caching (no DB, no FTS)

_Cited research pass for the `odm-index` crate: a read-acceleration "mini-infra"
layer, distinct from storage (read/write of individual node files) and from the
user-facing rollup (a generated view). Its job: at startup, know which files
define projects/arcs/slices, support fast metadata filtering/sorting, and feed
the graph build — without re-walking and re-parsing the whole corpus every run.
Compiled 2026-06-20 from five parallel primary-source research streams
(git stat-cache & racy-git; build-system change detection; Rust serialization &
atomic persistence; Build Systems à la Carte & incremental computation;
filesystem watching). Every load-bearing claim is tagged **[E]**
empirical/primary-doc or **[P]** practitioner-lore. The racy-git mechanism and
all performance claims were adversarially cross-checked against two independent
primary sources each; §1 records what survived that check and what did not._

---

## 0. The convergent finding (the one that matters)

Every mature system that must answer "what changed since last time?" cheaply and
**correctly** — git, Mercurial, jj, Make, Ninja, Bazel, Buck2, Turborepo — has
independently converged on the same shape, and it is one we can build with
nothing but the OS filesystem and our own code:

> **A persisted, sorted, derived "stat cache": one record per tracked file
> holding `(path, identity, lstat fields, content-hash, extracted metadata)`.
> Change detection is a cheap `lstat`-compare against the cached record; the
> content hash is the authority, consulted only when the stat result is
> ambiguous. The cache is rebuildable from source, written atomically
> (temp + rename), and validated, never trusted, by a watcher.**

This is precisely git's index (`DIRC`) [E], Mercurial's `dirstate` [E], and jj's
`TreeState` [E]. The *first* run pays a full scan + hash and persists the cache;
*subsequent* runs `lstat`-compare and touch only the delta — exactly the
FIRST-full / SUBSEQUENT-incremental behavior odm requires.

The single most important correctness lesson is the **"racy git" problem**: a
stat-only check **silently misses** an in-place edit that lands in the same
mtime tick as the cache write and does not change file size. The mitigation is
universal and non-negotiable: record file **size** as a second stat signal, and
**content-hash** any entry whose mtime is `>=` the cache's own timestamp. Skip
this and odm will, rarely but really, serve a stale graph. This is the one place
where "stat-only" is a correctness bug, not a performance tradeoff [E].

The second structural lesson, from *Build Systems à la Carte* (which ODD-0011
already cites), is the clean split: a **change detector** (the rebuilder's
decision: is this file's derived data still valid?) is orthogonal to a
**scheduler** (the order in which the graph/rollup is recomputed). `odm-index`
should own the former and feed the latter. **Early cutoff** — if a changed file
re-parses to byte-identical metadata, downstream graph/rollup work is skipped —
is the cheapest large win available and falls out naturally from hashing derived
metadata, not just source bytes [E].

The corpus sizes in scope (10k–100k+ small markdown files) are well within reach
of a single snapshot file rewritten on each mutation; sharding is a
**later** optimization triggered by measured load latency, not a day-one
requirement [E/P]. No DB and no FTS engine is required to hit these goals, and
this document argues explicitly (§4) about what that costs us.

---

## 1. Evidence strength: what to trust vs. lore

The house style demands honesty about calibration. Here is what is
well-grounded versus convention.

**Rock-solid [E] (primary docs, cross-verified):**

- **The git index records `ctime`, `mtime` (each as 32-bit secs + 32-bit nsec
  fractions), `dev`, `ino`, `mode`, `uid`, `gid`, a 32-bit-truncated `size`, and
  the object hash** — verbatim from git's `index-format` documentation [E].
- **The racy-git rule: an entry is "racily clean" iff its `mtime >=` the index's
  own timestamp, and such entries must be content-verified.** Confirmed by
  *two* independent primary sources: git's `racy-git.txt` ("st_mtime is the same
  as (or newer than) the timestamp of the index file itself ... it also compares
  the contents") and gix's `Stat::is_racy` rustdoc ("racy if its mtime is larger
  or equal to the index timestamp ... will need to be examined ... by actually
  reading the file from disk") [E]. **Agreement is exact (`>=`).**
- **Size is the primary backstop for the racy case; when size also fails
  (same-size in-place edit), git additionally zeroes the recorded size on the
  next index write to force a permanent cheap mismatch** [E, git racy-git.txt].
- **Bazel checks up-to-dateness by content checksum, using mtime as a shortcut
  gated on ctime** ("we don't checksum the file if its ctime hasn't changed") —
  verbatim, Bazel's maintainer-authored codebase doc [E]. This is the inverse of
  Make's model and validates "hash is authority, stat is the fast filter."
- **Make is purely mtime-ordinal, no content awareness** [E]. **Ninja adds a
  build log so a changed command line forces a rebuild, and `restat` gives early
  cutoff** [E]. **Mercurial's dirstate forces a content lookup when a file could
  have changed within the same one-second tick as the dirstate write** [E] — an
  independent re-derivation of the racy mitigation, dirstate-v2 even matching
  nanosecond precision and flagging ambiguity explicitly.
- **rename(2) is atomic w.r.t. the destination; durability additionally requires
  fsync of the file *and* the containing directory** — POSIX/Linux man pages
  [E]. `tempfile::NamedTempFile::persist()` does the atomic rename but **no
  fsync** [E].
- **rkyv 0.7 archives are not readable by 0.8; format is stable only within a
  major** [E]. **postcard has a documented, versioned, stable wire format since
  1.0** [E]. **serde_cbor and bincode-v1's GitHub repo are archived** [E].
- **Build Systems à la Carte**: build = scheduler × rebuilder; verifying vs.
  constructive traces; early cutoff; minimality and correctness definitions —
  all verbatim from the ICFP 2018 paper with page numbers [E].
- **inotify(7) itself prescribes rebuilding the cache on inconsistency/overflow;
  kqueue needs one open fd per watched object; FSEvents flags `MustScanSubDirs`
  on dropped events; git fsmonitor is a speed hint validated against the index**
  — all primary man pages / git docs [E].

**Conventional / practitioner [P] (true in practice, not formally guaranteed):**

- "A single snapshot file is fine up to tens of MB / hundreds of ms load" — a
  reasonable engineering threshold, not a measured odm benchmark [P].
- "kqueue doesn't scale to many files" — follows directly from the fd-per-object
  design (which *is* [E]), but the "doesn't scale" framing is consensus [P].
- Choice of postcard vs. bincode-v2 vs. rkyv is an engineering judgment; the
  *facts* about each format are [E], the *recommendation* is [P].

**Corrections to the brief's premises (negative findings) [E]:**

- **`core.trustmtime` does not exist.** Only `core.trustctime` exists in git's
  `core.adoc`. The "trust mtime" behavior is governed by the compile-time
  `USE_NSEC` option and defaults (off on Linux), not a config knob [E].
- **On Linux, git does *not* compare nanosecond mtime by default** (`USE_NSEC`
  off) because evicting/reloading an inode can perturb the in-core sub-second
  time, causing false "modified" reports [E]. gix mirrors this (`use_nsec`
  defaults `false`). This is counter-intuitive and load-bearing: **we cannot
  lean on nanosecond mtime for correctness.**
- **memmap2's own docs say "Undefined Behavior," not "SIGBUS"** — the
  SIGBUS-on-truncation mechanism is documented in POSIX `mmap(2)`, not the crate.
  Cite the right source.

**Source-quality caveats to verify before relying further:**

- The canonical `racy-git.txt` raw mirrors (kernel.org, raw.githubusercontent)
  returned empty bodies; the text was obtained from the git-scm.com rendered
  page, which reproduces the file verbatim. The mechanism is nonetheless
  double-confirmed by gix's independent rustdoc.
- rkyv's `validation.html` timed out twice; validation claims rest on the rkyv
  FAQ + crate docs. The rkyv 0.8 version-stability claim came partly via search
  snippets of release notes — **verify directly before committing to rkyv.**
- Bazel's `--guard_against_concurrent_changes` default semantics came partly via
  a search snippet — treat the *default* as [P] until checked.
- Apple's live FSEvents developer docs are JS-gated and could not be fetched;
  FSEvents claims rest on notify's docs citing Apple + Apple's `FSEvents.h`
  header text via mirror.

---

## 2. Design principles mapped to odm's requirements

### 2.1 FIRST-full / SUBSEQUENT-incremental → the stat-cache pattern

Every VCS working-copy scanner solves exactly odm's problem and solves it the
same way: persist per-file `lstat` results so the next run can skip work.

- **git index** stores `ctime/mtime/dev/ino/mode/uid/gid/size + object hash` per
  path; `git status` first `lstat`s, compares against the cached stat, and "can
  tell that the files are modified without even looking at their contents" if any
  field differs [E].
- **Mercurial dirstate** stores `state, mode, size, mtime, name` per file [E].
- **jj TreeState** "keeps track of the mtime and size for each tracked file" and
  `snapshot()` "will use the recorded mtimes and sizes and detect changes" [E].

The mapping to odm:

| Run | odm-index behavior | Cost |
|-----|--------------------|------|
| First (cold) | `walkdir` the `nodes/` tree, `lstat` + read + hash + parse frontmatter for every file, build records, persist atomically | O(corpus) — pay once |
| Subsequent (warm) | load persisted cache (one read), `lstat` each candidate, compare, re-hash/re-parse only ambiguous or changed entries, detect deletions, persist if changed | O(delta) for changed files + O(corpus) for the cheap `lstat` sweep |

Note the warm path still does one `lstat` per file (cheap, ~µs) unless an
optional watcher (§5) supplies a dirty-set. That `lstat` sweep is the
correctness floor and is what git's `core.preloadIndex` parallelizes [E].

### 2.2 No DB / no FTS → snapshot file + in-memory indexes

The hard constraint (filesystem + our tool only) is satisfied by exactly what
git does: a **single sorted binary file**, plus **in-memory maps** built on
load. This is not a compromise — git, Mercurial, and jj run global-scale
workflows on flat index files, no embedded DB. What we forgo (transactions,
concurrent multi-writer, SQL ad-hoc queries) we do not need: odm is a
single-user CLI whose index is a *derived cache*, rebuildable on demand. §4
states the costs honestly.

### 2.3 The racy-git correctness lesson → size + conditional hash

Spelled out because it is the one subtle bug:

The naive check "mtime unchanged ⇒ file unchanged" fails when an edit lands in
the same mtime granularity tick as the last cache write **and** leaves size
unchanged. git's defense, which we adopt verbatim [E]:

1. Compare **size** as well as mtime — most edits change size, caught cheaply.
2. For any entry whose **`mtime >= cache_timestamp`** ("racily clean"), do not
   trust stat — **read and hash** the file and compare to the recorded hash.
3. The number of racily-clean entries is naturally tiny (only files touched in
   the same tick as the last write), so the hash fallback is bounded. git
   measured ~2.2s vs ~0.14s for `diff-files` over ~20k racily-clean entries [E]
   — i.e., the fallback is expensive *per entry* but rarely triggered, which is
   why bounding the racy set matters.
4. Do not depend on nanosecond mtime for correctness (Linux default-off,
   network/exotic FS unreliable) [E]. Treat whole-second mtime + size as the
   fast signal and the content hash as the authority.

Because odm already reads + parses frontmatter to build records, **we will
already have the file bytes in hand on any cold/changed path** — so computing the
content hash is nearly free there, and the hash doubles as the early-cutoff
fingerprint (§2.5). The only "extra" work the racy rule imposes is hashing the
small set of racily-clean entries on the warm path.

### 2.4 Dependency-aware invalidation → rebuilder × scheduler (BSàlC)

ODD-0011 already cites *Build Systems à la Carte*; `odm-index` is where its
abstractions become concrete [E]:

- A **rebuilder** decides whether a key's value is still valid. `odm-index`'s
  per-file change detection *is* a rebuilder over the "parse this file" task. The
  cache record (stat + hash + parsed metadata) is a **verifying trace**: store
  the hash of the input and of the derived value; recompute only on mismatch.
- A **scheduler** sequences recomputation in dependency order. The graph build /
  topological sort is the scheduler. `odm-index` feeds it the dirty set.
- **Early cutoff** [E]: when a file changes but re-parses to identical metadata
  (e.g., a typo fixed in the prose body, no frontmatter change), the derived
  metadata hash is unchanged, so the graph and rollup need not be recomputed.
  This is the same mechanism as Salsa's "backdating" [E] and Ninja's `restat`
  [E]. It requires fingerprinting the *derived* metadata, not just source bytes.
- **Verifying vs. constructive traces** [E]: a verifying trace (store hashes,
  recompute on mismatch) is the right fit — it keeps the cache small and needs no
  shared value store. Constructive traces (store the value for cross-machine
  cache sharing, à la Bazel/CloudBuild) are overkill for a single-user CLI.
- **Reverse-edge invalidation**: when a node changes, the graph's
  backlink/reverse-edge map identifies which downstream derived artifacts (rollup
  sections, dependency closures) are affected. The production pattern (Salsa,
  per rust-analyzer's author) is **lazy** on-demand revalidation — forward-flood
  to inputs, then backward-flood recomputation that stops at the first unchanged
  result (early cutoff) — rather than eager reverse-dep marking, which wastes
  work under bursty edits [E/P]. For odm's batch-CLI usage, a simpler eager
  recompute of the affected closure is acceptable and easier to reason about;
  the Salsa pattern is the optimization if interactive latency ever matters.

### 2.5 Memoization / fingerprinting

The cache record carries two fingerprints: the **content hash of the source
file** (input fingerprint, for change detection per §2.3) and the **hash of the
extracted metadata** (derived fingerprint, for early cutoff per §2.4). Storing
both lets odm answer "did this file change?" and "did its *meaning* change?"
independently — the former gates re-parsing, the latter gates graph/rollup
recompute.

---

## 3. Recommended design for `odm-index`

A concrete, constraint-respecting design. Treat the numbers as defaults to
validate, not gospel.

### 3.1 Index record shape

One record per tracked node file:

```
IndexRecord {
  // identity
  id:            Ulid,          // == filename stem; canonical key
  rel_path:      String,        // nodes/YYYY/MM/<ULID>.md, for I/O & deletion detection

  // change-detection (the stat cache; mirrors git/hg/jj)
  mtime_secs:    i64,           // whole-second mtime  (do NOT rely on nsec for correctness)
  mtime_nsec:    u32,           // recorded for completeness / opportunistic compare only
  size:          u64,           // full u64 (not git's 32-bit truncation — we have no format reason to truncate)
  inode:         u64,           // optional dev/ino; helps detect rename vs. edit; skip on network FS
  mode:          u32,           // file type / exec bit

  // fingerprints
  content_hash:  [u8; 32],      // hash of raw file bytes (input fingerprint)
  meta_hash:     [u8; 32],      // hash of normalized extracted metadata (derived fingerprint, early cutoff)

  // extracted metadata for in-memory filter/sort (no re-parse needed)
  node_type:     NodeType,      // project / arc / slice / odd / adr / ...
  state:         State,         // lifecycle state / gate
  tags:          Vec<String>,
  edges:         Vec<EdgeRef>,  // dependency edges (id + kind) for graph build
  title:         String,
  created:       Ulid-derived,  // from id
  updated:       Date,
}
```

Plus a small file **header**: magic bytes, **format version**, **hash algorithm
id**, the **index timestamp** (for the racy `>=` test), record count, and a
trailing **checksum** over the body (git ends its index with a checksum [E];
corrupt ⇒ rebuild). The header's format-version field is what makes "corrupt or
stale ⇒ rebuild" cheap and safe, and is mandatory if we ever choose rkyv (whose
format breaks across majors [E]).

Hash choice: a fast non-cryptographic hash (e.g., a 128-bit xxh3) is sufficient
for a *local, derived* cache — we are detecting accidental change, not defending
against adversarial collisions, and the cache is rebuildable. (If we ever want
git-object compatibility we would use the repo's SHA-1/SHA-256, but that is a
separate concern from the index.) [P]

### 3.2 Change-detection algorithm (the warm path)

```
load index (one read; verify header version + checksum; on mismatch → full rebuild)
candidates = current set of node files (from walkdir, OR from optional watcher dirty-set §5)

for each candidate path:
    st = lstat(path)
    rec = index.get(id_from_path(path))
    if rec is None:                      → NEW: read+hash+parse, insert
    elif st.size != rec.size
         or st.mtime_secs != rec.mtime_secs
         or st.mode != rec.mode:         → CHANGED (cheap signal): read+hash+parse, update
    elif st.mtime_secs >= index.timestamp:   // RACILY CLEAN — stat cannot be trusted
         h = hash(read(path))
         if h != rec.content_hash:       → CHANGED: re-parse, update
         else:                           → clean (but consider re-stamping; see note)
    else:                                → CLEAN: skip (the fast majority)

for each id in index not seen above:     → DELETED: remove record

if any change:
    set index.timestamp = now (just before write)
    optionally zero the recorded size of any still-racy entries   // git's same-size-edit defense [E]
    persist atomically (§3.3)

on any change to a node's meta_hash:      → mark its graph/rollup closure dirty (§3.4)
```

Notes:
- The `>=` racy test and the size+mtime+mode cheap comparison are taken directly
  from git/gix [E]. The set entering the hash branch is small by construction
  (only files touched in the same tick as the last write).
- **Deletion detection** is free: any cached id not present on disk this run is
  gone. (git/hg do the same via the index being the authoritative file list.)
- **Rename** (same content, new path/id) is detectable via `(inode, content_hash)`
  but odm's filenames *are* ids, so a "rename" is really a delete + create of
  distinct nodes; inode is mostly a diagnostic aid here.
- The same-size-edit defense (zeroing recorded size of racy entries [E]) is the
  belt-and-suspenders against the worst case; it is cheap and worth adopting.

### 3.3 Persistence format & atomic write

**Recommended default: a single sorted snapshot file, serialized with `postcard`
or `bincode 2` (controlled writer + reader, both pure Rust), with a versioned
header + trailing checksum, written via temp-file + atomic rename.**

Rationale, from the format research [E]:

- **Why snapshot, not append-log:** the index is *derived and rebuildable*, so
  the crash-safety bar is low (corrupt ⇒ rebuild). Snapshot is the
  lowest-complexity correct design — no tombstones, no compaction/GC, every read
  is one sequential load, the file is always internally consistent via atomic
  rename. Append-only (Bitcask-style) only pays off for a long-lived process
  doing many tiny incremental writes — not a CLI [E]. At 10k–100k small records
  the snapshot is single-digit MB; a full rewrite costs milliseconds, so write
  amplification is "mostly irrelevant unless writes are high-frequency" [E].
- **Why postcard / bincode 2 over the alternatives:**
  - **postcard** — most compact (varint, no field names), documented *stable
    versioned* wire format since 1.0, pure Rust, no_std. Strongest stability
    story of the serde binary formats [E].
  - **bincode 2** — config-tunable, native (non-serde) derive macros, pure Rust.
    Pin the major version *and* the config (the wire format is config-dependent
    and v1 ≠ v2) [E]. Its GitHub repo is archived (dev moved to sourcehut) —
    verify ongoing maintenance before committing [E].
  - **rkyv 0.8 + mmap** — the *only* total-zero-copy option; load is a pointer
    cast, no parse, scales sub-linearly with data; pairs with mmap so only
    touched pages fault in [E]. **Defer unless load latency is measured to
    dominate.** Costs: two layered `unsafe` preconditions (valid archive +
    file-not-mutated-under-map, the latter being literal UB per memmap2 [E]),
    and a hard format break on every rkyv major upgrade [E] (mitigated by our
    version header + rebuild, but real). Our atomic-rename design actually makes
    mmap safe-ish: never mutate in place; write a new file + rename; readers
    holding an old mmap keep seeing the old (now-unlinked) inode.
  - **serde_json / NDJSON** — choose only if human-readable debuggability of the
    cache outweighs size/speed; least compact (text + field names) [E]. A
    reasonable *debug* dump format even if the primary format is binary.
  - **Avoid:** serde_cbor (archived 2021 [E]); CBOR/MessagePack self-describing
    overhead buys us nothing for a private cache we both write and read.
- **Atomic write sequence** [E]:
  1. write temp file **in the same directory** as the target (avoid `EXDEV`);
  2. `sync_all()` the temp file (durability — `persist()` does *not* fsync [E]);
  3. atomic rename temp → `index` (POSIX-atomic replace);
  4. fsync the containing directory (persists the rename itself — rename
     atomicity ≠ durability [E]).
  Steps 2 and 4 are skippable if we accept "crash ⇒ rebuild" (we do, since the
  cache is derived) — but they are cheap insurance against a torn/empty index.
  Use `tempfile::NamedTempFile::persist()` for step 3, adding the fsyncs
  ourselves.

- **Sharding (defer):** stay single-file until per-invocation load/parse is
  user-perceptible (~hundreds of ms / tens of MB) *or* per-write rewrite cost
  bites. Then shard a *snapshot* index by **id-prefix** (the ULID's leading
  bytes are time-sortable → natural range shards aligned with the `YYYY/MM`
  layout), so a write rewrites only the affected shard (~1/N amplification)
  without going append-only [E]. Hash-sharding for uniform point lookups; the
  prefix scheme is the better fit for odm's time-ordered ids. Pick a **fixed**
  shard count up front; never `hash mod N` (reshuffles everything on count
  change) [E].

### 3.4 Invalidation strategy

- The index holds the per-file **verifying trace** (content_hash + meta_hash).
- On a metadata change (meta_hash differs), mark dirty the node and — via the
  graph's reverse-edge map — its dependents' derived artifacts (graph closure,
  rollup sections). On a body-only change (content_hash differs, meta_hash same)
  → **early cutoff**: update the cache record, recompute *nothing* downstream
  [E]. This is the single highest-value behavior the design enables.
- For odm's batch CLI, eager recompute of the affected closure is acceptable
  (simpler than Salsa's lazy forward/backward flooding). Adopt the lazy pattern
  only if interactive latency becomes a goal [E/P].

### 3.5 Filter / sort / search without an FTS dependency

- **Metadata filtering & sorting** (by type / tag / state-gate / edge): build
  **in-memory indexes on load** from the records that are already in the cache —
  `HashMap<NodeType, Vec<Ulid>>`, `HashMap<Tag, Vec<Ulid>>`,
  `HashMap<State, Vec<Ulid>>`, and adjacency/reverse-adjacency maps for edges.
  At 10k–100k records these are a few MB and build in milliseconds. Sorting is a
  `sort_by` over the relevant field already present in each record. **No DB, no
  index engine needed** — the whole point of caching the extracted metadata is
  that filter/sort never touches disk after load.
- **Text search over bodies** is the only case that might tempt an FTS engine.
  Argue it explicitly:
  - For 10k–100k files, a **linear scan** (grep-style, optionally over an mmap or
    the already-read bytes) is simple, dependency-free, and fast enough for an
    on-demand `odm search` — hundreds of MB/s; sub-second for typical corpora
    [P]. This is the recommended default.
  - A **hand-rolled minimal inverted index** (token → posting list of ids,
    persisted as another derived shard) is warranted *only* if interactive
    body-search latency over a very large corpus becomes a real requirement.
    Build it from the same parse pass that fills the cache; invalidate per-file
    on content_hash change. This stays within "our tool only" — it is our code,
    not a Lucene/Tantivy dependency.
  - **Why avoid general FTS/DB engines:** they violate the hard constraint
    (heavy deps, often C, sometimes a daemon), add a second source of truth to
    keep consistent, and solve problems (ranking, fuzzy, multi-user concurrency,
    millions of docs) odm does not have. What we *give up* is out-of-the-box
    relevance ranking, stemming, and large-scale full-text speed — acceptable for
    a single-user planning tool whose primary access is structured-metadata
    filtering, not prose retrieval (§4).

---

## 4. What the evidence does NOT support / guardrails

- **Stat-only is not a safe shortcut for correctness.** The racy-git evidence is
  unambiguous: without size + conditional content-hash, a real (rare) class of
  edits is missed [E]. Do not ship a stat-only fast path that skips the `>=`
  racy test. This is the cost of the "minimal infra" ethos paid back in care.
- **Nanosecond mtime is not a correctness signal.** Linux default-off, exotic/
  network FS unreliable [E]. We may use it opportunistically as an *extra* cheap
  dirty signal, never as proof of cleanliness.
- **A watcher cannot be the source of truth.** inotify(7) *itself* says rebuild
  the cache on overflow/inconsistency; kqueue needs an fd per file; FSEvents
  drops events and flags `MustScanSubDirs`; none survive tool restarts or
  network FS; git fsmonitor is explicitly a validated *hint* [E]. Correctness
  must rest on the stat-walk; the watcher is pure acceleration (§5).
- **No-DB costs us real things, honestly:** no transactions or safe concurrent
  multi-writer access to the index (mitigate: single-writer CLI + atomic rename +
  "corrupt ⇒ rebuild"); no ad-hoc query language (mitigate: the metadata we
  actually filter on is pre-extracted into typed maps); a full snapshot rewrite
  per mutation has write amplification that *will* matter if odm ever does many
  rapid small writes in one process (mitigate: batch writes per command; shard
  later). These are acceptable for the stated use, but they are not free.
- **No-FTS costs us:** no ranking, stemming, fuzzy, or sub-100ms full-text over
  very large corpora. Linear scan is "fast enough" is a [P] claim for the
  expected sizes — **validate with a benchmark on a synthetic 100k-node corpus
  before declaring victory.**
- **rkyv is attractive but not yet justified.** Its zero-copy win is real [E],
  but it adds layered `unsafe` and per-major format breaks, and we have **no
  measurement** that load latency dominates. Start with postcard/bincode-2;
  revisit rkyv only if profiling demands it. The version-stability detail came
  partly via search snippets — verify before adopting.
- **Performance numbers herein are mostly thresholds, not odm measurements.**
  The "single file fine to ~tens of MB," "linear search fast enough," and
  "snapshot rewrite is milliseconds" claims are [P] engineering judgments. The
  *first* implementation milestone should include a benchmark harness over
  synthetic 1k / 10k / 100k corpora to convert these [P] claims to [E].
- **`core.trustmtime` does not exist; do not design around it.** Only
  `core.trustctime` is real [E]. The analogous odm knob, if any, would be
  "ignore ctime differences" for environments where crawlers/backups touch ctime
  (the documented reason git added `core.trustctime`) [E].

---

## Sources

Primary docs are tagged where load-bearing. URLs as fetched 2026-06-20.

**Git stat-cache & racy-git**
- Git — index-format Documentation. https://git-scm.com/docs/index-format [E]
- Git — racy-git Documentation. https://git-scm.com/docs/racy-git [E]
- Git — core configuration (`core.adoc`: trustctime, checkStat, fsmonitor, preloadIndex).
  https://raw.githubusercontent.com/git/git/master/Documentation/config/core.adoc [E]
- gix-index crate (gitoxide): root, `entry::Stat` (`is_racy`, `matches`),
  `entry::stat::Options`. https://docs.rs/gix-index/latest/gix_index/ [E]
- git-fsmonitor--daemon(1). https://git-scm.com/docs/git-fsmonitor--daemon [E]
- git-update-index(1) (FILE SYSTEM MONITOR, fsmonitor-valid). https://git-scm.com/docs/git-update-index [E]

**Build & VCS change detection**
- GNU Make / make(1) man page. https://www.man7.org/linux/man-pages/man1/make.1.html [E];
  How Make Works (manual). https://www.gnu.org/software/make/manual/html_node/How-Make-Works.html [E, page blocked; man page substituted]
- The Ninja build system manual (restat, .ninja_log). https://ninja-build.org/manual.html [E]
- The Bazel codebase (artifact up-to-dateness, checksum + mtime/ctime shortcut, action cache).
  https://bazel.build/versions/8.0.0/contribute/codebase [E]
- Bazel command-line reference (`--guard_against_concurrent_changes`). https://bazel.build/reference/command-line-reference [E/P]
- Why Buck2; Buck2 dep files. https://buck2.build/docs/about/why/ , https://buck2.build/docs/rule_authors/dep_files/ [E]
- Turborepo caching (global/task hash, fingerprints). https://turborepo.dev/docs/crafting-your-repository/caching [E]
- Mercurial — internals.dirstate-v2 (mtime/size, HAS_MTIME, nanosecond/ambiguity flags).
  https://repo.mercurial-scm.org/hg/file/tip/mercurial/helptext/internals/dirstate-v2.txt [E, canonical blocked; faithful hgweb mirror used]; DirState wiki (v1 format). https://www.mercurial-scm.org/wiki/DirState [E]
- Jujutsu — Architecture (TreeState, snapshot), Working copy, Configuration (fsmonitor/watchman).
  https://docs.jj-vcs.dev/latest/technical/architecture/ , https://docs.jj-vcs.dev/latest/working-copy/ , https://docs.jj-vcs.dev/latest/config/ [E]

**Rust serialization & atomic persistence**
- bincode 1.3.3 / 2.0.1 docs & config. https://docs.rs/crate/bincode/1.3.3 , https://docs.rs/bincode/2.0.1/bincode/ [E]
- postcard docs & wire-format spec. https://docs.rs/postcard/latest/postcard/ , https://postcard.jamesmunns.com/wire-format.html [E]
- ciborium (CBOR). https://docs.rs/ciborium/latest/ciborium/ [E]; serde_cbor (deprecated). https://docs.rs/serde_cbor/latest/serde_cbor/ [E]
- serde_json + StreamDeserializer; JSON Lines. https://docs.rs/serde_json/latest/serde_json/ , https://jsonlines.org [E]
- rmp-serde (MessagePack). https://docs.rs/rmp-serde/latest/rmp_serde/ [E]
- rkyv — zero-copy, FAQ, validation, releases. https://rkyv.org/zero-copy-deserialization.html , https://rkyv.org/faq.html , https://github.com/rkyv/rkyv [E; validation.html timed out]
- memmap2 Mmap/MmapMut safety. https://docs.rs/memmap2/latest/memmap2/struct.Mmap.html [E]
- POSIX rename. https://pubs.opengroup.org/onlinepubs/9799919799/functions/rename.html ; Linux rename(2). https://man7.org/linux/man-pages/man2/rename.2.html ; fsync(2). https://man7.org/linux/man-pages/man2/fsync.2.html ; mmap(2). https://man7.org/linux/man-pages/man2/mmap.2.html [E]
- LWN "Ensuring data reaches disk" (Moyer, 2011). https://lwn.net/Articles/457667/ [P]
- Pillai et al., "All File Systems Are Not Created Equal," OSDI'14. https://www.usenix.org/conference/osdi14/technical-sessions/presentation/pillai [E]
- Crotty/Leis/Pavlo, "Are You Sure You Want to Use MMAP in Your DBMS?" CIDR'22. https://db.cs.cmu.edu/papers/2022/cidr2022-p13-crotty.pdf [E]
- Sheehy & Smith, "Bitcask," 2010. https://riak.com/assets/bitcask-intro.pdf [E]
- tempfile NamedTempFile::persist. https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html [E]

**Incremental computation**
- Mokhov, Mitchell, Peyton Jones, "Build Systems à la Carte," ICFP 2018.
  https://www.microsoft.com/en-us/research/wp-content/uploads/2018/03/build-systems.pdf , https://doi.org/10.1145/3236774 [E]
- Salsa book: red-green algorithm, maybe_changed_after, backdate, durability.
  https://salsa-rs.github.io/salsa/reference/algorithm.html (and /plumbing/, /reference/durability.html) [E]
- matklad, "Durable Incrementality." https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html [E/P]

**Filesystem watching**
- notify crate. https://docs.rs/notify/latest/notify/ ; README (platforms). https://github.com/notify-rs/notify [E]
- inotify(7). https://man7.org/linux/man-pages/man7/inotify.7.html [E]
- kqueue(2) (FreeBSD). https://man.freebsd.org/cgi/man.cgi?query=kqueue&sektion=2 [E]
- Apple FileSystemEventSecurity. https://developer.apple.com/library/mac/documentation/Darwin/Conceptual/FSEvents_ProgGuide/FileSystemEventSecurity/FileSystemEventSecurity.html [E]; FSEvents.h primer (mirror). https://github.com/zchee/go.fsevents/blob/master/INTERNALS.md [P/E]

**Source-quality caveats:** racy-git.txt raw mirrors returned empty (used git-scm.com rendered page,
double-confirmed by gix rustdoc); rkyv validation.html timed out (FAQ + crate docs used) and rkyv 0.8
version-stability partly from release-note search snippets — verify before adopting rkyv; Bazel
`--guard_against_concurrent_changes` default partly from a snippet; Apple live FSEvents docs JS-gated
(used Apple header text via mirror); GNU Make and Mercurial canonical pages blocked (FSF man page and a
faithful hgweb mirror substituted). Everything else fetched cleanly or was read verbatim from saved fetches.

---

## 5. Appendix — Optional filesystem watching (acceleration, never correctness)

Kept deliberately at the end and OPTIONAL: correctness must not depend on it.

- The **notify** crate abstracts platform backends: inotify (Linux), FSEvents or
  kqueue (macOS), ReadDirectoryChangesW (Windows), kqueue (BSD), polling
  everywhere; `recommended_watcher()` picks the best per platform [E].
- **Why it can only ever be a hint** [E]: inotify(7) *itself* says "do some
  consistency checking, and rebuild the cache when inconsistencies are detected";
  its queue can overflow (`IN_Q_OVERFLOW`, events lost), is not recursive (a
  watch per directory, bounded by `max_user_watches`), misses mmap writes, and
  does not see network-FS events. kqueue needs one open fd per watched object
  (hits `EMFILE` at scale). FSEvents historically delivers directory-granularity
  events, coalesces with latency, drops events under load and signals
  `kFSEventStreamEventFlagMustScanSubDirs` to demand a rescan. None survive the
  tool not running. This is exactly why git's fsmonitor treats the watcher as a
  speed hint validated against the index/stat [E].
- **Recommended posture for odm:** watcher is opt-in; when present it supplies a
  *dirty-set* that lets the warm path skip `lstat`-ing untouched files. On
  watcher startup, any overflow/`MustScanSubDirs`, tool restart, or unsupported
  FS (network/pseudo/emulated), fall back to a full reconciling stat-walk — the
  same source of truth as the no-watcher path. notify's **PollWatcher** is, by
  construction, a stat-walk on a timer (not limited by watch descriptors, latency
  = poll interval, CPU ∝ tree size) and is the natural universal fallback — which
  is why falling back to polling never sacrifices correctness, only latency [E/P].
