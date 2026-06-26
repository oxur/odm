# CDC Verification — Arc 04 / Slice 02: Cold-path build

> Independent verification of CC's closed ledger (impl `d03d5d0`; closed `169cc86`),
> per LEDGER-DISCIPLINE v2.0 (slice scale, §A). CDC reproduces structural rows here;
> cargo rows route to CI / local 1.85+.

## Environment constraint (disclosed)

No 1.85+ toolchain in the sandbox; cargo executions route to CI / CC's local 1.95.0 run
(held **attested pending CI**). CDC reproduces structural rows by reading the branch
`arc04-slice02-cold-path-build` at `d03d5d0`.

## Row dispositions

**Row count:** 9 opened, 9 addressed. No silent drops. ✔

**Reproduced by CDC (structural, at `d03d5d0`):**

- **B-1** — `Store::node_paths()` factored out (`store.rs:81`); `load_all` consumes it
  (`store.rs:116`); the cold build calls `store.node_paths()` (`build.rs:78`). The
  `nodes/YYYY/MM` traversal lives once, reused — not re-derived. ✔
- **B-2/B-3/B-4** — per-file population in `build.rs`: stat fields, `content_hash` (raw
  bytes), and metadata from the parsed `Document` (`node_type`, `gates` from
  `status().reached()`, `tags`, `title` = `name()`, `updated`). ✔
- **B-5** — `EdgeRef { kind, qualifier: Option<EdgeQualifier> }`; `EdgeQualifier`
  preserves `depends_on.satisfied_at`, `Supersede(SupersedeKind)`, and the tear
  `because`; `map_edges` covers all kinds. **Decision: full qualifier fidelity** (the
  slice-doc's open question, resolved). ✔
- **B-6** — `meta_fingerprint(MetaInput { node_type, gates, tags, edges, title })`;
  `updated` and stat fields are **excluded** (built separately, not hashed) → a
  body-only or `updated`-only change leaves `meta_hash` unchanged. The exact property
  slice05's early cutoff needs. ✔
- **B-7** — assembles `Snapshot::new(now, records)` and persists via slice01's
  `Snapshot::persist` (reuse, not reimplemented). ✔
- **postcard trap** — no `skip_serializing_if` anywhere in `odm-index/src` (a skipped
  field desyncs a non-self-describing stream); the regression seed
  (`tests/snapshot.proptest-regressions`) is committed. ✔
- **B-9 (no `unsafe`)** — grep over `odm-index/src` → no matches. ✔

**Attested by CC (local rustc 1.95.0), pending CI:** `cargo test -p odm-index` → 16
passed; full workspace → 246 passed; clippy `-D warnings` → exit 0; `fmt --check` clean;
line coverage **98.68%**. → **PENDING CI** (`attested → reproduced`).

## Bubble-up check (PM Part IV / LEDGER v2.0 §A)

- **Delivered its assigned piece?** ✔ — the cold path (walk → stat+hash+parse → records
  → persisted snapshot), exactly slice02's scope; every "Out" item held out (no warm
  detection, no consumers, no early-cutoff consumption, no benchmark).
- **Silent-drop diff honest?** ✔ — 9/9 dispositioned; no drops.
- **Findings dispositioned + arc-plan updated?** ✔ — arc-plan **v1.4** routes all three:
  EdgeRef qualifiers → slice04 (graph-build/orient read satisfaction + tears from the
  index, strengthening A-10); meta_hash field set → slice05; the postcard no-skip rule
  as a standing format constraint.
- **Convention check:** A-2's Status is `open` with the strength in Evidence — the
  convention CDC normalized on A-1 last slice **propagated correctly**. No fix needed.

## Rulings on CC's flagged items

1. **`EdgeRef` enriched; `FORMAT_VERSION` stays 1.** **Accepted** — sound: no on-disk
   index exists (the crate is wired into no command), so changing the record shape
   between slice01 and slice02 breaks nothing in the wild. **Watch-item carried to
   slice04:** it is the first slice that wires a *persisting* consumer, so it owns the
   obligation to freeze the record shape or bump `FORMAT_VERSION` — and, per finding #3,
   any future optional field is a version bump, never a `skip`.
2. **`meta_hash` excludes `updated` + stat.** **Accepted** — the load-bearing early-cutoff
   decision: the derived fingerprint tracks *meaning* (type/gates/tags/edges/title), so a
   bookkeeping or body-only change does not force downstream recompute. Verified the
   field set in `meta_fingerprint`.
3. **postcard `skip_serializing_if` desync (caught via a transient proptest).**
   **Accepted, and commended** — a genuine, subtle correctness bug (a skipped field
   misaligns a non-self-describing stream), fixed by always-serializing, regression seed
   committed, *and* generalized into a standing rule. Turning a one-off catch into a
   format constraint is the trending discipline working.

## CDC plan-keeping

**None needed this slice.** CC's bubble-up was complete and accurate; the slice04 /
slice05 body lines remain correct (the findings *strengthen* them, not contradict);
and the A-2 status convention was applied correctly. (Contrast slice01, which needed a
body propagation + a convention normalization — the discipline is settling in.)

## Verdict

**Arc 04 / Slice 02 CDC-verified on structure; all three flags accepted (one a
real bug caught pre-ship); cargo rows pending CI.** The cold build fills the snapshot
with full-fidelity records — qualifier-preserving edges and a semantic `meta_hash` —
on a single reused walk seam. On CI green, A-2 flips `open → done`. slice03 (warm-path
change detection) can now take the incremental path against a built index.

CDC: planning thread, 2026-06-26. Iterations used: 1.
